use std::fmt::Formatter;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Writer<'a> {
    path: &'a PathBuf,
    file: File,
    indent: usize,
}
#[allow(unused)]
pub enum Visibility {
    Public,
    Crate,
    Super,
    Private,
}
impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => f.write_str("pub"),
            Visibility::Crate => f.write_str("pub"),
            Visibility::Super => f.write_str("pub(super)"),
            Visibility::Private => f.write_str(""),
        }
    }
}
impl<'a> Writer<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
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
    pub fn write_line<S>(&mut self, line: S) -> &mut Writer<'a>
    where
        S: ToString + std::fmt::Display,
    {
        let indent = " ".repeat(self.indent);
        self.file
            .write_all(format!("{indent}{}\n", line.to_string().trim()).as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write file"));
        self
    }
    pub fn write_newline(&mut self) {
        self.file
            .write_all(&[0x0Au8])
            .unwrap_or_else(|_| panic!("Failed to write file"));
    }
    pub fn write_inner_attribute(&mut self, attributes: Vec<&str>) {
        for attribute in attributes {
            self.write_line(format!("#![{attribute}]"));
        }
    }
    pub fn write_outer_attribute(&mut self, attributes: Vec<&str>) {
        for attribute in attributes {
            self.write_line(format!("#[{attribute}]"));
        }
    }
    pub fn write_import_packages(&mut self, crates: Vec<&str>) {
        for _crate in crates {
            self.write_line(format!("use {_crate};"));
        }
    }
    pub fn write_comment(&mut self, comment: &str) {
        self.write_line(format!("/// {comment}"));
    }
    pub fn write_header(&mut self, comment: &str) {
        self.write_line(format!("// {comment}"));
    }
    pub fn write_enum<S>(&mut self, name: &str, scope: S, visibility: Visibility)
    where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{visibility} enum {name}{{"));

        self.scope(scope);
        self.write_line("}");
    }
    #[allow(unused)]
    pub fn write_struct<S>(&mut self, name: &str, scope: S, visibility: Visibility)
    where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{visibility} struct {name}{{"));
        self.scope(scope);
        self.write_line("}");
        self.write_newline();
    }
    pub fn write_enum_member(&mut self, name: &str, value: Option<String>) {
        let value = if let Some(value) = value {
            format!(" => {value}")
        } else {
            String::new()
        };
        self.write_line(format!("{name}{value},"));
    }
    pub fn write_struct_member(&mut self, name: &str, value: &str, visibility: Visibility) {
        self.write_line(format!("{visibility} {name}: {value},"));
    }
    pub fn write_impl<S>(&mut self, type_name: &str, trait_name: Option<&str>, scope: S)
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
    pub fn write_code_fragment<S>(&mut self, part: S)
    where
        S: ToString,
    {
        let part = part.to_string();
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
                    .skip_while(|(pos, char)| *pos < indent && char.is_whitespace())
                    .map(|(_, char)| char)
                    .collect::<String>(),
            );
        }
    }
    pub fn scope<S>(&mut self, scope: S)
    where
        S: FnOnce(&mut Self),
    {
        self.increase_indentation();
        scope(self);
        self.decrease_indention();
    }
    pub fn increase_indentation(&mut self) -> &mut Writer<'a> {
        self.indent += 4;
        self
    }
    pub fn decrease_indention(&mut self) -> &mut Writer<'a> {
        self.indent -= 4;
        self
    }
    pub fn write_type(&mut self, name: &str, value: &str, visibility: Visibility) {
        self.write_line(format!("{visibility} type {name} = {value};"));
    }
    pub fn write_fn<S>(
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
    pub fn write_block<S>(&mut self, statement: &str, scope: S)
    where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{statement} {{"));
        self.scope(scope);
        self.write_line("}");
    }
    pub fn write_call<S>(&mut self, func: &str, parameters: Vec<S>)
    where
        S: ToString + std::fmt::Display,
    {
        self.write_line(format!("{func}(")).increase_indentation();
        for parameter in parameters {
            self.write_code_fragment(&format!("{parameter},"));
        }
        self.decrease_indention().write_line(");");
    }
    pub fn write_block_with_end_symbol<S>(&mut self, statement: &str, scope: S, end_symbol: &str)
    where
        S: FnOnce(&mut Self),
    {
        self.write_line(format!("{statement} {{"));
        self.scope(scope);
        self.write_line(format!("}}{}", end_symbol));
    }
    pub fn fmt(&self) {
        let path = self.path;
        std::process::Command::new("rustfmt")
            .arg(path)
            .status()
            .unwrap_or_else(|_| panic!("failed to execute rustfmt on {path:?}"));
    }
}
