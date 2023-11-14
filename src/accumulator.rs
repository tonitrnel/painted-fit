use std::collections::HashMap;

pub(crate) struct AccumulatedField {
    accumulated_value: usize,
    last_value: usize,
}

impl AccumulatedField {
    pub(crate) fn new(value: usize) -> Self {
        AccumulatedField {
            accumulated_value: value,
            last_value: value,
        }
    }
    pub(crate) fn accumulate(&mut self, value: usize, bits: u8) -> usize {
        let mask = (1 << bits) - 1;
        self.accumulated_value += (value - self.last_value) & mask;
        self.last_value = value;
        self.accumulated_value
    }
}

pub(crate) struct Accumulator {
    messages: HashMap<u16, HashMap<u8, AccumulatedField>>,
}

impl Accumulator {
    pub(crate) fn new() -> Accumulator {
        Accumulator {
            messages: HashMap::new(),
        }
    }
    pub(crate) fn add(&mut self, msg_no: u16, field_no: u8, value: usize) {
        if let Some(fields) = self.messages.get_mut(&msg_no) {
            fields.insert(field_no, AccumulatedField::new(value));
        } else {
            let mut map = HashMap::new();
            map.insert(field_no, AccumulatedField::new(value));
            self.messages.insert(msg_no, map);
        }
    }
    pub(crate) fn accumulate(
        &mut self,
        msg_no: u16,
        field_no: u8,
        value: usize,
        bits: u8,
    ) -> usize {
        self.messages
            .get_mut(&msg_no)
            .and_then(|fields| fields.get_mut(&field_no))
            .map(|field| field.accumulate(value, bits))
            .unwrap_or(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut accumulator = Accumulator::new();
        accumulator.add(0, 0, 0);
        assert_eq!(accumulator.accumulate(0, 0, 1, 8), 1);

        accumulator.add(0, 0, 0);
        assert_eq!(accumulator.accumulate(0, 0, 2, 8), 2);

        accumulator.add(0, 0, 0);
        assert_eq!(accumulator.accumulate(0, 0, 3, 8), 3);

        accumulator.add(0, 0, 0);
        assert_eq!(accumulator.accumulate(0, 0, 4, 8), 4);
    }
}
