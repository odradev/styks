use odra::prelude::*;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum TWAPError {
    WindowCannotBeZero,
    ToleranceMustBeLessThanWindow,
    TooManyValues,
}

pub struct TWAP {
    window: u32,
    tolerance: u32,
    storage: VecDeque<Option<u64>>,
}

impl TWAP {
    pub fn new(window: u32, tolerance: u32, values: Vec<Option<u64>>) -> Result<Self, TWAPError> {
        if window == 0 {
            return Err(TWAPError::WindowCannotBeZero);
        }

        if tolerance >= window {
            return Err(TWAPError::ToleranceMustBeLessThanWindow);
        }

        if values.len() > window as usize {
            return Err(TWAPError::TooManyValues);
        }

        let mut storage = VecDeque::with_capacity(window as usize);
        storage.extend(values);

        Ok(Self {
            window,
            tolerance,
            storage,
        })
    }

    fn push_to_storage(&mut self, value: Option<u64>) {
        if self.storage.len() == self.window as usize {
            self.storage.pop_front();
        }
        self.storage.push_back(value);
    }

    pub fn add_value(&mut self, value: u64) {
        self.push_to_storage(Some(value));
    }

    pub fn add_missed_value(&mut self) {
        self.push_to_storage(None);
    }

    pub fn calculate(&self) -> Option<u64> {
        let current_values: Vec<u64> = self.storage.iter().filter_map(|&v| v).collect();
        let current_values_count = current_values.len() as u64;

        let required_values = (self.window - self.tolerance) as usize;

        if current_values.len() < required_values {
            return None;
        }

        let sum: u64 = current_values.iter().sum();
        Some(sum / current_values_count)
    }

    pub fn values(&self) -> Vec<Option<u64>> {
        self.storage.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twap() {
        fn add_value(twap: &mut TWAP, value: u64, expected: Option<u64>) {
            twap.add_value(value);
            assert_eq!(twap.calculate(), expected);
        }

        fn missed_value(twap: &mut TWAP, expected: Option<u64>) {
            twap.add_missed_value();
            assert_eq!(twap.calculate(), expected);
        }

        // Test the TWAP with 0 missed values tolerance.
        let mut twap = TWAP::new(3, 0, vec![]).unwrap();
        add_value(&mut twap, 100, None);
        add_value(&mut twap, 200, None);
        add_value(&mut twap, 300, Some(200));
        missed_value(&mut twap, None);
        add_value(&mut twap, 400, None);
        add_value(&mut twap, 500, None);
        add_value(&mut twap, 600, Some(500));

        // Test the TWAP with 1 missed values tolerance.
        let mut twap = TWAP::new(3, 1, vec![]).unwrap();
        add_value(&mut twap, 100, None);
        add_value(&mut twap, 200, Some(150));
        add_value(&mut twap, 300, Some(200));
        missed_value(&mut twap, Some(250));
        missed_value(&mut twap, None);
        add_value(&mut twap, 400, None);
        add_value(&mut twap, 500, Some(450));

        // Test the TWAP with 2 missed values tolerance.
        let mut twap = TWAP::new(3, 2, vec![]).unwrap();
        add_value(&mut twap, 100, Some(100));
        add_value(&mut twap, 200, Some(150));
        missed_value(&mut twap, Some(150));
        missed_value(&mut twap, Some(200));
        missed_value(&mut twap, None);
        missed_value(&mut twap, None);
        add_value(&mut twap, 400, Some(400));
    }

    #[test]
    fn test_getting_values() {
        let mut twap = TWAP::new(2, 0, vec![]).unwrap();
        assert_eq!(twap.values(), vec![]);
        twap.add_value(100);
        assert_eq!(twap.values(), vec![Some(100)]);
        twap.add_value(200);
        assert_eq!(twap.values(), vec![Some(100), Some(200)]);
        twap.add_value(300);
        assert_eq!(twap.values(), vec![Some(200), Some(300)]);
        twap.add_missed_value();
        assert_eq!(twap.values(), vec![Some(300), None]);

        let mut twap = TWAP::new(3, 1, vec![Some(100), Some(200)]).unwrap();
        assert_eq!(twap.values(), vec![Some(100), Some(200)]);
        twap.add_value(300);
        assert_eq!(twap.values(), vec![Some(100), Some(200), Some(300)]);
        twap.add_missed_value();
        assert_eq!(twap.values(), vec![Some(200), Some(300), None]);
    }
}
