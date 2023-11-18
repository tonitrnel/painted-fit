use crate::byte_reader::ByteReader;
use crate::crc;
use crate::error::{ErrorKind, ParserResult};
use crate::fit;
use std::collections::HashMap;
use std::sync::Arc;

macro_rules! fit_value_covert {
    ($value: expr, $variant: ident) => {
        $value
            .and_then(|it| {
                if let fit::Value::$variant(val) = it {
                    Some(*val)
                } else {
                    None
                }
            })
            .unwrap()
    };
}

#[derive(Debug, Clone)]
#[allow(unused)]
struct FitFileHeader {
    pub header_size: u32,
    pub protocol_version: u8,
    pub profile_version: u16,
    pub data_size: u32,
    pub data_type: String,
    pub header_crc: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Architecture {
    LittleEndian,
    BigEndian,
}
impl Architecture {
    fn is_big_endian(&self) -> bool {
        matches!(self, Architecture::BigEndian)
    }
}
impl From<u8> for Architecture {
    fn from(value: u8) -> Self {
        if value == 0x01 {
            Architecture::BigEndian
        } else {
            Architecture::LittleEndian
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FitMessageType {
    Data,
    Definition,
}
#[derive(Debug, Clone)]
struct FitMessageHeader {
    contains_developer_data: bool,
    local_message_number: u8,
    message_type: FitMessageType,
    time_offset: Option<u8>,
}

enum FitMessage {
    Data(FitDataMessage),
    Definition(FitDefinitionMessage),
}
#[derive(Debug, Clone)]
struct FieldDefinition {
    field_definition_number: u8,
    size: u8,
    base_type: fit::BaseType,
}
#[derive(Debug, Clone)]
#[allow(unused)]
struct DeveloperFieldDefinition {
    field_number: u8,
    size: u8,
    developer_data_index: u8,
}

#[derive(Debug, Clone)]
struct FitDefinitionMessage {
    architecture: Architecture,
    local_message_number: u8,
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
    developer_field_definitions: Vec<DeveloperFieldDefinition>,
}

#[derive(Debug, Clone)]
struct FitDataMessage {
    global_message_number: u16,
    time_offset: Option<u8>,
    // HashMap<field_definition_number, Value>
    fields: HashMap<u8, fit::Value>,
    // HashMap<developer_data_index, HashMap<field_definition_number, Value>>
    developer_fields: HashMap<u8, HashMap<u8, fit::Value>>,
}

#[derive(Debug, Clone)]
#[allow(unused)]
struct FitDeveloperDataDefinition {
    developer_id: Option<fit::Value>,
    application_id: Option<fit::Value>,
    manufacturer_id: Option<fit::Value>,
    developer_data_index: u8,
    application_version: u32,
    field_map: HashMap<u8, HashMap<&'static str, fit::Value>>,
}

const CRC_SIZE: u32 = 2;

/// Decode fit file
pub struct Decoder<'input> {
    reader: ByteReader<'input>,
    defs: HashMap<u8, Arc<FitDefinitionMessage>>,
    dev_data_defs: HashMap<u8, FitDeveloperDataDefinition>,
    dev_data: HashMap<String, HashMap<u8, fit::Value>>,
    timestamp_ref: Option<u32>,
    errors: Vec<ErrorKind>,
}

pub type Record = HashMap<&'static str, fit::Value>;
pub type Messages = HashMap<String, Vec<Record>>;

impl<'input> Decoder<'input> {
    pub fn new(bytes: &'input [u8]) -> Self {
        Decoder {
            reader: bytes.into(),
            defs: HashMap::new(),
            dev_data_defs: HashMap::new(),
            dev_data: HashMap::new(),
            errors: Vec::new(),
            timestamp_ref: None,
        }
    }
    fn read_file_header(&mut self) -> FitFileHeader {
        let header_size = self.reader.read_next_u8();
        let protocol_version = self.reader.read_next_u8();
        let profile_version = self.reader.read_next_u16(false);
        let data_size = self.reader.read_next_u32(false);
        let data_type = self.reader.read_next_uft8_string(4);
        let crc = if header_size == 0x0E {
            self.reader.read_next_u16(false)
        } else {
            0
        };
        FitFileHeader {
            header_size: header_size as u32,
            protocol_version,
            profile_version,
            data_size,
            data_type,
            header_crc: crc,
        }
    }
    /// 检查头以确定是否是 FIT 文件
    pub fn is_fit(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }
        if bytes[0] != 0x0E && bytes[0] != 0x0C {
            return false;
        }
        if bytes.len() < ((bytes[0] as u32) + CRC_SIZE) as usize {
            return false;
        }
        if String::from_utf8_lossy(&bytes[8..12]) != ".FIT" {
            return false;
        }
        true
    }
    /// 检查是否为 FIT 文件并验证 Header 和 CRC
    pub fn check_integrity(&mut self) -> bool {
        let header = self.read_file_header();
        if self.reader.len() < (header.header_size + header.data_size + CRC_SIZE) as usize {
            return false;
        }
        if header.header_size == 0xE
            && header.header_crc != 0x0000
            && header.header_crc != crc::crc_16(&self.reader[0..12])
        {
            return false;
        }
        let file_crc = ((self.reader[(header.header_size + header.data_size + 1) as usize] as u16)
            << 8)
            + self.reader[(header.header_size + header.data_size) as usize] as u16;
        if file_crc
            != crc::crc_16(&self.reader[0..(header.header_size + header.data_size) as usize])
        {
            return false;
        }
        true
    }
    fn crc_valid(&mut self, start: usize, end: usize) -> bool {
        self.reader.read_next_u16(false) == crc::crc_16(&self.reader[start..end])
    }
    /// 阅读信息
    pub fn decode(&mut self) -> ParserResult<(Vec<ErrorKind>, Messages)> {
        self.reader.reset();
        let mut messages: Messages = HashMap::new();
        while !self.reader.is_end() {
            self.decode_next_file(&mut messages)?;
        }
        Ok((self.errors.to_owned(), messages))
    }
    fn decode_next_file(&mut self, messages: &mut Messages) -> ParserResult<()> {
        let start = self.reader.offset();
        if !Decoder::is_fit(&self.reader[start..]) {
            return Err(ErrorKind::InvalidFitFile);
        };
        let header = self.read_file_header();
        let end = start + header.header_size as usize + header.data_size as usize;
        while self.reader.offset() < end {
            match self.decode_next_record() {
                Ok(Some((k, v))) => {
                    messages.entry(k).or_default().push(v);
                }
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }
        if !self.crc_valid(start, end) {
            return Err(ErrorKind::InvalidCRC);
        }
        Ok(())
    }

    fn decode_next_record(&mut self) -> ParserResult<Option<(String, Record)>> {
        let message = self.read_message()?;
        match message {
            FitMessage::Definition(message) => {
                self.defs
                    .insert(message.local_message_number, Arc::new(message));
                Ok(None)
            }
            FitMessage::Data(message) => match self.decode_message(message) {
                Ok((k, v)) => Ok(Some((k, v))),
                Err(e) => {
                    self.errors.push(e);
                    Ok(None)
                }
            },
        }
    }

    /// read message header
    ///
    /// ## structure
    ///
    /// normal:
    /// - bit 7: normal header(value: 0)
    /// - bit 6: message type(0: data message, 1: definition message)
    /// - bit 5: message type specific
    /// - bit 4: reserved
    /// - bit 3..0: local message type
    ///
    /// compressed timestamp header
    /// - bit 7: compressed timestamp(value: 1)
    /// - bit 6..5: local message type
    /// - bit 4..0: time offset
    fn read_message_header(&mut self) -> ParserResult<FitMessageHeader> {
        let byte = self.reader.read_next_u8();
        if byte & 0x80 == 0x80 {
            // compressed timestamp header
            Ok(FitMessageHeader {
                contains_developer_data: false,
                local_message_number: (byte >> 5) & 0x03,
                message_type: FitMessageType::Data,
                time_offset: Some(byte & 0x1F),
            })
        } else if byte & 0x40 == 0x40 {
            // definition message
            Ok(FitMessageHeader {
                contains_developer_data: byte & 0x20 == 0x20,
                local_message_number: byte & 0x0F,
                message_type: FitMessageType::Definition,
                time_offset: None,
            })
        } else if byte & 0x40 == 0x00 {
            // data message
            Ok(FitMessageHeader {
                contains_developer_data: false,
                local_message_number: byte & 0x0F,
                message_type: FitMessageType::Data,
                time_offset: None,
            })
        } else {
            Err(ErrorKind::InvalidMessageHeader)
        }
    }
    fn read_message(&mut self) -> ParserResult<FitMessage> {
        let header = self.read_message_header()?;
        Ok(match &header.message_type {
            FitMessageType::Definition => {
                FitMessage::Definition(self.read_definition_message(&header)?)
            }
            FitMessageType::Data => FitMessage::Data(self.read_data_message(&header)?),
        })
    }
    fn read_data_message(&mut self, header: &FitMessageHeader) -> ParserResult<FitDataMessage> {
        let def = self
            .defs
            .get(&header.local_message_number)
            .ok_or(ErrorKind::LocalDefinitionMessageNotFound(
                header.local_message_number,
            ))?
            .clone();

        let mut fields = HashMap::new();
        for field_def in &def.field_definitions {
            match self.read_field_value(
                field_def.size as usize,
                field_def.base_type,
                def.architecture.is_big_endian(),
            ) {
                Ok(value) => {
                    fields.insert(field_def.field_definition_number, value);
                }
                // skip invalid field
                Err(e) => {
                    self.errors.push(ErrorKind::DecodeFieldFailed {
                        message_no: def.global_message_number,
                        field_no: field_def.field_definition_number,
                        reason: e.to_string(),
                    });
                }
            }
        }

        let mut developer_fields = HashMap::new();

        for field_def in &def.developer_field_definitions {
            let value = match self.read_field_value(
                field_def.size as usize,
                fit::BaseType::Byte,
                def.architecture.is_big_endian(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    self.errors.push(ErrorKind::DecodeDeveloperFieldFailed {
                        message_no: def.global_message_number,
                        data_index: field_def.developer_data_index,
                        reason: e.to_string(),
                    });
                    continue;
                }
            };
            developer_fields
                .entry(field_def.developer_data_index)
                .or_insert_with(HashMap::new)
                .insert(field_def.field_number, value);
        }
        Ok(FitDataMessage {
            fields,
            developer_fields,
            global_message_number: def.global_message_number,
            time_offset: header.time_offset,
        })
    }
    fn read_field_value(
        &mut self,
        size: usize,
        base_type: fit::BaseType,
        is_big_endian: bool,
    ) -> ParserResult<fit::Value> {
        use fit::{BaseType, Value};
        if size % base_type.size() as usize != 0 {
            return Err(ErrorKind::SizeMismatch {
                field_size: size,
                base_type_size: base_type.size(),
            });
        }
        let reader = &mut self.reader;
        let end = reader.offset() + size;
        let mut values = Vec::new();
        while reader.offset() < end {
            let value = match base_type {
                BaseType::Enum => Value::Enum(reader.read_next_u8()),
                BaseType::SInt8 => Value::SInt8(reader.read_next_i8()),
                BaseType::UInt8 => Value::UInt8(reader.read_next_u8()),
                BaseType::SInt16 => Value::SInt16(reader.read_next_i16(is_big_endian)),
                BaseType::UInt16 => Value::UInt16(reader.read_next_u16(is_big_endian)),
                BaseType::SInt32 => Value::SInt32(reader.read_next_i32(is_big_endian)),
                BaseType::UInt32 => Value::UInt32(reader.read_next_u32(is_big_endian)),
                BaseType::String => {
                    let bytes = reader.read_bytes(size);
                    let mut new_bytes = Vec::new();
                    for byte in bytes {
                        if byte == &0u8 {
                            break;
                        }
                        new_bytes.push(*byte)
                    }
                    Value::String(String::from_utf8(new_bytes).unwrap())
                }
                BaseType::Float32 => Value::Float32(reader.read_next_f32(is_big_endian)),
                BaseType::Float64 => Value::Float64(reader.read_next_f64(is_big_endian)),
                BaseType::UInt8z => Value::UInt8(reader.read_next_u8()),
                BaseType::UInt16z => Value::UInt16z(reader.read_next_u16(is_big_endian)),
                BaseType::UInt32z => Value::UInt32z(reader.read_next_u32(is_big_endian)),
                BaseType::Byte => Value::Byte(reader.read_next_u8()),
                BaseType::SInt64 => Value::SInt64(reader.read_next_i64(is_big_endian)),
                BaseType::UInt64 => Value::UInt64(reader.read_next_u64(is_big_endian)),
                BaseType::UInt64z => Value::UInt64z(reader.read_next_u64(is_big_endian)),
            };
            values.push(value);
        }
        let value = if values.len() == 1 {
            values.swap_remove(0)
        } else {
            Value::Array(values)
        };
        if !value.is_valid() {
            Err(ErrorKind::InvalidFieldValue {
                value,
                base_type,
                invalid: base_type.invalid(),
            })
        } else {
            Ok(value)
        }
    }
    /// Read definition message
    ///
    /// ## Structure
    ///
    /// ```text
    /// -------------------------------------------------------------------------
    /// | Reserved | Architecture | Global Msg No. | No. of Fields | Field Def* |
    /// -------------------------------------------------------------------------
    /// ```
    /// Expand if contains developer data included
    /// ```text
    /// -------------------------------------------------------
    /// | ... Field Def* | No. of Dev Fields | Dev Field Def* |
    /// -------------------------------------------------------
    /// ```
    ///
    /// "Field Def" structure
    ///
    /// ```text
    /// ------------------------------------
    /// | Field Def No. | Size | Base Type |
    /// ------------------------------------
    /// ```
    ///
    /// "Dev Field Def" structure
    ///
    /// ```text
    /// ----------------------------------------------
    /// | Field Number | Size | Developer Data Index |
    /// ----------------------------------------------
    /// ```
    ///
    /// - "Global Msg No.": `Profile.xlsx/Types/mesg_num`
    /// - "Field Def No.": `Profile.xlsx/Messages/[Global Msg No.]/`
    fn read_definition_message(
        &mut self,
        header: &FitMessageHeader,
    ) -> ParserResult<FitDefinitionMessage> {
        // Consume reserved byte
        self.reader.read_next_u8();
        let architecture = Architecture::from(self.reader.read_next_u8());
        let global_message_number = self.reader.read_next_u16(architecture.is_big_endian());
        let field_definitions = {
            let mut definitions = Vec::new();
            let number_of_fields = self.reader.read_next_u8();
            for _ in 0..number_of_fields {
                let field_definition_number = self.reader.read_next_u8();
                let size = self.reader.read_next_u8();
                let base_type =
                    fit::BaseType::try_from(self.reader.read_next_u8()).map_err(|err| {
                        ErrorKind::BaseTypeMismatch {
                            reason: err.to_owned(),
                        }
                    })?;
                definitions.push(FieldDefinition {
                    field_definition_number,
                    size,
                    base_type,
                });
            }
            definitions
        };
        let developer_field_definitions = if header.contains_developer_data {
            let mut definitions = Vec::new();
            let number_of_fields = self.reader.read_next_u8();
            for _ in 0..number_of_fields {
                let field_number = self.reader.read_next_u8();
                let size = self.reader.read_next_u8();
                let developer_data_index = self.reader.read_next_u8();
                definitions.push(DeveloperFieldDefinition {
                    field_number,
                    size,
                    developer_data_index,
                });
            }
            definitions
        } else {
            Vec::new()
        };

        Ok(FitDefinitionMessage {
            architecture,
            local_message_number: header.local_message_number,
            global_message_number,
            field_definitions,
            developer_field_definitions,
        })
    }
    fn decode_message(&mut self, message: FitDataMessage) -> ParserResult<(String, Record)> {
        use crate::profile::{messages, types};
        let mut accumulator = crate::accumulator::Accumulator::default();
        let msg_ty = types::MesgNum::try_from(&fit::Value::UInt16(message.global_message_number))
            .map_err(|_| {
            ErrorKind::GlobalDefinitionMessageNotFound(message.global_message_number)
        })?;
        let decode = messages::from_message_type(&msg_ty.to_string())
            .ok_or(ErrorKind::UnknownMessage(msg_ty.to_string()))?;
        let mut message_map = HashMap::new();
        for (field_def_number, val) in message.fields.iter() {
            if let Err(e) = decode(
                &mut message_map,
                &mut accumulator,
                messages::MessageDecodeArgs {
                    msg_ty: &msg_ty,
                    msg_no: message.global_message_number,
                    field_no: *field_def_number,
                    value: val,
                    fields: &message.fields,
                },
            )
            .map_err(|err| ErrorKind::DecodeMessageFailed {
                message: msg_ty.to_string(),
                field_no: *field_def_number,
                reason: err.to_owned(),
            }) {
                self.errors.push(e)
            };

            // common timestamp field, used in combination with the compressed timestamp
            if field_def_number == &253 {
                self.timestamp_ref = Some(val.try_as_usize().unwrap() as u32)
            }
        }

        for (developer_data_index, fields) in message.developer_fields {
            let developer_data_def = if let Some(r) = self.dev_data_defs.get(&developer_data_index)
            {
                r
            } else {
                self.errors
                    .push(ErrorKind::MissingDeveloperDataDef(developer_data_index));
                continue;
            };
            for (field_def_number, val) in fields {
                let val = if let fit::Value::Byte(r) = val { r } else { 0 };
                let field_def = if let Some(r) = developer_data_def.field_map.get(&field_def_number)
                {
                    r
                } else {
                    self.errors
                        .push(ErrorKind::MissingDeveloperFieldDescription(
                            developer_data_index,
                            field_def_number,
                        ));
                    continue;
                };
                let base_type = field_def
                    .get("fit_base_type_id")
                    .map(|it| it.to_string())
                    .unwrap();
                let value = match base_type.as_str() {
                    "uint8" => fit::Value::UInt8(val),
                    "float32" => fit::Value::Float32(val as f32),
                    _ => fit::Value::Byte(val),
                };
                self.dev_data
                    .entry(msg_ty.to_string())
                    .or_default()
                    .insert(field_def_number, value);
            }
        }
        if message.time_offset.is_some() {
            let result = self
                .update_time_offset(message.time_offset.unwrap())
                .and_then(|timestamp| {
                    chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0)
                        .ok_or(ErrorKind::InvalidTimestamp(timestamp))
                        .map(fit::Value::DateTime)
                });
            match result {
                Ok(value) => {
                    message_map.insert(
                        "timestamp",
                        messages::Field {
                            name: "timestamp",
                            value,
                            units: "",
                            is_subfield: false,
                        },
                    );
                }
                Err(err) => self.errors.push(err),
            };
        }
        if msg_ty == types::MesgNum::DeveloperDataId {
            let developer_data_map = message_map
                .into_iter()
                .map(|(name, field)| match name {
                    "manufacturer_id" => (
                        name,
                        fit::Value::String(
                            types::Manufacturer::try_from(&field.value)
                                .unwrap()
                                .to_string(),
                        ),
                    ),
                    _ => (name, field.value),
                })
                .collect::<HashMap<_, _>>();
            let developer_data_index =
                fit_value_covert!(developer_data_map.get("developer_data_index"), UInt8);
            self.dev_data_defs.insert(
                developer_data_index,
                FitDeveloperDataDefinition {
                    developer_data_index,
                    developer_id: developer_data_map
                        .get("developer_id")
                        .map(|it| it.to_owned()),
                    application_id: developer_data_map
                        .get("application_id")
                        .map(|it| it.to_owned()),
                    manufacturer_id: developer_data_map
                        .get("manufacturer_id")
                        .map(|it| it.to_owned()),
                    application_version: fit_value_covert!(
                        developer_data_map.get("application_version"),
                        UInt32
                    ),
                    field_map: HashMap::new(),
                },
            );
            Ok((msg_ty.to_string(), developer_data_map))
        } else if msg_ty == types::MesgNum::FieldDescription {
            let field_def_names = [
                "developer_data_index",
                "field_definition_number",
                "fit_base_type_id",
                "field_name",
            ];
            if let Some(field_def_name) = field_def_names
                .iter()
                .find(|&it| !message_map.contains_key(it))
            {
                return Err(ErrorKind::InvalidDeveloperField {
                    name: field_def_name.to_string(),
                });
            }
            let field_description_map = message_map
                .into_iter()
                .map(|(name, field)| match name {
                    "fit_base_type_id" => (
                        name,
                        fit::Value::String(
                            types::FitBaseType::try_from(&field.value)
                                .unwrap()
                                .to_string(),
                        ),
                    ),
                    "fit_base_unit_id" => (
                        name,
                        fit::Value::String(
                            types::FitBaseUnit::try_from(&field.value)
                                .unwrap()
                                .to_string(),
                        ),
                    ),
                    "native_mesg_num" => (
                        name,
                        fit::Value::String(
                            types::MesgNum::try_from(&field.value).unwrap().to_string(),
                        ),
                    ),
                    _ => (name, field.value),
                })
                .collect::<HashMap<_, _>>();
            let developer_data_index =
                fit_value_covert!(field_description_map.get("developer_data_index"), UInt8);
            let field_definition_number =
                fit_value_covert!(field_description_map.get("field_definition_number"), UInt8);
            if let Some(def) = self.dev_data_defs.get_mut(&developer_data_index) {
                def.field_map
                    .insert(field_definition_number, field_description_map.clone());
            }
            Ok((msg_ty.to_string(), field_description_map))
        } else {
            Ok((
                msg_ty.to_string(),
                message_map
                    .into_iter()
                    .map(|(k, v)| (k, v.value))
                    .collect::<HashMap<_, _>>(),
            ))
        }
    }
    fn update_time_offset(&mut self, offset: u8) -> ParserResult<u32> {
        let previous = if let Some(previous) = self.timestamp_ref {
            previous
        } else {
            return Err(ErrorKind::MissingTimestampRef);
        };
        let offset = offset as u32;
        let next = if offset >= (previous & 0x0000_001F) {
            (previous & 0xFFFF_FFE0) + offset
        } else {
            (previous & 0xFFFF_FFE0) + offset + 0x20
        };
        self.timestamp_ref = Some(next);
        Ok(next)
    }
}
