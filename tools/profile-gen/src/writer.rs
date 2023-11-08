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
    pub(crate) fn write_line<S>(&mut self, line: S)
    where
        S: ToString + std::fmt::Display,
    {
        let indent = " ".repeat(self.indent);
        self.file
            .write_all(format!("{indent}{}\n", line.to_string().trim()).as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write file"));
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
    pub(crate) fn write_use(&mut self, crates: Vec<&str>) {
        for _crate in crates {
            self.write_line(format!("use {_crate};"));
        }
    }
    pub(crate) fn write_comment(&mut self, comment: &str) {
        self.write_line(format!("/// {comment}"))
    }
    pub(crate) fn write_header(&mut self, comment: &str) {
        self.write_line(format!("//! {comment}"))
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
        self.write_line(format!("{name}{value},"))
    }
    pub(crate) fn write_struct_member(&mut self, name: &str, value: &str, visibility: Visibility) {
        self.write_line(format!("{visibility} {name}: {value},"))
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
    pub(crate) fn scope<S>(&mut self, scope: S)
    where
        S: FnOnce(&mut Self),
    {
        self.increase_indentation();
        scope(self);
        self.decrease_indention();
    }
    pub(crate) fn increase_indentation(&mut self) {
        self.indent += 4;
    }
    pub(crate) fn decrease_indention(&mut self) {
        self.indent -= 4;
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
        "allow(dead_code)",
        "allow(clippy::unreadable_literal)",
    ]);
    writer.write_use(vec!["std::fmt", "crate::fit"]);
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
                for value in &t.values {
                    if let Some(comment) = &value.comment {
                        writer.write_comment(comment);
                    }
                    writer.write_enum_member(&to_pascal_case(&value.name), None)
                }
            },
            Visibility::Crate,
        );
        if t.values.is_empty() {
            continue;
        }
        writer.write_impl(&enum_name, None, |writer| {
            writer.write_fn(
                "value",
                vec!["&self"],
                Some("fit::Value"),
                |writer| {
                    writer.write_block("match self", |writer| {
                        for value in &t.values {
                            writer.write_line(format!(
                                "{enum_name}::{} => fit::Value::{base_type}({}),",
                                to_pascal_case(&value.name),
                                value.value
                            ))
                        }
                    });
                },
                Visibility::Crate,
            );
        });
        writer.write_impl(&enum_name, Some("fmt::Display"), |writer| {
            writer.write_fn(
                "fmt",
                vec!["&self", "f: &mut fmt::Formatter<'_>"],
                Some("fmt::Result"),
                |writer| {
                    writer.write_block("match self", |writer| {
                        for value in &t.values {
                            writer.write_line(format!(
                                "{enum_name}::{} => f.write_str(\"{}\"),",
                                to_pascal_case(&value.name),
                                value.name
                            ))
                        }
                    })
                },
                Visibility::Private,
            );
        });
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
                            ))
                        }
                        writer.write_line(format!(
                            "_ => Err(\"No corresponding {enum_name} exists\"),",
                        ))
                    })
                },
                Visibility::Private,
            );
        });
        writer.write_impl(&enum_name, Some("TryFrom<fit::Value>"), |writer| {
            writer.write_type("Error", "&'static str", Visibility::Private);
            writer.write_fn(
                "try_from",
                vec!["value: fit::Value"],
                Some("Result<Self, Self::Error>"),
                |writer| {
                    writer.write_block("match value", |writer| {
                        for value in &t.values {
                            writer.write_line(format!(
                                "fit::Value::{base_type}({}) => Ok({enum_name}::{}),",
                                value.value,
                                to_pascal_case(&value.name)
                            ))
                        }
                        writer.write_line(format!(
                            "_ => Err(\"No corresponding {enum_name} exists\"),",
                        ))
                    })
                },
                Visibility::Private,
            );
        });
        writer.write_newline();
    }
    writer.fmt();
}

pub(crate) fn process_messages(messages: &crate::parser::Messages) {
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
        "allow(dead_code)",
        "allow(clippy::unreadable_literal)",
    ]);
    writer.write_use(vec![
        "lazy_static::lazy_static",
        "std::collections::HashMap",
    ]);
    writer.write_newline();

    writer.write_outer_attribute(vec!["derive(Debug, Clone)"]);
    writer.write_struct(
        "SubField",
        |writer| {
            writer.write_struct_member("field_name", "&'static str", Visibility::Crate);
            writer.write_struct_member("field_type", "&'static str", Visibility::Crate);
            writer.write_struct_member("array", "Option<usize>", Visibility::Crate);
            writer.write_struct_member(
                "components",
                "Option<&'static [&'static str]>",
                Visibility::Crate,
            );
            writer.write_struct_member("scale", "Option<&'static [f64]>", Visibility::Crate);
            writer.write_struct_member("offset", "Option<&'static [f64]>", Visibility::Crate);
            writer.write_struct_member(
                "units",
                "Option<&'static [&'static str]>",
                Visibility::Crate,
            );
            writer.write_struct_member("bits", "Option<&'static [u32]>", Visibility::Crate);
            writer.write_struct_member(
                "map",
                "&'static [(&'static str, &'static str)]",
                Visibility::Crate,
            );
        },
        Visibility::Crate,
    );

    writer.write_outer_attribute(vec!["derive(Debug, Clone)"]);
    writer.write_struct(
        "Field",
        |writer| {
            writer.write_struct_member("field_name", "&'static str", Visibility::Crate);
            writer.write_struct_member("field_type", "&'static str", Visibility::Crate);
            writer.write_comment("0 as N");
            writer.write_struct_member("array", "Option<usize>", Visibility::Crate);
            writer.write_struct_member(
                "components",
                "Option<&'static [&'static str]>",
                Visibility::Crate,
            );
            writer.write_comment("Field value is divided by scale (a float qty, 1=1.0)");
            writer.write_struct_member("scale", "Option<&'static [f64]>", Visibility::Crate);
            writer.write_comment("Field value has offset subtracted (a float qty, 0=0.0)");
            writer.write_struct_member("offset", "Option<&'static [f64]>", Visibility::Crate);
            writer.write_struct_member(
                "units",
                "Option<&'static [&'static str]>",
                Visibility::Crate,
            );
            writer.write_comment(
                "Only used with components, number of bits to extract for the field.",
            );
            writer.write_struct_member("bits", "Option<&'static [u32]>", Visibility::Crate);
            writer.write_struct_member("accumulate", "Option<&'static [bool]>", Visibility::Crate);
            writer.write_struct_member(
                "sub_fields",
                "Option<&'static [SubField]>",
                Visibility::Crate,
            );
        },
        Visibility::Crate,
    );

    writer.write_type("MessageMap", "HashMap<u8, Field>", Visibility::Crate);

    for message in messages {
        writer.write_block("lazy_static!", |writer| {
            if let Some(comment) = &message.comment {
                writer.write_comment(comment);
            }
            writer.write_line(&format!(
                "pub(crate) static ref {}: MessageMap = {{",
                message.message_name.to_uppercase()
            ));
            writer.increase_indentation();

            writer.write_line("let mut map: MessageMap = HashMap::new();");
            for field in &message.fields {
                writer.write_line("map.insert(");
                writer.increase_indentation();
                writer.write_line(format!("{},", field.field_def_number));
                writer.write_block("Field", |writer| {
                    writer.write_struct_member(
                        "field_name",
                        &quote_text(&field.field_name),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "field_type",
                        &quote_text(&field.field_type),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "array",
                        &to_option_usize_text(&field.array),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "components",
                        &to_static_array_text(&field.components),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "scale",
                        &to_static_numeric_array_text(&field.scale, "f64"),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "offset",
                        &to_static_numeric_array_text(&field.offset, "f64"),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "units",
                        &to_static_array_text(&field.units),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "bits",
                        &to_static_numeric_array_text(&field.bits, "u32"),
                        Visibility::Private,
                    );
                    writer.write_struct_member(
                        "accumulate",
                        &to_static_bool_array_text(&field.accumulate),
                        Visibility::Private,
                    );
                    if field.sub_fields.is_empty() {
                        writer.write_struct_member("sub_fields", "None", Visibility::Private);
                        return;
                    }
                    writer.write_line("sub_fields: Some(&[");
                    writer.increase_indentation();
                    for sub_field in &field.sub_fields {
                        writer.write_line("SubField {");
                        writer.increase_indentation();
                        writer.write_struct_member(
                            "field_name",
                            &quote_text(&sub_field.field_name),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "field_type",
                            &quote_text(&sub_field.field_type),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "array",
                            &to_option_usize_text(&sub_field.array),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "components",
                            &to_static_array_text(&sub_field.components),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "scale",
                            &to_static_numeric_array_text(&sub_field.scale, "f64"),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "offset",
                            &to_static_numeric_array_text(&sub_field.offset, "f64"),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "units",
                            &to_static_array_text(&sub_field.units),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "bits",
                            &to_static_numeric_array_text(&sub_field.bits, "u32"),
                            Visibility::Private,
                        );
                        writer.write_struct_member(
                            "map",
                            &to_static_tuple_array_text(
                                &sub_field.ref_field_name,
                                &sub_field.ref_field_value,
                            ),
                            Visibility::Private,
                        );
                        writer.decrease_indention();
                        writer.write_line("},");
                    }
                    writer.decrease_indention();
                    writer.write_line("]),");
                });
                writer.decrease_indention();
                writer.write_line(");");
            }
            writer.write_line("map");
            writer.decrease_indention();
            writer.write_line("};");
        });
    }

    writer.write_fn(
        "from_str",
        vec!["s: &str"],
        Some("Option<&MessageMap>"),
        |writer| {
            writer.write_block("match s", |writer| {
                for message in messages {
                    writer.write_line(format!(
                        "\"{}\" => Some(&self::{}),",
                        message.message_name,
                        message.message_name.to_uppercase()
                    ));
                }
                writer.write_line("_ => None");
            })
        },
        Visibility::Crate,
    )
}

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
fn to_option_usize_text(text: &Option<String>) -> String {
    text.as_ref()
        .and_then(|it| {
            let text = it.trim();
            if !(text.starts_with("[") && text.ends_with("]")) {
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
                .map(|it| ty_parse(it))
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
