use crate::fit;

pub(crate) struct BitReader {
    bytes: Vec<usize>,
    per_size: usize,
    consumed: usize,
    total: usize,
}

impl BitReader {
    pub(crate) fn new(value: fit::Value) -> Self {
        let bytes = if let fit::Value::Array(arr) = value {
            arr
        } else {
            vec![value]
        };
        let per_size = (fit::BaseType::from(&bytes[0]).size() * 8) as usize;
        let bytes = bytes
            .iter()
            .map(|it| it.try_as_usize().unwrap())
            .collect::<Vec<_>>();
        Self {
            per_size,
            consumed: 0,
            total: per_size * bytes.len(),
            bytes,
        }
    }
    pub(crate) fn available(&self) -> bool {
        self.consumed < self.total
    }
    pub(crate) fn next(&mut self) -> Option<u8> {
        if !self.available() {
            // println!(
            //     "bytes = {:?}, consumed = {}, total = {}",
            //     self.bytes, self.consumed, self.total
            // );
            return None;
        }
        let bit = (self.bytes[self.consumed / self.per_size] >> (self.consumed % self.per_size)
            & 0x01) as u8;
        self.consumed += 1;
        Some(bit)
    }
    pub(crate) fn read_bits(&mut self, len: usize) -> Option<usize> {
        // println!("read len = {}", len);
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

    #[derive(Debug)]
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
            assert_eq!(reader.consumed, index);
            assert!(reader.available());
            assert_eq!(Some(*expected), reader.next());
            assert_eq!(reader.consumed, index + 1);
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
        for (idx, scenario) in scenarios.iter().enumerate() {
            let mut reader = BitReader::new(scenario.to_value());
            for (index, value) in scenario.values.iter().enumerate() {
                assert_eq!(
                    Some(*value),
                    reader.read_bits(scenario.n_bits_to_read[index]),
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
