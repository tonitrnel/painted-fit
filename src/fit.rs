use chrono::{DateTime, Local};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub(crate) enum BaseType {
    Enum = 0x00,
    SInt8 = 0x01,
    UInt8 = 0x02,
    SInt16 = 0x83,
    UInt16 = 0x84,
    SInt32 = 0x85,
    UInt32 = 0x86,
    String = 0x07,
    Float32 = 0x88,
    Float64 = 0x89,
    UInt8z = 0x0A,
    UInt16z = 0x8B,
    UInt32z = 0x8C,
    Byte = 0x0D,
    SInt64 = 0x8E,
    UInt64 = 0x8F,
    UInt64z = 0x90,
}

impl BaseType {
    // pub(crate) fn is_numeric_field(&self) -> bool {
    //     matches!(
    //         self,
    //         BaseType::SInt8
    //             | BaseType::SInt16
    //             | BaseType::SInt32
    //             | BaseType::SInt64
    //             | BaseType::UInt8
    //             | BaseType::UInt16
    //             | BaseType::UInt32
    //             | BaseType::UInt64
    //             | BaseType::UInt8z
    //             | BaseType::UInt16z
    //             | BaseType::UInt32z
    //             | BaseType::UInt64z
    //             | BaseType::Float32
    //             | BaseType::Float64
    //             | BaseType::Byte
    //     )
    // }
    pub(crate) fn size(&self) -> u8 {
        match self {
            BaseType::Enum => 1,
            BaseType::SInt8 => 1,
            BaseType::UInt8 => 1,
            BaseType::SInt16 => 2,
            BaseType::UInt16 => 2,
            BaseType::SInt32 => 4,
            BaseType::UInt32 => 4,
            BaseType::String => 1,
            BaseType::Float32 => 4,
            BaseType::Float64 => 8,
            BaseType::UInt8z => 1,
            BaseType::UInt16z => 2,
            BaseType::UInt32z => 4,
            BaseType::Byte => 1,
            BaseType::SInt64 => 8,
            BaseType::UInt64 => 8,
            BaseType::UInt64z => 8,
        }
    }
    pub(crate) fn invalid(&self) -> usize {
        match self {
            BaseType::Enum => 0xFF,
            BaseType::SInt8 => 0x7F,
            BaseType::UInt8 => 0xFF,
            BaseType::SInt16 => 0x7FFF,
            BaseType::UInt16 => 0xFFFF,
            BaseType::SInt32 => 0x7FFF_FFFF,
            BaseType::UInt32 => 0xFFFF_FFFF,
            BaseType::String => 0x00,
            BaseType::Float32 => 0xFFFF_FFFF,
            BaseType::Float64 => 0xFFFF_FFFF_FFFF_FFFF,
            BaseType::UInt8z => 0x00,
            BaseType::UInt16z => 0x0000,
            BaseType::UInt32z => 0x0000_0000,
            BaseType::Byte => 0xFF,
            BaseType::SInt64 => 0x7FFF_FFFF_FFFF_FFFF,
            BaseType::UInt64 => 0xFFFF_FFFF_FFFF_FFFF,
            BaseType::UInt64z => 0x0000_0000_0000_0000,
        }
    }
}

impl TryFrom<u8> for BaseType {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(BaseType::Enum),
            0x01 => Ok(BaseType::SInt8),
            0x02 => Ok(BaseType::UInt8),
            0x83 => Ok(BaseType::SInt16),
            0x84 => Ok(BaseType::UInt16),
            0x85 => Ok(BaseType::SInt32),
            0x86 => Ok(BaseType::UInt32),
            0x07 => Ok(BaseType::String),
            0x88 => Ok(BaseType::Float32),
            0x89 => Ok(BaseType::Float64),
            0x0A => Ok(BaseType::UInt8z),
            0x8B => Ok(BaseType::UInt16z),
            0x8C => Ok(BaseType::UInt32z),
            0x0D => Ok(BaseType::Byte),
            0x8E => Ok(BaseType::SInt64),
            0x8F => Ok(BaseType::UInt64),
            0x90 => Ok(BaseType::UInt64z),
            _ => Err("No corresponding fit BaseType exists"),
        }
    }
}

impl TryFrom<&str> for BaseType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "enum" => Ok(BaseType::Enum),
            "sint8" => Ok(BaseType::SInt8),
            "uint8" => Ok(BaseType::UInt8),
            "sint16" => Ok(BaseType::SInt16),
            "uint16" => Ok(BaseType::UInt16),
            "sint32" => Ok(BaseType::SInt32),
            "uint32" => Ok(BaseType::UInt32),
            "string" => Ok(BaseType::String),
            "float32" => Ok(BaseType::Float32),
            "float64" => Ok(BaseType::Float64),
            "uint8z" => Ok(BaseType::UInt8z),
            "uint16z" => Ok(BaseType::UInt16z),
            "uint32z" => Ok(BaseType::UInt32z),
            "byte" => Ok(BaseType::Byte),
            "sint64" => Ok(BaseType::SInt64),
            "uint64" => Ok(BaseType::UInt64),
            "uint64z" => Ok(BaseType::UInt64z),
            _ => Err("No corresponding BaseType exists"),
        }
    }
}

pub(crate) fn is_base_type(val: &str) -> bool {
    if val == "bool" {
        true
    } else {
        BaseType::try_from(val).is_ok()
    }
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BaseType::Enum => f.write_str("enum"),
            BaseType::SInt8 => f.write_str("sint8"),
            BaseType::UInt8 => f.write_str("uint8"),
            BaseType::SInt16 => f.write_str("sint16"),
            BaseType::UInt16 => f.write_str("uint16"),
            BaseType::SInt32 => f.write_str("sint32"),
            BaseType::UInt32 => f.write_str("uint32"),
            BaseType::String => f.write_str("string"),
            BaseType::Float32 => f.write_str("float32"),
            BaseType::Float64 => f.write_str("float64"),
            BaseType::UInt8z => f.write_str("uint8z"),
            BaseType::UInt16z => f.write_str("uint16z"),
            BaseType::UInt32z => f.write_str("uint32z"),
            BaseType::Byte => f.write_str("byte"),
            BaseType::SInt64 => f.write_str("sint64"),
            BaseType::UInt64 => f.write_str("uint64"),
            BaseType::UInt64z => f.write_str("uint64z"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) enum Value {
    Enum(u8),
    SInt8(i8),
    UInt8(u8),
    SInt16(i16),
    UInt16(u16),
    SInt32(i32),
    UInt32(u32),
    String(String),
    Float32(f32),
    Float64(f64),
    UInt8z(u8),
    UInt16z(u16),
    UInt32z(u32),
    Byte(u8),
    SInt64(i64),
    UInt64(u64),
    UInt64z(u64),
    // supplementary fields
    Bool(bool),
    Array(Vec<Self>),
}
impl Value {
    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Value::Enum(val) => *val != 0xFF,
            Value::SInt8(val) => *val != 0x7F,
            Value::UInt8(val) => *val != 0xFF,
            Value::SInt16(val) => *val != 0x7FFF,
            Value::UInt16(val) => *val != 0xFFFF,
            Value::SInt32(val) => *val != 0x7FFF_FFFF,
            Value::UInt32(val) => *val != 0xFFFF_FFFF,
            Value::String(val) => !val.contains('\0'),
            Value::Float32(val) => val.is_finite(),
            Value::Float64(val) => val.is_finite(),
            Value::UInt8z(val) => *val != 0x0,
            Value::UInt16z(val) => *val != 0x0,
            Value::UInt32z(val) => *val != 0x0,
            Value::Byte(val) => *val != 0xFF,
            Value::SInt64(val) => *val != 0x7FFF_FFFF_FFFF_FFFF,
            Value::UInt64(val) => *val != 0xFFFF_FFFF_FFFF_FFFF,
            Value::UInt64z(val) => *val != 0x0,
            Value::Bool(_) => true,
            Value::Array(vals) => !vals.is_empty() && vals.iter().all(|v| v.is_valid()),
        }
    }
}
