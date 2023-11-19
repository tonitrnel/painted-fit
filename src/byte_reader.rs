use crate::error::ErrorKind;
use std::ops::{Index, Range, RangeFrom, RangeTo};

pub(crate) struct ByteReader<'input> {
    offset: usize,
    bytes: &'input [u8],
}

macro_rules! convert_impl {
    ($func_name:ident, $type:ty, $size:expr) => {
        /// Read specified size bytes and converts it to the target type
        pub(crate) fn $func_name(&mut self, is_big_endian: bool) -> $type {
            let bytes = self
                .read_bytes($size)
                .try_into()
                .expect("incorrect number of bytes");
            if is_big_endian {
                <$type>::from_be_bytes(bytes)
            } else {
                <$type>::from_le_bytes(bytes)
            }
        }
    };
}

impl<'input> ByteReader<'input> {
    pub(crate) fn new(bytes: &'input [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }
    pub(crate) fn offset(&self) -> usize {
        self.offset
    }
    pub(crate) fn read_bytes(&mut self, len: usize) -> &'input [u8] {
        if self.offset + len > self.bytes.len() {
            panic!(
                "{:?}",
                ErrorKind::OutOfBoundsRead {
                    offset: self.offset,
                    requested_len: len,
                    remaining_len: self.bytes.len().saturating_sub(self.offset),
                }
            );
        }
        let bytes = &self.bytes[self.offset..self.offset + len];
        self.offset += len;
        bytes
    }
    pub(crate) fn is_end(&self) -> bool {
        self.offset >= self.bytes.len() - 1
    }
    pub(crate) fn read_next_u8(&mut self) -> u8 {
        self.read_bytes(1)[0]
    }
    pub(crate) fn read_next_i8(&mut self) -> i8 {
        self.read_bytes(1)[0] as i8
    }

    convert_impl!(read_next_u16, u16, 2);
    convert_impl!(read_next_i16, i16, 2);
    convert_impl!(read_next_u32, u32, 4);
    convert_impl!(read_next_i32, i32, 4);
    convert_impl!(read_next_f32, f32, 4);
    convert_impl!(read_next_u64, u64, 8);
    convert_impl!(read_next_i64, i64, 8);
    convert_impl!(read_next_f64, f64, 8);

    pub(crate) fn read_next_uft8_string(&mut self, len: usize) -> String {
        String::from_utf8_lossy(self.read_bytes(len)).to_string()
    }
    pub(crate) fn reset(&mut self) {
        self.offset = 0
    }
}

trait FromNumeric {
    fn from_be_bytes(bytes: &[u8]) -> Self;
    fn from_le_bytes(bytes: &[u8]) -> Self;
}

impl<'input> From<&'input [u8]> for ByteReader<'input> {
    fn from(value: &'input [u8]) -> Self {
        ByteReader::new(value)
    }
}

impl Index<usize> for ByteReader<'_> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        self.bytes.index(index)
    }
}

impl Index<Range<usize>> for ByteReader<'_> {
    type Output = [u8];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        self.bytes.index(index)
    }
}
impl Index<RangeFrom<usize>> for ByteReader<'_> {
    type Output = [u8];
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        self.bytes.index(index)
    }
}
impl Index<RangeTo<usize>> for ByteReader<'_> {
    type Output = [u8];
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        self.bytes.index(index)
    }
}
