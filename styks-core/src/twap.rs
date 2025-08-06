use odra::prelude::*;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum TWAPError {
    WindowCannotBeZero,
    ToleranceMustBeLessThanWindow,
}

pub struct TWAP {
    window: u64,
    tolerance: u64,
    values: Vec<Option<u128>>,
}

impl TWAP {
    pub fn new(window: u64, tolerance: u64) -> Result<Self, TWAPError> {
        if window == 0 {
            return Err(TWAPError::WindowCannotBeZero);
        }

        if tolerance >= window {
            return Err(TWAPError::ToleranceMustBeLessThanWindow);
        }

        Ok(Self {
            window,
            tolerance,
            values: Vec::new(),
        })
    }

    pub fn add_value(&mut self, value: u128) {
        self.values.push(Some(value));
    }

    pub fn add_missed_value(&mut self) {
        self.values.push(None);
    }

    pub fn calculate(&self) -> Option<u128> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twap() {
        fn add_value(twap: &mut TWAP, value: u128, expected: Option<u128>) {
            twap.add_value(value);
            assert_eq!(twap.calculate(), expected);
        }

        fn missed_value(twap: &mut TWAP, expected: Option<u128>) {
            twap.add_missed_value();
            assert_eq!(twap.calculate(), expected);
        }


        // Test the TWAP with 0 missed values tolerance.
        let mut twap = TWAP::new(3, 0).unwrap();
        add_value(&mut twap, 100, None);
        add_value(&mut twap, 200, None);
        add_value(&mut twap, 300, Some(200));
        missed_value(&mut twap, None);
        add_value(&mut twap, 400, None);
        add_value(&mut twap, 500, None);
        add_value(&mut twap, 600, Some(500));

        // Test the TWAP with 1 missed values tolerance.
        let mut twap = TWAP::new(3, 1).unwrap();
        add_value(&mut twap, 100, None);
        add_value(&mut twap, 200, Some(150));
        add_value(&mut twap, 300, Some(200));
        missed_value(&mut twap, Some(200));
        missed_value(&mut twap, None);
        add_value(&mut twap, 400, None);
        add_value(&mut twap, 500, Some(450));

        // Test the TWAP with 2 missed values tolerance.
        let mut twap = TWAP::new(3, 2).unwrap();
        add_value(&mut twap, 100, Some(100));
        add_value(&mut twap, 200, Some(150));
        missed_value(&mut twap, Some(150));
        missed_value(&mut twap, Some(150));
        missed_value(&mut twap, None);
    }
}
