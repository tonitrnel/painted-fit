mod accumulator;
mod bit_reader;
mod byte_reader;
mod crc;
pub mod decoder;
pub mod error;
mod fit;
pub mod profile;

pub use fit::Value;
pub use profile::VERSION as PROFILE_VERSION;

#[allow(unused_imports)]
use decoder::Decoder;
