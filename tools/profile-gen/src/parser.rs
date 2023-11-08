use calamine as xlsx;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct TypeValue {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) comment: Option<String>,
}
#[derive(Debug, Clone)]
pub(crate) struct Type {
    pub(crate) type_name: String,
    pub(crate) base_type: String,
    pub(crate) values: Vec<TypeValue>,
}

#[derive(Debug, Clone)]
pub(crate) struct SubMessageField {
    pub(crate) field_name: String,
    pub(crate) field_type: String,
    pub(crate) array: Option<String>,
    pub(crate) components: Option<String>,
    pub(crate) scale: Option<String>,
    pub(crate) offset: Option<String>,
    pub(crate) units: Option<String>,
    pub(crate) bits: Option<String>,
    pub(crate) accumulate: Option<String>,
    pub(crate) ref_field_name: Option<String>,
    pub(crate) ref_field_value: Option<String>,
    pub(crate) comment: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct MessageField {
    pub(crate) field_def_number: u8,
    pub(crate) field_name: String,
    pub(crate) field_type: String,
    pub(crate) array: Option<String>,
    pub(crate) components: Option<String>,
    pub(crate) scale: Option<String>,
    pub(crate) offset: Option<String>,
    pub(crate) units: Option<String>,
    pub(crate) bits: Option<String>,
    pub(crate) accumulate: Option<String>,
    pub(crate) comment: Option<String>,
    pub(crate) sub_fields: Vec<SubMessageField>,
}

#[derive(Debug, Clone)]
pub(crate) struct Message {
    pub(crate) message_name: String,
    pub(crate) comment: Option<String>,
    pub(crate) fields: Vec<MessageField>,
}
pub(crate) type Messages = Vec<Message>;
pub(crate) type Types = Vec<Type>;

pub(crate) struct Profile {
    pub(crate) messages: Messages,
    pub(crate) types: Types,
}

type Sheet = xlsx::Range<xlsx::DataType>;

fn process_types(sheet: &Sheet) -> Types {
    let mut types: Types = Vec::new();
    // skip header
    for row in sheet.rows().skip(1) {
        if !row[0].is_empty() {
            let type_name = row[0].to_string();
            types.push(Type {
                type_name: type_name.clone(),
                base_type: row[1].to_string(),
                values: Vec::new(),
            });
            continue;
        }
        if let Some(t) = types.last_mut() {
            if row[2].is_empty() {
                panic!("\"Value Name\" cannot be null")
            }
            t.values.push(TypeValue {
                name: row[2].to_string(),
                value: row[3].to_string(),
                comment: row[4].as_string(),
            })
        } else {
            panic!("Invalid rows {:#?}", row);
        }
    }
    types
}
fn process_messages(sheet: &Sheet) -> Messages {
    let mut messages: Messages = Vec::new();
    for row in sheet.rows().skip(1) {
        if !row[0].is_empty() {
            let message_name = row[0].to_string();
            messages.push(Message {
                message_name: message_name.clone(),
                comment: row[13].as_string(),
                fields: Vec::new(),
            });
            continue;
        }
        if let Some(current_message) = messages.last_mut() {
            // It should be a category split line.
            if row[2].is_empty() {
                // panic!("\"Field Name\" cannot be null \n{:?}", row);
                continue;
            }
            if row[1].is_empty() {
                current_message
                    .fields
                    .last_mut()
                    .unwrap_or_else(|| panic!("\"Field Def #\" column cannot be null \n{:?}", row))
                    .sub_fields
                    .push(SubMessageField {
                        field_name: row[2].to_string(),
                        field_type: row[3].to_string(),
                        array: row[4].as_string(),
                        components: row[5]
                            .as_string(),
                        scale: row[6].as_string(),
                        offset: row[7].as_string(),
                        units: row[8].as_string(),
                        bits: row[9].as_string(),
                        accumulate: row[10].as_string(),
                        ref_field_name: row[11]
                            .as_string(),
                        ref_field_value: row[12]
                            .as_string(),
                        comment: row[13].as_string(),
                    });
                continue;
            }
            let field_def_number = u8::from_str(&row[1].to_string()).unwrap();
            current_message.fields.push(MessageField {
                field_def_number,
                field_name: row[2].to_string(),
                field_type: row[3].to_string(),
                array: row[4].as_string(),
                components: row[5].as_string(),
                scale: row[6].as_string(),
                offset: row[7].as_string(),
                units: row[8].as_string(),
                bits: row[9].as_string(),
                accumulate: row[10].as_string(),
                comment: row[13].as_string(),
                sub_fields: Vec::new(),
            });
        } else {
            panic!("Invalid rows {:#?}", row);
        }
    }
    messages
}

pub(crate) fn process_profile(path: &PathBuf) -> Profile {
    use xlsx::Reader;
    let mut excel: xlsx::Xlsx<_> = xlsx::open_workbook(path).unwrap();
    let types = if let Some(Ok(sheet)) = excel.worksheet_range("Types") {
        process_types(&sheet)
    } else {
        panic!("Could not access workbook sheet 'Types'");
    };
    let messages = if let Some(Ok(sheet)) = excel.worksheet_range("Messages") {
        process_messages(&sheet)
    } else {
        panic!("Could not access workbook sheet 'Messages'");
    };
    Profile { types, messages }
}
