use crate::byte_reader::ByteReader;
use crate::crc;
use crate::fit;
use std::collections::HashMap;
use std::sync::Arc;

struct DecoderOptions {
    expand_sub_fields: bool,
    expand_components: bool,
    apply_scale_and_offset: bool,
    convert_types_to_string: bool,
    merge_heart_rates: bool,
}
#[derive(Debug, Clone)]
struct FitFileHeader {
    header_size: u32,
    protocol_version: u8,
    profile_version: u16,
    data_size: u32,
    data_type: String,
    header_crc: u16,
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
    time_offset: u8,
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
struct DeveloperFieldDefinition {
    field_number: u8,
    size: u8,
    developer_data_index: u8,
}

#[derive(Debug, Clone)]
struct FitProfileMessage {
    name: String,
    message_key: String,
    number: u8,
    fields: HashMap<u8, u8>,
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
    time_offset: u8,
    fields: HashMap<u8, fit::Value>,
    developer_fields: Vec<fit::Value>,
}

const CRC_SIZE: u32 = 2;

struct Decoder<'input> {
    reader: ByteReader<'input>,
    defs: HashMap<u8, Arc<FitDefinitionMessage>>,
}

impl<'input> Decoder<'input> {
    fn new(bytes: &'input [u8]) -> Self {
        Decoder {
            reader: bytes.into(),
            defs: HashMap::new(),
        }
    }
    fn read_file_header(&mut self) -> FitFileHeader {
        self.reader.reset();
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
    fn is_fit(bytes: &[u8]) -> bool {
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
    fn check_integrity(&self, header: &FitFileHeader) -> bool {
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
    /// 阅读信息
    fn read(&mut self) {
        self.reader.reset();
        while !self.reader.is_end() {
            self.read_next_record();
        }
    }
    fn read_next_record(&mut self) {
        let start = self.reader.offset();
        // println!("read next record, start at {}", start);
        if !Decoder::is_fit(&self.reader[start..]) {
            panic!("Error: input is not a FIT file")
        };
        let header = self.read_file_header();
        let end = start + header.header_size as usize + header.data_size as usize;
        while self.reader.offset() < end {
            let message = self.read_fit_message();
            match message {
                FitMessage::Definition(message) => {
                    // println!(
                    //     "[->] Definition G[{}] L[{}] defs: {} developer_defs: {}",
                    //     message.global_message_number,
                    //     message.local_message_number,
                    //     message.field_definitions.len(),
                    //     message.developer_field_definitions.len()
                    // );
                    self.defs
                        .insert(message.local_message_number, Arc::new(message));
                }
                FitMessage::Data(message) => decode_message(message),
            }
        }
        if self.reader.read_next_u16(false) != crc::crc_16(&self.reader[start..end]) {
            panic!("CRC invalid")
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
    fn read_message_header(&mut self) -> FitMessageHeader {
        let byte = self.reader.read_next_u8();
        if byte & 0x80 == 0x80 {
            // compressed timestamp header
            FitMessageHeader {
                contains_developer_data: false,
                local_message_number: (byte >> 5) & 0x03,
                message_type: FitMessageType::Data,
                time_offset: byte & 0x1F,
            }
        } else if byte & 0x40 == 0x40 {
            // definition message
            FitMessageHeader {
                contains_developer_data: byte & 0x20 == 0x20,
                local_message_number: byte & 0x0F,
                message_type: FitMessageType::Definition,
                time_offset: 0,
            }
        } else if byte & 0x40 == 0x00 {
            // data message
            FitMessageHeader {
                contains_developer_data: false,
                local_message_number: byte & 0x0F,
                message_type: FitMessageType::Data,
                time_offset: 0,
            }
        } else {
            panic!("ERROR: invalid message header")
        }
    }
    fn read_fit_message(&mut self) -> FitMessage {
        let header = self.read_message_header();
        match &header.message_type {
            FitMessageType::Definition => {
                FitMessage::Definition(self.read_definition_message(&header))
            }
            FitMessageType::Data => FitMessage::Data(self.read_data_message(&header)),
        }
    }
    fn read_data_message(&mut self, header: &FitMessageHeader) -> FitDataMessage {
        let def = self
            .defs
            .get(&header.local_message_number)
            .expect("ERROR: can't find a corresponding definition message")
            .clone();

        let mut fields = HashMap::new();
        for field_def in &def.field_definitions {
            let value = self.read_field_value(
                field_def.size as usize,
                field_def.base_type,
                def.architecture.is_big_endian(),
            );
            fields.insert(field_def.field_definition_number, value);
        }

        let mut developer_fields = Vec::new();

        for field_def in &def.developer_field_definitions {
            let value = self.read_field_value(
                field_def.size as usize,
                fit::BaseType::Byte,
                def.architecture.is_big_endian(),
            );
            developer_fields.push(value);
        }
        FitDataMessage {
            fields,
            developer_fields,
            global_message_number: def.global_message_number,
            time_offset: header.time_offset,
        }
    }
    fn read_field_value(
        &mut self,
        size: usize,
        base_type: fit::BaseType,
        is_big_endian: bool,
    ) -> fit::Value {
        use fit::{BaseType, Value};
        if size % base_type.size() as usize != 0 {
            panic!("ERROR: field size: {} is not a multiple of the base type {:?} (size {}) parsing as a byte array", size, base_type, base_type.size());
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
            panic!("ERROR: invalid field value");
        }
        value
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
    fn read_definition_message(&mut self, header: &FitMessageHeader) -> FitDefinitionMessage {
        // consume reserved byte
        self.reader.read_next_u8();
        let architecture = Architecture::from(self.reader.read_next_u8());
        let global_message_number = self.reader.read_next_u16(architecture.is_big_endian());
        let field_definitions = {
            let mut definitions = Vec::new();
            let number_of_fields = self.reader.read_next_u8();
            for _ in 0..number_of_fields {
                let field_definition_number = self.reader.read_next_u8();
                let size = self.reader.read_next_u8();
                let base_type = fit::BaseType::try_from(self.reader.read_next_u8()).unwrap();
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
        FitDefinitionMessage {
            architecture,
            local_message_number: header.local_message_number,
            global_message_number,
            field_definitions,
            developer_field_definitions,
        }
    }
}
fn decode_message(message: FitDataMessage) {
    let message_type =
        crate::profile::types::MesgNum::try_from(fit::Value::UInt16(message.global_message_number))
            .unwrap_or_else(|_| panic!("Cannot find message definition in profile"))
            .to_string();
    if message_type != "file_id" {
        return;
    }
    println!("message type: {}", message_type);
    let message_map = crate::profile::messages::from_str(&message_type);
    if let Some(message_map) = message_map {
        for (field_def_number, val) in message.fields.iter() {
            println!(
                "field_def_number = {field_def_number} val = {val:?} => {:?}",
                message_map.get(field_def_number)
            );
        }
    } else {
        panic!("Cannot find message in profile")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn it_works() {
        use std::fs::File;
        let mut fp = File::open("examples/Activity.fit").unwrap();
        let mut buffer = Vec::new();
        fp.read_to_end(&mut buffer).expect("Read file failed");
        println!("is_fit = {}", Decoder::is_fit(&buffer));
        let mut decoder = Decoder::new(&buffer);
        let header = decoder.read_file_header();
        println!("check_integrity = {}", decoder.check_integrity(&header));
        println!("{:#?}", header);
        decoder.read();
    }
}
