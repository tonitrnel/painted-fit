use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum BaseType {
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
    pub fn size(&self) -> u8 {
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

    #[allow(unused)]
    pub fn invalid(&self) -> usize {
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
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            BaseType::SInt8
                | BaseType::SInt16
                | BaseType::SInt32
                | BaseType::SInt64
                | BaseType::UInt8
                | BaseType::UInt16
                | BaseType::UInt32
                | BaseType::UInt64
                | BaseType::UInt8z
                | BaseType::UInt16z
                | BaseType::UInt32z
                | BaseType::UInt64z
                | BaseType::Float32
                | BaseType::Float64
                | BaseType::Byte
        )
    }
}

impl TryFrom<u8> for BaseType {
    type Error = String;
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
            _ => Err(format!(
                "Value '{value}' does not exist to match fit BaseType"
            )),
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

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
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
    // Supplementary fields
    DateTime(DateTime<Utc>), // Appears only after parsing
    Bool(bool),
    Array(Vec<Self>),
}
impl Value {
    pub fn is_valid(&self) -> bool {
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
            // Supplementary fields
            Value::DateTime(_) => true, // Appears only after parsing
            Value::Bool(_) => true,
            Value::Array(vals) => !vals.is_empty() && vals.iter().all(|v| v.is_valid()),
        }
    }

    // pub fn try_as_u8(&self) -> Result<u8, &'static str> {
    //     match self {
    //         Value::Enum(v) => Ok(*v),
    //         Value::UInt8(v) => Ok(*v),
    //         Value::UInt8z(v) => Ok(*v),
    //         Value::Byte(v) => Ok(*v),
    //         _ => Err("Cannot be converted to 'u8' type."),
    //     }
    // }
    // pub fn try_as_f64(&self) -> Result<f64, &'static str> {
    //     match self {
    //         Value::SInt8(v) => Ok(*v as f64),
    //         Value::SInt16(v) => Ok(*v as f64),
    //         Value::SInt32(v) => Ok(*v as f64),
    //         // It will overflow.
    //         // Value::SInt64(v) => Ok(*v as f64),
    //         Value::UInt8(v) => Ok(*v as f64),
    //         Value::UInt16(v) => Ok(*v as f64),
    //         Value::UInt32(v) => Ok(*v as f64),
    //         // Value::UInt64(v) => Ok(*v as f64),
    //         Value::UInt8z(v) => Ok(*v as f64),
    //         Value::UInt16z(v) => Ok(*v as f64),
    //         Value::UInt32z(v) => Ok(*v as f64),
    //         // Value::UInt64z(v) => Ok(*v as f64),
    //         Value::Float32(v) => Ok(*v as f64),
    //         Value::Float64(v) => Ok(*v),
    //         Value::Byte(v) => Ok(*v as f64),
    //         _ => Err("Cannot be converted to 'f64' type."),
    //     }
    // }
    pub fn try_as_usize(&self) -> Result<usize, &'static str> {
        match self {
            Value::UInt8(v) => Ok(*v as usize),
            Value::UInt16(v) => Ok(*v as usize),
            Value::UInt32(v) => Ok(*v as usize),
            Value::UInt64(v) => Ok(*v as usize),
            Value::UInt8z(v) => Ok(*v as usize),
            Value::UInt16z(v) => Ok(*v as usize),
            Value::UInt32z(v) => Ok(*v as usize),
            Value::Byte(v) => Ok(*v as usize),
            _ => Err("Cannot be converted to 'usize' type."),
        }
    }

    #[allow(unused)]
    pub fn to_base_type_str(&self) -> &str {
        match self {
            Value::Enum(_) => "enum",
            Value::SInt8(_) => "sint8",
            Value::UInt8(_) => "uint8",
            Value::SInt16(_) => "sint16",
            Value::UInt16(_) => "uint16",
            Value::SInt32(_) => "sint32",
            Value::UInt32(_) => "uint32",
            Value::String(_) => "string",
            Value::Float32(_) => "float32",
            Value::Float64(_) => "float64",
            Value::UInt8z(_) => "uint8z",
            Value::UInt16z(_) => "uint16z",
            Value::UInt32z(_) => "uint32z",
            Value::Byte(_) => "byte",
            Value::SInt64(_) => "sint64",
            Value::UInt64(_) => "uint64",
            Value::UInt64z(_) => "uint64z",
            Value::DateTime(_) => "uint32",
            Value::Bool(_) => "byte",
            Value::Array(arr) => arr[0].to_base_type_str(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Value::DateTime(val) => val.to_string(),
            Value::Enum(val) => val.to_string(),
            Value::SInt8(val) => val.to_string(),
            Value::UInt8(val) => val.to_string(),
            Value::SInt16(val) => val.to_string(),
            Value::UInt16(val) => val.to_string(),
            Value::SInt32(val) => val.to_string(),
            Value::UInt32(val) => val.to_string(),
            Value::String(val) => val.to_owned(),
            Value::Float32(val) => val.to_string(),
            Value::Float64(val) => val.to_string(),
            Value::UInt8z(val) => val.to_string(),
            Value::UInt16z(val) => val.to_string(),
            Value::UInt32z(val) => val.to_string(),
            Value::Byte(val) => val.to_string(),
            Value::SInt64(val) => val.to_string(),
            Value::UInt64(val) => val.to_string(),
            Value::UInt64z(val) => val.to_string(),
            Value::Bool(val) => val.to_string(),
            Value::Array(vals) => vals.iter().map(|it| it.to_string()).collect::<String>(),
        };
        write!(f, "{}", str)
    }
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::DateTime(val) => write!(f, "DateTime({:?})", val),
            Value::Enum(val) => write!(f, "Enum({:?})", val),
            Value::SInt8(val) => write!(f, "SInt8({:?})", val),
            Value::UInt8(val) => write!(f, "UInt8({:?})", val),
            Value::SInt16(val) => write!(f, "SInt16({:?})", val),
            Value::UInt16(val) => write!(f, "UInt16({:?})", val),
            Value::SInt32(val) => write!(f, "SInt32({:?})", val),
            Value::UInt32(val) => write!(f, "UInt32({:?})", val),
            Value::String(val) => write!(f, "String({:?})", val),
            Value::Float32(val) => write!(f, "Float32({:?})", val),
            Value::Float64(val) => write!(f, "Float64({:?})", val),
            Value::UInt8z(val) => write!(f, "UInt8z({:?})", val),
            Value::UInt16z(val) => write!(f, "UInt16z({:?})", val),
            Value::UInt32z(val) => write!(f, "UInt32z({:?})", val),
            Value::Byte(val) => write!(f, "Byte({:?})", val),
            Value::SInt64(val) => write!(f, "SInt64({:?})", val),
            Value::UInt64(val) => write!(f, "UInt64({:?})", val),
            Value::UInt64z(val) => write!(f, "UInt64z({:?})", val),
            Value::Bool(val) => write!(f, "Bool({:?})", val),
            Value::Array(vals) => write!(
                f,
                "Array([{}])",
                vals.iter()
                    .map(|it| format!("{:?}", it))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl From<&Value> for BaseType {
    fn from(value: &Value) -> Self {
        match value {
            Value::Enum(_) => BaseType::Enum,
            Value::SInt8(_) => BaseType::SInt8,
            Value::UInt8(_) => BaseType::UInt8,
            Value::SInt16(_) => BaseType::SInt16,
            Value::UInt16(_) => BaseType::UInt16,
            Value::SInt32(_) => BaseType::SInt32,
            Value::UInt32(_) => BaseType::UInt32,
            Value::String(_) => BaseType::String,
            Value::Float32(_) => BaseType::Float32,
            Value::Float64(_) => BaseType::Float64,
            Value::UInt8z(_) => BaseType::UInt8z,
            Value::UInt16z(_) => BaseType::UInt16z,
            Value::UInt32z(_) => BaseType::UInt32z,
            Value::Byte(_) => BaseType::Byte,
            Value::SInt64(_) => BaseType::SInt64,
            Value::UInt64(_) => BaseType::UInt64,
            Value::UInt64z(_) => BaseType::UInt64z,
            Value::DateTime(_) => BaseType::UInt32,
            Value::Bool(_) => BaseType::Byte,
            Value::Array(arr) => BaseType::from(&arr[0]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        println!("{:?} {}", Value::UInt32(55), Value::UInt32(55))
    }
}
