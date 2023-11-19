use crate::fit;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ErrorKind {
    #[error("Input is not a FIT file.")]
    InvalidFitFile,
    #[error("CRC invalid.")]
    InvalidCRC,
    #[error("Out of bounds read: attempted to read {requested_len} bytes from offset {offset}, but only {remaining_len} bytes are available.")]
    OutOfBoundsRead {
        offset: usize,
        requested_len: usize,
        remaining_len: usize,
    },
    #[error("Byte conversion error: failed to convert {source_len} bytes into a fixed-size type.")]
    ByteConversionError { source_len: usize },
    #[error("Invalid message header.")]
    InvalidMessageHeader,
    #[error("Invalid field value: expected type '{base_type:?}', not equal to '{invalid}', received '{value:?}'")]
    InvalidFieldValue {
        base_type: fit::BaseType,
        invalid: usize,
        value: fit::Value,
    },
    #[error("Failed to decode message '{message_no}' field '{field_no}',reason: {reason}")]
    DecodeFieldFailed {
        message_no: u16,
        field_no: u8,
        reason: String,
    },
    #[error("Failed to decode message '{message_no}' data_index '{data_index}',reason: {reason}")]
    DecodeDeveloperFieldFailed {
        message_no: u16,
        data_index: u8,
        reason: String,
    },
    #[error("Field size mismatch: {field_size} is not a multiple of base type size {base_type_size}, indicating potential misalignment.")]
    SizeMismatch {
        field_size: usize,
        base_type_size: u8,
    },
    #[error("Definition for local message number {0} not found.")]
    LocalDefinitionMessageNotFound(u8),
    #[error("Definition for global message number {0} not found.")]
    GlobalDefinitionMessageNotFound(u16),
    #[error("Unexpected missing global message {0} definition.")]
    UnknownMessage(String),
    #[error("Developer data definition missing for field '{name}'")]
    InvalidDeveloperField { name: String },
    #[error("Failed to decode message '{message}', field {field_no}: {reason}")]
    DecodeMessageFailed {
        message: String,
        reason: String,
        field_no: u8,
    },
    #[error("Base type mismatch: {reason}")]
    BaseTypeMismatch { reason: String },
    #[error("Invalid compressed timestamp: missing timestamp reference.")]
    MissingTimestampRef,
    #[error("Invalid timestamp {timestamp}: cannot convert to DateTime")]
    InvalidTimestamp { timestamp: u32 },
    #[error("Missing developer data definition for developer data index {developer_data_index}")]
    MissingDeveloperDataDef { developer_data_index: u8 },
    #[error("Missing developer field description for developer data index {developer_data_index}, field {field_no}")]
    MissingDeveloperFieldDescription {
        developer_data_index: u8,
        field_no: u8,
    },
}
pub type ParserResult<T> = Result<T, ErrorKind>;

impl ErrorKind {
    fn kind(&self) -> &'static str {
        use ErrorKind::*;
        match self {
            InvalidFitFile => "InvalidFitFile",
            InvalidCRC => "InvalidCRC",
            OutOfBoundsRead { .. } => "OutOfBoundsRead",
            ByteConversionError { .. } => "ByteConversionError",
            InvalidMessageHeader => "InvalidMessageHeader",
            InvalidFieldValue { .. } => "InvalidFieldValue",
            DecodeFieldFailed { .. } => "DecodeFieldFailed",
            DecodeDeveloperFieldFailed { .. } => "DecodeDeveloperFieldFailed",
            SizeMismatch { .. } => "SizeMismatch",
            LocalDefinitionMessageNotFound(_) => "LocalDefinitionMessageNotFound",
            GlobalDefinitionMessageNotFound(_) => "GlobalDefinitionMessageNotFound",
            UnknownMessage(_) => "UnknownMessage",
            InvalidDeveloperField { .. } => "InvalidDeveloperField",
            DecodeMessageFailed { .. } => "DecodeMessageFailed",
            BaseTypeMismatch { .. } => "BaseTypeMismatch",
            MissingTimestampRef => "MissingTimestampRef",
            InvalidTimestamp { .. } => "InvalidTimestamp",
            MissingDeveloperDataDef { .. } => "MissingDeveloperDataDef",
            MissingDeveloperFieldDescription { .. } => "MissingDeveloperFieldDescription",
        }
    }
}

impl Serialize for ErrorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut builder = serializer.serialize_struct("Error", 2).unwrap();
        builder.serialize_field("kind", self.kind()).unwrap();
        builder
            .serialize_field("message", &self.to_string())
            .unwrap();
        builder.end()
    }
}
