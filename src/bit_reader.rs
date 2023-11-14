use crate::fit;

pub(crate) struct BitReader {
    bytes: Vec<fit::Value>,
    byte_pos: usize,
    bit_pos: usize,
    per_size: usize,
    available_bits: usize,
}

impl BitReader {
    pub(crate) fn new(value: fit::Value) -> Self {
        let bytes = if let fit::Value::Array(arr) = value {
            arr
        } else {
            vec![value]
        };
        let per_size = (fit::BaseType::from(&bytes[0]).size() * 8) as usize;
        let available_bits = per_size * bytes.len();
        Self {
            bytes,
            per_size,
            available_bits,
            byte_pos: 0,
            bit_pos: 0,
        }
    }
    pub(crate) fn available(&self) -> bool {
        self.available_bits > 0
    }
    pub(crate) fn next(&mut self) -> Option<u8> {
        if !self.available() {
            return None;
        }
        let bit = (self.bytes[self.byte_pos].try_as_usize().ok()? >> self.bit_pos & 0x01) as u8;
        self.bit_pos += 1;
        if self.bit_pos >= self.per_size {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
        self.available_bits -= 1;
        Some(bit)
    }
    pub(crate) fn read_bits(&mut self, len: usize) -> Option<usize> {
        let mut value = 0usize;
        for i in 0..len {
            value |= (self.next()? as usize) << i;
        }
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scenario {
        data: &'static [usize],
        base_type: fit::BaseType,
        n_bits_to_read: &'static [usize],
        values: &'static [usize],
    }

    impl Scenario {
        fn to_value(&self) -> fit::Value {
            fit::Value::Array(
                self.data
                    .iter()
                    .map(|d| match self.base_type {
                        fit::BaseType::UInt8 => fit::Value::UInt8(*d as u8),
                        fit::BaseType::UInt16 => fit::Value::UInt16(*d as u16),
                        fit::BaseType::UInt32 => fit::Value::UInt32(*d as u32),
                        _ => panic!("Not supported"),
                    })
                    .collect::<Vec<_>>(),
            )
        }
    }

    #[test]
    fn from_byte_array_tests() {
        let mut reader = BitReader::new(fit::Value::Array(vec![
            fit::Value::UInt8(0xAA),
            fit::Value::UInt8(0xAA),
        ]));
        let values = &[0u8, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        for (index, expected) in values.iter().enumerate() {
            assert_eq!(reader.available_bits, values.len() - index);
            assert!(reader.available());
            assert_eq!(Some(*expected), reader.next());
            assert_eq!(reader.available_bits, values.len() - index - 1);
            assert_eq!(reader.available(), index + 1 != values.len());
        }
        let scenarios = vec![
            Scenario {
                data: &[0xAA],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[4, 4],
                values: &[0xA, 0xA],
            },
            Scenario {
                data: &[0xAA],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[8],
                values: &[0xAA],
            },
            Scenario {
                data: &[0xAA, 0xAA],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[16],
                values: &[0xAAAA],
            },
            Scenario {
                data: &[0xFF, 0xFF],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[16],
                values: &[0xFFFF],
            },
            Scenario {
                data: &[0xAA, 0xAA, 0xAA, 0x2A],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[32],
                values: &[0x2AAAAAAA],
            },
            Scenario {
                data: &[0x10, 0x32, 0x54, 0x76],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[32],
                values: &[0x76543210],
            },
        ];
        for scenario in scenarios {
            let mut reader = BitReader::new(scenario.to_value());
            for (index, value) in scenario.values.iter().enumerate() {
                assert_eq!(
                    Some(*value),
                    reader.read_bits(scenario.n_bits_to_read[index])
                );
            }
        }
    }

    #[test]
    fn from_integer_tests() {
        let mut reader = BitReader::new(fit::Value::UInt16(0x0FAA));
        let values = &[0u8, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0];
        for value in values {
            assert_eq!(Some(*value), reader.next())
        }
        let scenarios = vec![
            Scenario {
                data: &[0xAA],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[4],
                values: &[0xA],
            },
            Scenario {
                data: &[0xAA],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[4, 4],
                values: &[0xA, 0xA],
            },
            Scenario {
                data: &[0xAA],
                base_type: fit::BaseType::UInt8,
                n_bits_to_read: &[4, 1, 1, 1, 1],
                values: &[0xA, 0x0, 0x1, 0x0, 0x1],
            },
            Scenario {
                data: &[0xAA],
                base_type: fit::BaseType::UInt16,
                n_bits_to_read: &[4, 1, 1, 1, 1],
                values: &[0xA, 0x0, 0x1, 0x0, 0x1],
            },
            Scenario {
                data: &[0xAAAA, 0x2AAA],
                base_type: fit::BaseType::UInt16,
                n_bits_to_read: &[32],
                values: &[0x2AAAAAAA],
            },
            Scenario {
                data: &[0xAAAAAAAA],
                base_type: fit::BaseType::UInt32,
                n_bits_to_read: &[16, 8, 8],
                values: &[0xAAAA, 0xAA, 0xAA],
            },
        ];

        for scenario in scenarios {
            let mut reader = BitReader::new(scenario.to_value());
            for (index, value) in scenario.values.iter().enumerate() {
                assert_eq!(
                    Some(*value),
                    reader.read_bits(scenario.n_bits_to_read[index])
                );
            }
        }
    }

    #[test]
    fn exception() {
        // When reading more bits than available bts should got None
        let mut reader = BitReader::new(fit::Value::UInt16(0xAAAA));
        reader.read_bits(16);
        assert_eq!(reader.next(), None);

        let mut reader = BitReader::new(fit::Value::UInt16(0xAAAA));
        assert_eq!(reader.read_bits(32), None);
    }
}
