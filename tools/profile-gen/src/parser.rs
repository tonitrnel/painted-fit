use calamine as xlsx;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

macro_rules! split_csv_string ( ($value:expr) => ( {$value.split(',').map(|v| v.trim().to_string())} ););

#[derive(Debug, Clone)]
pub struct TypeValue {
    pub name: String,
    pub value: String,
    pub comment: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Type {
    pub type_name: String,
    pub base_type: String,
    pub values: Vec<TypeValue>,
}
#[derive(Debug, Clone)]
pub struct NestedMessageField {
    pub field_name: String,
    pub field_type: String,
    pub array: Option<usize>,
    pub components: Vec<(u8, MessageField)>,
    pub scale: f64,
    pub offset: f64,
    pub units: String,
    pub ref_field_name: String,
    pub ref_field_value: String,
    pub comment: Option<String>,
    _raw_components: Vec<MessageComponent>,
}
#[derive(Debug, Clone)]
pub struct MessageField {
    pub field_no: u8,
    pub field_name: String,
    pub field_type: String,
    pub array: Option<usize>,
    pub components: Vec<(u8, MessageField)>,
    pub scale: f64,
    pub offset: f64,
    pub units: String,
    pub accumulate: bool,
    pub comment: Option<String>,
    pub sub_fields: Vec<NestedMessageField>,
    _raw_components: Vec<MessageComponent>,
}

#[derive(Debug, Clone)]
struct MessageComponent {
    name: String,
    scale: f64,
    offset: f64,
    units: String,
    bits: u8,
    accumulate: bool,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    pub comment: Option<String>,
    pub fields: Vec<MessageField>,
}
pub type Messages = Vec<Message>;
pub type Types = Vec<Type>;

pub struct FitProfile {
    pub messages: Messages,
    pub types: Types,
}

type Sheet = xlsx::Range<xlsx::DataType>;
type Row<'a> = &'a [xlsx::DataType];

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
fn parse_array_size(str: String) -> Option<usize> {
    let mut str = str.trim();
    if !(str.starts_with('[') && str.ends_with(']')) {
        return None;
    }
    str = str.trim_start_matches('[').trim_end_matches(']');
    if str == "N" {
        Some(0usize)
    } else {
        str.parse::<usize>().ok()
    }
}
fn parse_components(row: Row) -> Vec<MessageComponent> {
    let mut components = Vec::new();
    let names = match row[5].get_string() {
        Some(str) => split_csv_string!(str).collect::<Vec<String>>(),
        _ => return components,
    };
    let cols = row[6..=10]
        .iter()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();
    let mut scales = split_csv_string!(cols[0]).map(|s| s.parse::<f64>().ok());
    let mut offsets = split_csv_string!(cols[1]).map(|s| s.parse::<f64>().ok());
    let mut units = split_csv_string!(cols[2]);
    let mut bits = split_csv_string!(cols[3]).map(|s| s.parse::<u8>().ok());
    let mut accumulates = split_csv_string!(cols[4]).map(|s| s == "1");
    for name in names {
        components.push(MessageComponent {
            name,
            scale: scales.next().flatten().unwrap_or(1.0),
            offset: offsets.next().flatten().unwrap_or(0.0),
            units: units.next().unwrap_or_default(),
            bits: bits
                .next()
                .flatten()
                .unwrap_or_else(|| panic!("Invalid 'bit' field, row={row:?} column=9")),
            accumulate: accumulates.next().unwrap_or(false),
        })
    }
    components
}
fn process_messages(sheet: &Sheet) -> Messages {
    let mut messages: Messages = Vec::new();
    // collect all messages and its fields.
    for row in sheet.rows().skip(1) {
        // message row
        if !row[0].is_empty() {
            messages.push(Message {
                name: row[0].to_string(),
                comment: row[13].as_string(),
                fields: Vec::new(),
            });
            continue;
        }
        // message field row
        if let Some(current_message) = messages.last_mut() {
            // It should be a category split line.
            if row[2].is_empty() {
                // panic!("\"Field Name\" cannot be null \n{:?}", row);
                continue;
            }
            if !row[1].is_empty() {
                let field_no = u8::from_str(&row[1].to_string()).unwrap();
                current_message.fields.push(MessageField {
                    field_no,
                    field_name: row[2].to_string(),
                    field_type: row[3].to_string(),
                    array: row[4].as_string().and_then(parse_array_size),
                    scale: row[6].get_float().unwrap_or(1.0),
                    offset: row[7].get_float().unwrap_or(0.0),
                    units: row[8].to_string(),
                    accumulate: false,
                    comment: row[13].as_string(),
                    components: Vec::new(),
                    _raw_components: parse_components(row),
                    sub_fields: Vec::new(),
                });
            } else {
                // sub field row
                let parent = current_message
                    .fields
                    .last_mut()
                    .unwrap_or_else(|| panic!("Invalid sub field row={row:?}"));
                let ref_field_names = row[11].get_string().unwrap_or_else(|| {
                    panic!("Missing reference field name(s), row={row:?} column=11")
                });
                let ref_field_values = row[12].get_string().unwrap_or_else(|| {
                    panic!("Missing reference field value(s), row={row:?} column=12")
                });
                for (name, value) in
                    split_csv_string!(ref_field_names).zip(split_csv_string!(ref_field_values))
                {
                    parent.sub_fields.push(NestedMessageField {
                        field_name: row[2].to_string(),
                        field_type: row[3].to_string(),
                        array: row[4].as_string().and_then(parse_array_size),
                        scale: row[6].get_float().unwrap_or(1.0),
                        offset: row[7].get_float().unwrap_or(0.0),
                        units: row[8].to_string(),
                        ref_field_name: name,
                        ref_field_value: value,
                        comment: row[13].as_string(),
                        components: Vec::new(),
                        _raw_components: parse_components(row),
                    });
                }
            }
        } else {
            panic!("Invalid rows {:#?}", row);
        }
    }
    // process field components
    messages.into_iter().map(process_message).collect()
}
fn process_message(message: Message) -> Message {
    let Message {
        name,
        comment,
        mut fields,
    } = message;
    let field_map = fields
        .iter()
        .map(|it| (it.field_name.to_owned(), it.to_owned()))
        .collect::<HashMap<_, _>>();
    let mut accumulates: HashSet<String> = HashSet::new();
    for field in &mut fields {
        if !field._raw_components.is_empty() {
            field.components =
                process_components(&field._raw_components, &field_map, &mut accumulates);
            field._raw_components = Vec::new()
        }
        for sub_field in &mut field.sub_fields {
            if !sub_field._raw_components.is_empty() {
                sub_field.components =
                    process_components(&sub_field._raw_components, &field_map, &mut accumulates);
                sub_field._raw_components = Vec::new()
            }
        }
    }
    Message {
        name,
        comment,
        fields: fields
            .into_iter()
            .map(|mut it| {
                it.accumulate = accumulates.contains(&it.field_name);
                it
            })
            .collect(),
    }
}
fn process_components(
    raw_components: &[MessageComponent],
    field_map: &HashMap<String, MessageField>,
    accumulates: &mut HashSet<String>,
) -> Vec<(u8, MessageField)> {
    let mut components = Vec::new();
    for component in raw_components {
        let dest = field_map.get(&component.name).unwrap_or_else(|| {
            panic!(
                "Cannot find '{}' field in '{}' message",
                component.name, component.name
            )
        });
        if component.accumulate {
            accumulates.insert(component.name.to_owned());
        }
        components.push((
            component.bits,
            MessageField {
                field_no: dest.field_no,
                field_name: dest.field_name.to_owned(),
                field_type: dest.field_type.to_owned(),
                array: dest.array.to_owned(),
                scale: component.scale,
                offset: component.offset,
                units: component.units.to_owned(),
                accumulate: component.accumulate,
                sub_fields: dest.sub_fields.to_owned(),
                components: process_components(&dest._raw_components, field_map, accumulates),
                _raw_components: Vec::new(),
                comment: dest.comment.to_owned(),
            },
        ));
    }
    components
}
pub fn process_profile(bytes: &[u8]) -> FitProfile {
    use xlsx::Reader;
    let cursor = std::io::Cursor::new(bytes);
    let mut excel = xlsx::open_workbook_auto_from_rs(cursor).unwrap();
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
    FitProfile { types, messages }
}
