#![allow(unused, dead_code)]

use crate::parser::MessageField;
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

struct Writer<'a> {
    path: &'a PathBuf,
    file: File,
    indent: usize,
}
enum Visibility {
    Public,
    Crate,
    Super,
    Private,
}
impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => f.write_str("pub"),
            Visibility::Crate => f.write_str("pub(crate)"),
            Visibility::Super => f.write_str("pub(super)"),
            Visibility::Private => f.write_str(""),
        }
    }
}
impl<'a> Writer<'a> {
    pub(crate) fn new(path: &'a PathBuf) -> Self {
        Writer {
            path,
            indent: 0,
            file: OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .unwrap_or_else(|_| panic!("Failed to open file at path {path:?}")),
        }
    }
    pub(crate) fn write_line<S>(&mut self, line: S) -> &mut Writer<'a>
    where
        S: ToString + std::fmt::Display,
    {
        let indent = " ".repeat(self.indent);
        self.file
            .write_all(format!("{indent}{}\n", line.to_string().trim()).as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write file"));
        self
    }
    pub(crate) fn write_newline(&mut self) {
        self.file
            .write_all(&[0x0Au8])
            .unwrap_or_else(|_| panic!("Failed to write file"));
    }
    pub(crate) fn write_inner_attribute(&mut self, attributes: Vec<&str>) {
        for attribute in attributes {
            self.write_line(format!("#![{attribute}]"));
        }
    }
    pub(crate) fn write_outer_attribute(&mut self, attributes: Vec<&str>) {
        for attribute in attributes {
            self.write_line(format!("#[{attribute}]"));
        }
    }
    pub(crate) fn write_import_packages(&mut self, crates: Vec<&str>) {
        for _crate in crates {
            self.write_line(format!("use {_crate};"));
        }
    }
    pub(crate) fn write_comment(&mut self, comment: &str) {
        self.write_line(format!("/// {comment}"));
    }
    pub(crate) fn write_header(&mut self, comment: &str) {
        self.write_line(format!("//! {comment}"));
    }
    pub(crate) fn write_enum<S>(&mut self, name: &str, scope: S, visibility: Visibility)
    where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{visibility} enum {name}{{"));

        self.scope(scope);
        self.write_line("}");
    }
    pub(crate) fn write_struct<S>(&mut self, name: &str, scope: S, visibility: Visibility)
    where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{visibility} struct {name}{{"));
        self.scope(scope);
        self.write_line("}");
        self.write_newline();
    }
    pub(crate) fn write_enum_member(&mut self, name: &str, value: Option<String>) {
        let value = if let Some(value) = value {
            format!(" => {value}")
        } else {
            String::new()
        };
        self.write_line(format!("{name}{value},"));
    }
    pub(crate) fn write_struct_member(&mut self, name: &str, value: &str, visibility: Visibility) {
        self.write_line(format!("{visibility} {name}: {value},"));
    }
    pub(crate) fn write_impl<S>(&mut self, type_name: &str, trait_name: Option<&str>, scope: S)
    where
        S: FnOnce(&mut Self),
    {
        if let Some(trait_name) = trait_name {
            self.write_line(format!("impl {trait_name} for {type_name}{{"));
        } else {
            self.write_line(format!("impl {type_name}{{"));
        }
        self.scope(scope);
        self.write_line("}");
    }
    pub(crate) fn write_code_fragment(&mut self, part: &str) {
        let lines = part.split('\n').collect::<Vec<_>>();
        if lines.is_empty() {
            return;
        }
        if lines.len() == 1 {
            self.write_line(lines[0].trim_start());
            return;
        }
        let lines = if lines[0].trim().is_empty() {
            &lines[1..]
        } else {
            &lines[..]
        };
        let indent = lines[0].chars().take_while(|c| c.is_whitespace()).count() - 4;
        for line in part.split('\n') {
            self.write_line(
                line.char_indices()
                    // .skip_while(|(pos, char)| *pos < indent && char.is_whitespace())
                    .map(|(_, char)| char)
                    .collect::<String>(),
            );
        }
    }
    pub(crate) fn scope<S>(&mut self, scope: S)
    where
        S: FnOnce(&mut Self),
    {
        self.increase_indentation();
        scope(self);
        self.decrease_indention();
    }
    pub(crate) fn increase_indentation(&mut self) -> &mut Writer<'a> {
        self.indent += 4;
        self
    }
    pub(crate) fn decrease_indention(&mut self) -> &mut Writer<'a> {
        self.indent -= 4;
        self
    }
    pub(crate) fn write_type(&mut self, name: &str, value: &str, visibility: Visibility) {
        self.write_line(format!("{visibility} type {name} = {value};"));
    }
    pub(crate) fn write_fn<S>(
        &mut self,
        name: &str,
        parameters: Vec<&str>,
        return_type: Option<&str>,
        scope: S,
        visibility: Visibility,
    ) where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!(
            "{visibility} fn {name}({}){}{{",
            parameters.join(", "),
            if let Some(return_type) = return_type {
                format!(" -> {return_type}")
            } else {
                String::new()
            }
        ));
        self.scope(scope);
        self.write_line("}");
    }
    pub(crate) fn write_block<S>(&mut self, statement: &str, scope: S)
    where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{statement} {{"));
        self.scope(scope);
        self.write_line("}");
    }
    pub(crate) fn write_call<S>(&mut self, func: &str, parameters: Vec<S>)
    where
        S: ToString + std::fmt::Display,
    {
        self.write_line(format!("{func}(")).increase_indentation();
        for parameter in parameters {
            self.write_code_fragment(&format!("{parameter},"));
        }
        self.decrease_indention().write_line(");");
    }
    pub(crate) fn write_block_with_end_symbol<S>(
        &mut self,
        statement: &str,
        scope: S,
        end_symbol: &str,
    ) where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{statement} {{"));
        self.scope(scope);
        self.write_line(format!("}}{}", end_symbol));
    }
    pub(crate) fn fmt(&self) {
        let path = self.path;
        std::process::Command::new("rustfmt")
            .arg(path)
            .status()
            .unwrap_or_else(|_| panic!("failed to execute rustfmt on {path:?}"));
    }
}

pub(crate) fn process_types(types: &crate::parser::Types) {
    let dest = crate::ROOT_DIR.join("src/profile/types.rs");
    let mut writer = Writer::new(&dest);
    writer.write_header("========================================================");
    writer.write_header("|                  ****WARNING****                     |");
    writer.write_header("| This file is auto-generated!  Do NOT edit this file. |");
    writer.write_header("| Profile Version = ???                                |");
    writer.write_header("| Build Date = ???                                     |");
    writer.write_header("========================================================");

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
            Visibility::Crate,
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
                Visibility::Crate,
            );
            writer.write_fn(
                "base_type",
                vec![],
                Some("&'static str"),
                |writer| {
                    writer.write_line(quote_text(&t.base_type));
                },
                Visibility::Crate,
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

fn write_message_prelude(writer: &mut Writer) {
    writer.write_code_fragment(
        r#"
        #[derive(Debug, Clone)]
        pub(crate) struct Field {
            pub(crate) name: &'static str,
            pub(crate) value: fit::Value,
            pub(crate) is_subfield: bool,
        }
        impl fmt::Display for Field{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.value.fmt(f)
            }
        }
        #[derive(Debug, Clone)]
        pub(crate) struct MessageDecodeArgs<'input> {
            pub(crate) msg_ty: &'input types::MesgNum,
            pub(crate) msg_no: u16,
            pub(crate) field_no: u8,
            pub(crate) value: &'input fit::Value,
            pub(crate) fields: &'input HashMap<u8, fit::Value>,
        }

        struct TransformValueArgs<'input, R: ToString> {
            pub field_ty: &'static str,
            pub msg_ty: &'input types::MesgNum,
            pub scale: f64,
            pub offset: f64,
            pub units: &'static str,
            pub ty_to_str: Box<dyn Fn(&fit::Value) -> Option<R>>,
            pub is_base_type: bool,
            pub array: Option<usize>,
        }

        impl Sub<f64> for fit::Value {
            type Output = Result<fit::Value, &'static str>;
            fn sub(self, rhs: f64) -> Self::Output {
                match self {
                    fit::Value::SInt8(val) => Ok(fit::Value::SInt8((val as f64 - rhs) as i8)),
                    fit::Value::UInt8(val) => Ok(fit::Value::UInt8((val as f64 - rhs) as u8)),
                    fit::Value::SInt16(val) => Ok(fit::Value::SInt16((val as f64 - rhs) as i16)),
                    fit::Value::UInt16(val) => Ok(fit::Value::UInt16((val as f64 - rhs) as u16)),
                    fit::Value::SInt32(val) => Ok(fit::Value::SInt32((val as f64 - rhs) as i32)),
                    fit::Value::UInt32(val) => Ok(fit::Value::UInt32((val as f64 - rhs) as u32)),
                    fit::Value::Float32(val) => Ok(fit::Value::Float32((val as f64 - rhs) as f32)),
                    fit::Value::Float64(val) => Ok(fit::Value::Float64(val - rhs)),
                    fit::Value::UInt8z(val) => Ok(fit::Value::UInt8z((val as f64 - rhs) as u8)),
                    fit::Value::UInt16z(val) => Ok(fit::Value::UInt16z((val as f64 - rhs) as u16)),
                    fit::Value::UInt32z(val) => Ok(fit::Value::UInt32z((val as f64 - rhs) as u32)),
                    fit::Value::Byte(val) => Ok(fit::Value::Byte((val as f64 - rhs) as u8)),
                    fit::Value::SInt64(val) => Ok(fit::Value::SInt64((val as f64 - rhs) as i64)),
                    fit::Value::UInt64(val) => Ok(fit::Value::UInt64((val as f64 - rhs) as u64)),
                    fit::Value::UInt64z(val) => Ok(fit::Value::UInt64z((val as f64 - rhs) as u64)),
                    _ => Err("Unsupported operation: Value variant cannot be subtracted with f64"),
                }
            }
        }
        impl Div<f64> for fit::Value {
            type Output = Result<fit::Value, &'static str>;
            fn div(self, rhs: f64) -> Self::Output {
                match self {
                    fit::Value::SInt8(val) => Ok(fit::Value::SInt8((val as f64 / rhs) as i8)),
                    fit::Value::UInt8(val) => Ok(fit::Value::UInt8((val as f64 / rhs) as u8)),
                    fit::Value::SInt16(val) => Ok(fit::Value::SInt16((val as f64 / rhs) as i16)),
                    fit::Value::UInt16(val) => Ok(fit::Value::UInt16((val as f64 / rhs) as u16)),
                    fit::Value::SInt32(val) => Ok(fit::Value::SInt32((val as f64 / rhs) as i32)),
                    fit::Value::UInt32(val) => Ok(fit::Value::UInt32((val as f64 / rhs) as u32)),
                    fit::Value::Float32(val) => Ok(fit::Value::Float32((val as f64 / rhs) as f32)),
                    fit::Value::Float64(val) => Ok(fit::Value::Float64(val / rhs)),
                    fit::Value::UInt8z(val) => Ok(fit::Value::UInt8z((val as f64 / rhs) as u8)),
                    fit::Value::UInt16z(val) => Ok(fit::Value::UInt16z((val as f64 / rhs) as u16)),
                    fit::Value::UInt32z(val) => Ok(fit::Value::UInt32z((val as f64 / rhs) as u32)),
                    fit::Value::Byte(val) => Ok(fit::Value::Byte((val as f64 / rhs) as u8)),
                    fit::Value::SInt64(val) => Ok(fit::Value::SInt64((val as f64 / rhs) as i64)),
                    fit::Value::UInt64(val) => Ok(fit::Value::UInt64((val as f64 / rhs) as u64)),
                    fit::Value::UInt64z(val) => Ok(fit::Value::UInt64z((val as f64 / rhs) as u64)),
                    _ => Err("Unsupported operation: Value variant cannot be divided with f64"),
                }
            }
        }

        fn transform_value<R: ToString>(
            value: &fit::Value,
            args: TransformValueArgs<R>,
        ) -> Result<fit::Value, String> {
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
    writer.write_code_fragment("type MessageDecoder = Box<dyn Fn(&mut HashMap<&'static str, Field>, &mut crate::accumulator::Accumulator, MessageDecodeArgs) -> Result<(), String>>;");
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
                            units: {units},
                            array: {array},
                            ty_to_str: {ty_to_str_def},
                            is_base_type: {is_base_type}
                        }},
                    )
                    .unwrap(),
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
fn process_message_field(
    writer: &mut Writer,
    field: &MessageField,
    all: &[MessageField],
    types_map: &HashMap<String, &crate::parser::Type>,
) {
    let fit_base_type = types_map.get("fit_base_type").unwrap_or_else(|| {
        panic!("Cannot found fit base type definition, Profile missing 'fit_base_type' type")
    });
    let check_is_base_type =
        |ty: &str| ty == "bool" || fit_base_type.values.iter().any(|it| it.name == ty);
    write_message_field(
        writer,
        WriteMessageFieldArgs {
            value_source: "args.value",
            field_name: &field.field_name,
            field_type: &field.field_type,
            scale: &field.scale,
            offset: &field.offset,
            units: &field.units,
            array: &field.array,
            is_base_type: check_is_base_type(&field.field_type),
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
        for sub_field in &field.sub_fields {
            let ref_field = all
                .iter()
                .find(|it| it.field_name == sub_field.ref_field_name);
            let ref_field = if let Some(ref_field) = ref_field {
                ref_field
            } else {
                continue;
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
                continue;
            };
            writer.write_block(
                &format!(
                    "if args.fields.get(&{}u8) == Some(&{})",
                    ref_field.field_no, ref_type_value
                ),
                |writer| {
                    write_message_field(
                        writer,
                        WriteMessageFieldArgs {
                            value_source: "args.value",
                            field_name: &sub_field.field_name,
                            field_type: &sub_field.field_type,
                            scale: &sub_field.scale,
                            offset: &sub_field.offset,
                            units: &sub_field.units,
                            array: &sub_field.array,
                            is_base_type: check_is_base_type(&sub_field.field_type),
                            is_sub_field: true,
                        },
                    );
                },
            )
        }
    }
    if !field.components.is_empty() {
        writer.write_line("let mut bit_reader = BitReader::new(args.value.clone());");
        for (bits, component) in &field.components {
            writer.write_code_fragment(&format!(
                r"
            let value = accumulator.accumulate(
                args.msg_no,
                args.field_no,
                bit_reader.read_bits({bits}).unwrap(),
                {bits},
            );",
                bits = bits
            ));
            write_message_field(
                writer,
                WriteMessageFieldArgs {
                    value_source: &format!(
                        "&fit::Value::{}(value as {})",
                        to_pascal_case(&field.field_type),
                        try_convert_to_rust_ty(&field.field_type).unwrap()
                    ),
                    field_name: &component.field_name,
                    field_type: &component.field_type,
                    scale: &component.scale,
                    offset: &component.offset,
                    units: &component.units,
                    array: &component.array,
                    is_base_type: check_is_base_type(&field.field_type),
                    is_sub_field: false,
                },
            )
        }
    }
}

pub(crate) fn process_messages(messages: &crate::parser::Messages, types: &crate::parser::Types) {
    let dest = crate::ROOT_DIR.join("src/profile/messages.rs");
    let mut writer = Writer::new(&dest);
    writer.write_header("========================================================");
    writer.write_header("|                  ****WARNING****                     |");
    writer.write_header("| This file is auto-generated!  Do NOT edit this file. |");
    writer.write_header("| Profile Version = ???                                |");
    writer.write_header("| Build Date = ???                                     |");
    writer.write_header("========================================================");

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
                writer.write_line(r#"_ => Err(format!("'{}' message does not exist '{}' field", args.msg_no, args.field_no))"#);
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

fn to_option_usize_text(text: &Option<String>) -> String {
    text.as_ref()
        .and_then(|it| {
            let text = it.trim();
            if !(text.starts_with('[') && text.ends_with(']')) {
                return None;
            }
            let text = text.trim_start_matches('[').trim_end_matches(']');
            if text == "N" {
                Some(0usize)
            } else {
                text.parse::<usize>().ok()
            }
        })
        .map(|it| format!("Some({})", it))
        .unwrap_or("None".to_string())
}
fn to_static_array_text(text: &Option<String>) -> String {
    text.as_ref()
        .map(|it| {
            it.split(',')
                .map(|it| quote_text(it.trim()))
                .collect::<Vec<_>>()
                .join(",")
        })
        .map(|it| format!("Some(&[{}])", it))
        .unwrap_or("None".to_string())
}
fn to_static_numeric_array_text(text: &Option<String>, ty: &str) -> String {
    let ty_parse = |val: &str| -> String {
        match ty {
            "f64" => val.parse::<f64>().unwrap_or_default().to_string() + ty,
            "u32" => val.parse::<u32>().unwrap_or_default().to_string() + ty,
            "u8" => val.parse::<u8>().unwrap_or_default().to_string() + ty,
            _ => panic!("Not support the type {ty}"),
        }
    };
    text.as_ref()
        .map(|it| {
            it.trim()
                .split(',')
                .map(ty_parse)
                .collect::<Vec<_>>()
                .join(",")
        })
        .map(|it| format!("Some(&[{}])", it))
        .unwrap_or("None".to_string())
}
fn to_static_bool_array_text(text: &Option<String>) -> String {
    text.as_ref()
        .map(|it| {
            it.trim()
                .split(',')
                .map(|it| if it == "1" { "true" } else { "false" })
                .collect::<Vec<_>>()
                .join(",")
        })
        .map(|it| format!("Some(&[{}])", it))
        .unwrap_or("None".to_string())
}
fn to_static_tuple_array_text(name: &Option<String>, value: &Option<String>) -> String {
    let names = name.as_ref().map(|it| it.split(','));
    let values = value.as_ref().map(|it| it.split(','));
    names
        .and_then(|names| values.map(|values| names.zip(values)))
        .map(|it| {
            it.map(|(a, b)| format!("(\"{a}\", \"{b}\")"))
                .collect::<Vec<_>>()
                .join(",")
        })
        .map(|it| format!("&[{}]", it))
        .unwrap()
}
fn quote_text(text: &str) -> String {
    format!("\"{}\"", text)
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
