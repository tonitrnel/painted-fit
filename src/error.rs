use crate::fit;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ErrorKind {
    #[error("Input is not a FIT file.")]
    InvalidFitFile,
    #[error("CRC invalid.")]
    CRCInvalid,
    #[error("Invalid message header.")]
    MessageHeaderInvalid,
    #[error("Invalid field value {0:?}.")]
    FieldValueInvalid(fit::Value),
    #[error("The size of the field ({field_size}) is not a multiple of its base type size ({base_type_size}). This may indicate a misalignment or a mismatch in expected data layout.")]
    SizeMismatch {
        field_size: usize,
        base_type_size: usize,
    },
    #[error("Definition for local message number {0} not found.")]
    LocalDefinitionMessageNotFound(u8),
    #[error("Definition for global message number {0} not found.")]
    GlobalDefinitionMessageNotFound(u16),
    #[error("Unexpected missing global message {0} definition.")]
    UnknownMessage(String),
    #[error("Invalid developer field definition, missing '{name}' field.")]
    InvalidDeveloperField { name: String },
    #[error("Decode message field failed, reason: {reason}")]
    DecodeMessageFailed { reason: String },
    #[error("{reason}")]
    BaseTypeMismatch { reason: String },
    #[error("Invalid compressed timestamp, Cannot find timestamp reference.")]
    MissingTimestampRef,
    #[error("Unable to convert the timestamp {0} to a DateTime")]
    InvalidTimestamp(u32),
}
pub type ParserResult<T> = Result<T, ErrorKind>;
