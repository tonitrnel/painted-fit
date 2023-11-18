use crate::parser::MessageField;
use crate::writer::{Visibility, Writer};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref NOW_UTC: DateTime<Utc> = Utc::now();
}

pub fn process_types(types: &crate::parser::Types, version: &str) {
    let dest = crate::ROOT_DIR.join("src/profile/types.rs");
    let mut writer = Writer::new(&dest);
    write_file_header(&mut writer, version);

    writer.write_inner_attribute(vec![
        "allow(missing_docs)",
        "allow(dead_code, unused)",
        "allow(clippy::unreadable_literal, clippy::enum_variant_names, clippy::upper_case_acronyms)",
    ]);
    writer.write_import_packages(vec!["std::fmt", "crate::fit"]);
    writer.write_newline();

    for t in types {
        writer.write_outer_attribute(vec![
            "derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)",
        ]);
        let enum_name = to_pascal_case(&t.type_name);
        let base_type = to_pascal_case(&t.base_type);
        writer.write_enum(
            &enum_name,
            |writer| {
                if t.values.is_empty() {
                    writer.write_enum_member(
                        &format!("Value({})", try_convert_to_rust_ty(&t.base_type).unwrap()),
                        None,
                    )
                } else {
                    for value in &t.values {
                        if let Some(comment) = &value.comment {
                            writer.write_comment(comment);
                        }
                        writer.write_enum_member(&to_pascal_case(&value.name), None)
                    }
                }
            },
            Visibility::Public,
        );
        writer.write_impl(&enum_name, Some("fmt::Display"), |writer| {
            writer.write_fn(
                "fmt",
                vec!["&self", "f: &mut fmt::Formatter<'_>"],
                Some("fmt::Result"),
                |writer| {
                    writer.write_block("match self", |writer| {
                        if t.values.is_empty() {
                            writer.write_line(format!(
                                "{enum_name}::Value(val) => f.write_str(&val.to_string()),"
                            ));
                        } else {
                            for value in &t.values {
                                writer.write_line(format!(
                                    "{enum_name}::{} => f.write_str(\"{}\"),",
                                    to_pascal_case(&value.name),
                                    value.name
                                ));
                            }
                        }
                    })
                },
                Visibility::Private,
            );
        });
        writer.write_impl(&enum_name, None, |writer| {
            writer.write_fn(
                "value",
                vec!["&self"],
                Some("fit::Value"),
                |writer| {
                    writer.write_block("match self", |writer| {
                        if t.values.is_empty() {
                            writer.write_line(format!(
                                "{enum_name}::Value(val) => fit::Value::{base_type}(*val),",
                            ));
                        } else {
                            for value in &t.values {
                                writer.write_line(format!(
                                    "{enum_name}::{} => fit::Value::{base_type}({}),",
                                    to_pascal_case(&value.name),
                                    value.value
                                ));
                            }
                        }
                    });
                },
                Visibility::Public,
            );
            writer.write_fn(
                "base_type",
                vec![],
                Some("&'static str"),
                |writer| {
                    writer.write_line(quote_text(&t.base_type));
                },
                Visibility::Public,
            );
        });
        writer.write_impl(&enum_name, Some("TryFrom<&fit::Value>"), |writer| {
            writer.write_type("Error", "&'static str", Visibility::Private);
            writer.write_fn(
                "try_from",
                vec!["value: &fit::Value"],
                Some("Result<Self, Self::Error>"),
                |writer| {
                    writer.write_block("match value", |writer| {
                        if t.values.is_empty() {
                            writer.write_line(format!(
                                "fit::Value::{base_type}(val) => Ok({enum_name}::Value(*val)),"
                            ));
                            writer.write_line(format!(
                                "_ => Err(\"No corresponding {enum_name} exists\"),",
                            ));
                        } else {
                            for value in &t.values {
                                writer.write_line(format!(
                                    "fit::Value::{base_type}({}) => Ok({enum_name}::{}),",
                                    value.value,
                                    to_pascal_case(&value.name)
                                ));
                            }
                            writer.write_line(format!(
                                "_ => Err(\"No corresponding {enum_name} exists\"),",
                            ));
                        }
                    })
                },
                Visibility::Private,
            );
        });
        if t.values.is_empty() {
            continue;
        }
        writer.write_impl(&enum_name, Some("TryFrom<&str>"), |writer| {
            writer.write_type("Error", "&'static str", Visibility::Private);
            writer.write_fn(
                "try_from",
                vec!["value: &str"],
                Some("Result<Self, Self::Error>"),
                |writer| {
                    writer.write_block("match value", |writer| {
                        for value in &t.values {
                            writer.write_line(format!(
                                "\"{}\" => Ok({enum_name}::{}),",
                                value.name,
                                to_pascal_case(&value.name)
                            ));
                        }
                        writer.write_line(format!(
                            "_ => Err(\"No corresponding {enum_name} exists\"),",
                        ));
                    })
                },
                Visibility::Private,
            );
        });
        writer.write_newline();
    }
    writer.fmt();
}

pub fn process_messages(
    messages: &crate::parser::Messages,
    types: &crate::parser::Types,
    version: &str,
) {
    let dest = crate::ROOT_DIR.join("src/profile/messages.rs");
    let mut writer = Writer::new(&dest);
    write_file_header(&mut writer, version);

    writer.write_inner_attribute(vec![
        "allow(missing_docs)",
        "allow(dead_code, unused)",
        "allow(clippy::unreadable_literal, clippy::type_complexity)",
    ]);
    writer.write_import_packages(vec![
        "super::types",
        "crate::bit_reader::BitReader",
        "crate::fit",
        "chrono::{TimeZone, Utc}",
        "std::fmt",
        "std::collections::HashMap",
        "std::ops::{Deref, Div, Sub}",
        "std::borrow::Cow",
    ]);
    writer.write_newline();
    write_message_prelude(&mut writer);
    let types_map = types
        .iter()
        .map(|it| (it.type_name.to_owned(), it))
        .collect::<HashMap<_, _>>();
    let mut emitted_messages = Vec::new();
    for message in messages {
        if let Some(comment) = &message.comment {
            writer.write_comment(comment);
        }
        writer.write_fn(
            &format!("_{}", message.name),
            vec![
                "message_map: &mut HashMap<&'static str, Field>",
                "accumulator: &mut crate::accumulator::Accumulator",
                "args: MessageDecodeArgs",
            ],
            Some("Result<(), String>"),
            |writer| {
                writer
                    .write_line("match args.field_no{")
                    .increase_indentation();
                for field in &message.fields {
                    writer.write_block_with_end_symbol(
                        &format!("{} =>", field.field_no),
                        |writer| {
                            process_message_field(writer, field, &message.fields, &types_map);
                            writer.write_line("Ok(())");
                        },
                        ",",
                    );
                }
                writer.write_line(format!("_ => Err(format!(\"'{}' message does not exist field def number: '{{}}'\", args.field_no))", message.name));
                writer.decrease_indention().write_line("}");
            },
            Visibility::Private,
        );
        emitted_messages.push(message.name.to_owned());
    }
    writer.write_fn(
        "from_message_type",
        vec!["message_type: &str"],
        Some("Option<MessageDecoder>"),
        |writer| {
            writer.write_block("match message_type", |writer| {
                for emitted in emitted_messages {
                    writer.write_line(&format!("\"{emitted}\" => Some(Box::new(_{emitted})),"));
                }
                writer.write_line("_ => None,");
            });
        },
        Visibility::Crate,
    );
    writer.fmt();
}

pub fn process_version(version: &str) {
    let dest = crate::ROOT_DIR.join("src/profile/version.rs");
    let mut writer = Writer::new(&dest);
    write_file_header(&mut writer, version);
    writer.write_line(format!(
        "pub const VERSION: &str = {};",
        quote_text(version)
    ));
}

fn write_file_header(writer: &mut Writer, version: &str) {
    use chrono::{Datelike, Timelike};

    writer.write_header("# ======================================================== #");
    writer.write_header("#                  ****WARNING****                     ");
    writer.write_header("# This file is auto-generated!  Do NOT edit this file. ");
    writer.write_header(&format!("# Profile Version = {version}"));
    writer.write_header(&format!(
        "# Generated Date = {year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}",
        year = NOW_UTC.year(),
        month = NOW_UTC.month(),
        day = NOW_UTC.day(),
        hour = NOW_UTC.hour(),
        minute = NOW_UTC.minute(),
        second = NOW_UTC.second()
    ));
    writer.write_header("# ======================================================== #");
}
fn write_message_prelude(writer: &mut Writer) {
    writer.write_code_fragment(
        r#"
        #[derive(Debug, Clone)]
        pub struct Field {
            pub name: &'static str,
            pub value: fit::Value,
            pub is_subfield: bool,
            pub units: &'static str,
        }
        impl fmt::Display for Field{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.value.fmt(f)
            }
        }
        #[derive(Debug, Clone)]
        pub struct MessageDecodeArgs<'input> {
            pub msg_ty: &'input types::MesgNum,
            pub msg_no: u16,
            pub field_no: u8,
            pub value: &'input fit::Value,
            pub fields: &'input HashMap<u8, fit::Value>,
        }

        struct TransformValueArgs<'input, R: ToString> {
            pub field_ty: &'static str,
            pub msg_ty: &'input types::MesgNum,
            pub scale: f64,
            pub offset: f64,
            pub ty_to_str: Box<dyn Fn(&fit::Value) -> Option<R>>,
            pub is_base_type: bool,
            pub array: Option<usize>,
        }

        impl Sub<f64> for fit::Value {
            type Output = Result<fit::Value, &'static str>;
            fn sub(self, rhs: f64) -> Self::Output {
                if rhs == 0.0 { return Ok(self);  }
                match self {
                    fit::Value::SInt8(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::UInt8(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::SInt16(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::UInt16(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::SInt32(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::UInt32(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::Float32(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::Float64(val) => Ok(fit::Value::Float64(val - rhs)),
                    fit::Value::UInt8z(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::UInt16z(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::UInt32z(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::Byte(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::SInt64(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::UInt64(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    fit::Value::UInt64z(val) => Ok(fit::Value::Float64(val as f64 - rhs)),
                    _ => Err("Unsupported operation: Value variant cannot be subtracted with f64"),
                }
            }
        }
        impl Div<f64> for fit::Value {
            type Output = Result<fit::Value, &'static str>;
            fn div(self, rhs: f64) -> Self::Output {
                if rhs == 1.0 { return Ok(self)  }
                match self {
                    fit::Value::SInt8(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::UInt8(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::SInt16(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::UInt16(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::SInt32(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::UInt32(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::Float32(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::Float64(val) => Ok(fit::Value::Float64(val / rhs)),
                    fit::Value::UInt8z(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::UInt16z(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::UInt32z(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::Byte(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::SInt64(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::UInt64(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    fit::Value::UInt64z(val) => Ok(fit::Value::Float64(val as f64 / rhs)),
                    _ => Err("Unsupported operation: Value variant cannot be divided with f64"),
                }
            }
        }

        fn transform_value<R: ToString>(
            value: Cow<fit::Value>,
            args: TransformValueArgs<R>,
        ) -> Result<fit::Value, String> {
            let value = value.deref();
            if matches!(
                args.msg_ty,
                types::MesgNum::DeveloperDataId | types::MesgNum::FieldDescription
            ) {
                return Ok(value.clone());
            }
            if let fit::Value::Array(arr) = value {
                if let Some(len) = args.array {
                    if len != 0 && len != arr.len() {
                        return Err(format!(
                            "expected an array length of '{expected}', but got an array length of '{actual}'",
                           expected = len,
                           actual= arr.len()
                        ));
                    }
                } else {
                    return Err(format!(
                        "Expected field type to be array, but got type '{}'",
                        value
                    ));
                }
            }
            if args.field_ty == "string" {
                Ok(value.clone())
            } else if args.field_ty == "date_time" {
                return if let fit::Value::UInt32(timestamp) = value {
                    // The second offset between UNIX and FIT Epochs (631065600).
                    Ok(fit::Value::DateTime(
                        Utc.timestamp_opt(*timestamp as i64 + 631065600, 0).unwrap(),
                    ))
                } else {
                    Err(format!(
                        "Expected field type to be '{}' but got type '{}'",
                        types::DateTime::base_type(),
                        value
                    ))
                };
            }  else if args.is_base_type
                && fit::BaseType::try_from(args.field_ty)
                    .ok()
                    .filter(|ty| ty.is_numeric())
                    .is_some()
            {
                return if let fit::Value::Array(arr) = value {
                    Ok(fit::Value::Array(
                        arr.iter()
                            .map(|it| {
                                it.to_owned()
                                    .div(args.scale)
                                    .and_then(|it| it.sub(args.offset))
                                    .unwrap()
                            })
                            .collect::<Vec<_>>(),
                    ))
                } else {
                    Ok(value
                        .to_owned()
                        .div(args.scale)
                        .and_then(|it| it.sub(args.offset))
                        .unwrap())
                };
            } else {
                let ty_convert = |value: &fit::Value| {
                    args.ty_to_str.deref()(value)
                        .map(|it| fit::Value::String(it.to_string()))
                        .unwrap_or(value.clone())
                };
                return if let fit::Value::Array(arr) = value {
                    Ok(fit::Value::Array(
                        arr.iter().map(ty_convert).collect::<Vec<_>>(),
                    ))
                } else {
                    Ok(ty_convert(value))
                };
            }
        }
        "#
    );
    writer.write_code_fragment("pub type MessageDecoder = Box<dyn Fn(&mut HashMap<&'static str, Field>, &mut crate::accumulator::Accumulator, MessageDecodeArgs) -> Result<(), String>>;");
}
struct WriteMessageFieldArgs<'a> {
    value_source: &'a str,
    field_name: &'a str,
    field_type: &'a str,
    scale: &'a f64,
    offset: &'a f64,
    units: &'a str,
    array: &'a Option<usize>,
    is_base_type: bool,
    is_sub_field: bool,
}
fn write_message_field(writer: &mut Writer, args: WriteMessageFieldArgs) {
    let ty_to_str_def = if args.is_base_type {
        "Box::new(|val| Some(val.to_string()))".to_string()
    } else {
        format!(
            "Box::new(|val| types::{fit_ty}::try_from(val).ok())",
            fit_ty = to_pascal_case(args.field_type)
        )
    };
    writer.write_call(
        "message_map.insert",
        vec![format!(
            r"
                {field_name},
                Field {{
                    name: {field_name},
                    value: transform_value(
                        {value_source},
                        TransformValueArgs {{
                            field_ty: {field_ty},
                            msg_ty: args.msg_ty,
                            scale: {scale},
                            offset: {offset},
                            array: {array},
                            ty_to_str: {ty_to_str_def},
                            is_base_type: {is_base_type}
                        }},
                    )
                    .unwrap(),
                    units: {units},
                    is_subfield: {is_sub_field},
                }}
                ",
            field_name = quote_text(args.field_name),
            field_ty = quote_text(args.field_type),
            value_source = args.value_source,
            scale = format_float(*args.scale),
            offset = format_float(*args.offset),
            units = quote_text(args.units),
            array = args
                .array
                .map(|it| format!("Some({it})"))
                .unwrap_or("None".to_string()),
            is_base_type = args.is_base_type.to_string(),
            is_sub_field = args.is_sub_field.to_string()
        )],
    );
}

type TypesMap<'a> = HashMap<String, &'a crate::parser::Type>;
type MessageFields<'a> = [MessageField];
type MessageComponent = (u8, MessageField);

fn process_message_field(
    writer: &mut Writer,
    field: &MessageField,
    all: &MessageFields,
    types_map: &TypesMap,
) {
    write_message_field(
        writer,
        WriteMessageFieldArgs {
            value_source: "Cow::Borrowed(args.value)",
            field_name: &field.field_name,
            field_type: &field.field_type,
            scale: &field.scale,
            offset: &field.offset,
            units: &field.units,
            array: &field.array,
            is_base_type: check_is_base_type(types_map, &field.field_type),
            is_sub_field: false,
        },
    );
    if field.accumulate {
        writer.write_call(
            "accumulator.add",
            vec![
                "args.msg_no",
                "args.field_no",
                "args.value.try_as_usize().unwrap()",
            ],
        )
    }
    if !field.sub_fields.is_empty() {
        writer.write_line("// expansion sub fields");
        process_message_sub_fields(writer, types_map, all, field);
    }
    if !field.components.is_empty() {
        writer.write_line("// expansion components");
        process_message_components(writer, types_map, &field.components);
    }
}
fn process_message_sub_fields(
    writer: &mut Writer,
    types_map: &TypesMap,
    all: &MessageFields,
    field: &MessageField,
) {
    for sub_field in &field.sub_fields {
        let ref_field = all
            .iter()
            .find(|it| it.field_name == sub_field.ref_field_name);
        let ref_field = if let Some(ref_field) = ref_field {
            ref_field
        } else {
            return;
        };
        let ref_type = types_map.get(&ref_field.field_type).and_then(|ty| {
            ty.values.iter().find_map(|it| {
                if it.name == sub_field.ref_field_value {
                    Some((ty, it))
                } else {
                    None
                }
            })
        });

        let ref_type_value = if let Some((ref_type, ref_type_value)) = ref_type {
            format!(
                "fit::Value::{}({})",
                to_pascal_case(&ref_type.base_type),
                ref_type_value.value
            )
        } else {
            return;
        };
        writer.write_block(
            &format!(
                "if args.fields.get(&{}u8) == Some(&{})",
                ref_field.field_no, ref_type_value
            ),
            |writer| {
                let mut value_source = "args.value".to_string();
                // convert type
                let converted = convert_type(
                    writer,
                    types_map,
                    &mut value_source,
                    &field.field_type,
                    &sub_field.field_type,
                );
                if !converted {
                    value_source = format!("Cow::Borrowed({value_source})");
                }
                write_message_field(
                    writer,
                    WriteMessageFieldArgs {
                        value_source: &value_source,
                        field_name: &sub_field.field_name,
                        field_type: &sub_field.field_type,
                        scale: &sub_field.scale,
                        offset: &sub_field.offset,
                        units: &sub_field.units,
                        array: &sub_field.array,
                        is_base_type: check_is_base_type(types_map, &sub_field.field_type),
                        is_sub_field: true,
                    },
                );
            },
        );
        if !sub_field.components.is_empty() {
            process_message_components(writer, types_map, &sub_field.components)
        }
    }
}
fn process_message_components(
    writer: &mut Writer,
    types_map: &TypesMap,
    components: &[MessageComponent],
) {
    let components = group_components(components);
    writer.write_line("let mut bit_reader = BitReader::new(args.value.clone());");
    for (repeat, bits, component) in components {
        let base_ty = get_base_type(types_map, &component.field_type);
        if repeat > 1 {
            writer.write_code_fragment(format!(
                r"
                let value = {{
                    let mut values = Vec::new();
                    for _ in 0..{repeat}{{
                        let bit = if let Some(bit) = bit_reader.read_bits({bits}){{
                            bit
                        }} else {{
                            break
                        }};
                        let value = fit::Value::{base_ty}(accumulator.accumulate(
                            args.msg_no,
                            args.field_no,
                            bit,
                            {bits},
                        ) as {rust_ty});
                        values.push(value);
                    }}
                    fit::Value::Array(values)
                }};",
                repeat = repeat,
                bits = bits,
                base_ty = to_pascal_case(&base_ty),
                rust_ty = try_convert_to_rust_ty(&base_ty).unwrap()
            ));
        } else {
            writer.write_code_fragment(format!(
                r"
                let value = fit::Value::{base_ty}(accumulator.accumulate(
                    args.msg_no,
                    args.field_no,
                    bit_reader.read_bits({bits}).unwrap(),
                    {bits},
                ) as {rust_ty});",
                bits = bits,
                base_ty = to_pascal_case(&base_ty),
                rust_ty = try_convert_to_rust_ty(&base_ty).unwrap()
            ));
        }
        write_message_field(
            writer,
            WriteMessageFieldArgs {
                value_source: "Cow::Owned(value)",
                field_name: &component.field_name,
                field_type: &component.field_type,
                scale: &component.scale,
                offset: &component.offset,
                units: &component.units,
                array: &component.array,
                is_base_type: check_is_base_type(types_map, &component.field_type),
                is_sub_field: false,
            },
        )
    }
}

/// Convert snake case style to pascal case style
///
/// # Example
/// ```rust
/// to_pascal_case("message_type"); // => MessageType
/// to_pascal_case("uint8"); // => UInt8
/// to_pascal_case("sint64"); // => SInt8
/// to_pascal_case("activity_summary"); // => ActivitySummary
/// ```
fn to_pascal_case(s: &str) -> String {
    let s = s
        .split('_')
        .enumerate()
        .map(|(i, word)| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(char) if i == 0 && char.is_ascii_digit() => {
                    format!("N{}{}", char.to_uppercase(), chars.as_str())
                }
                Some(char) => char.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<String>();
    if s.starts_with("Sint") {
        s.replace("Sint", "SInt")
    } else if s.starts_with("Uint") {
        s.replace("Uint", "UInt")
    } else {
        s
    }
}

fn convert_type(
    writer: &mut Writer,
    types_map: &TypesMap,
    value_source: &mut String,
    old_ty: &str,
    new_ty: &str,
) -> bool {
    if old_ty == new_ty {
        return false;
    }
    let old_ty = if check_is_base_type(types_map, old_ty) {
        (try_convert_to_rust_ty(old_ty).unwrap(), old_ty.to_owned())
    } else {
        let base_type = types_map
            .get(old_ty)
            .map(|it| it.base_type.to_owned())
            .unwrap();
        (try_convert_to_rust_ty(&base_type).unwrap(), base_type)
    };
    let new_ty = if check_is_base_type(types_map, new_ty) {
        (try_convert_to_rust_ty(new_ty).unwrap(), new_ty.to_owned())
    } else {
        let base_type = types_map
            .get(new_ty)
            .map(|it| it.base_type.to_owned())
            .unwrap();
        (try_convert_to_rust_ty(&base_type).unwrap(), base_type)
    };
    if old_ty.1 != new_ty.1 {
        // write convert type
        writer.write_line(
            format!(
                "let value = if let fit::Value::{old_base_ty}(v) = {value_source}{{ Cow::Owned(fit::Value::{new_base_ty}(*v as {new_rust_ty})) }} else {{ Cow::Borrowed({value_source}) }};",
                old_base_ty = to_pascal_case(&old_ty.1), new_base_ty = to_pascal_case(&new_ty.1), new_rust_ty = new_ty.0)
        );
        *value_source = "value".to_string();
        true
    } else {
        false
    }
}

fn check_is_base_type(types_map: &TypesMap, ty: &str) -> bool {
    let fit_base_type = types_map.get("fit_base_type").unwrap_or_else(|| {
        panic!("Cannot found fit base type definition, Profile missing 'fit_base_type' type")
    });
    ty == "bool" || fit_base_type.values.iter().any(|it| it.name == ty)
}

fn get_base_type(types_map: &TypesMap, ty: &str) -> String {
    if check_is_base_type(types_map, ty) {
        ty.to_string()
    } else {
        types_map.get(ty).unwrap().base_type.to_owned()
    }
}

/// Try convert fit base type to rust type
///
/// # Example
/// ```rust
/// try_convert_to_rust_ty("uint8"); // => u8
/// ```
fn try_convert_to_rust_ty(base_type: &str) -> Option<String> {
    match base_type {
        "enum" => Some("u8".to_string()),
        "string" => Some("String".to_string()),
        "byte" => Some("u8".to_string()),
        ty if ty.starts_with("sint") => Some(ty.replace("sint", "i").replace('z', "").to_string()),
        ty if ty.starts_with("uint") => Some(ty.replace("uint", "u").replace('z', "").to_string()),
        ty if ty.starts_with("float") => Some(ty.replace("float", "f").to_string()),
        _ => None,
    }
}
fn format_float(num: f64) -> String {
    if num.fract() == 0.0 {
        // 如果没有小数部分，添加 ".0"
        format!("{}.0", num)
    } else {
        // 否则，使用普通的浮点格式化，它会保留所有的小数位
        format!("{}", num)
    }
}
fn quote_text(text: &str) -> String {
    format!("\"{}\"", text)
}
fn group_components(comps: &[(u8, MessageField)]) -> Vec<(u8, u8, MessageField)> {
    let mut counts = HashMap::new();
    let filtered_comps = comps
        .iter()
        .filter(|(_, comp)| {
            let mut filtered = false;
            counts
                .entry(comp.field_name.to_owned())
                .and_modify(|count| *count += 1)
                .or_insert_with(|| {
                    filtered = true;
                    1u8
                });
            filtered
        })
        .collect::<Vec<_>>();
    filtered_comps
        .iter()
        .map(|(bits, comp)| (*counts.get(&comp.field_name).unwrap(), *bits, comp.clone()))
        .collect::<Vec<_>>()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("example_string"), "ExampleString");
        assert_eq!(to_pascal_case("language_bits_1"), "LanguageBits1");
        assert_eq!(to_pascal_case("example"), "Example");
        assert_eq!(
            to_pascal_case("90_degree_cable_external_rotation"),
            "N90DegreeCableExternalRotation"
        );
    }
}
