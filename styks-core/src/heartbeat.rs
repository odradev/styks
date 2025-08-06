#[cfg_attr(test, derive(Debug))]
pub enum HeartbeatError {
    TolaranceShouldBeLessThanHalfOfInterval,
    IntervalShouldBeGreaterThanZero,
}

#[cfg_attr(test, derive(Debug))]
pub struct Heartbeat {
    current_time: u64,
    interval: u64,
    tolerance: u64,
}

impl Heartbeat {
    pub fn new(current_time: u64, interval: u64, tolerance: u64) -> Result<Self, HeartbeatError> {
        // Check if interval is greater than zero.
        if interval == 0 {
            return Err(HeartbeatError::IntervalShouldBeGreaterThanZero);
        }

        // Check if tolerance is less than half of the interval.
        // This makes sure tolarance periods from two consecutive heartbeats do not overlap.
        if tolerance >= interval / 2 {
            return Err(HeartbeatError::TolaranceShouldBeLessThanHalfOfInterval);
        }
        
        Ok(Heartbeat {
            current_time,
            interval,
            tolerance,
        })
    }

    // Previous heartbeat time.
    // It doesn't exist if the current_time=0.
    pub fn previous_heartbeat(&self) -> Option<u64> {
        if self.current_time == 0 {
            return None;
        }

        if self.current_time % self.interval == 0 {
            return Some(self.current_time - self.interval);
        }

        Some(self.current_time - self.current_time % self.interval)
    }

    // Next heartbeat time.
    pub fn next_heartbeat(&self) -> u64 {
        if self.current_time == 0 {
            return self.interval;
        }

        if self.current_time % self.interval == 0 {
            return self.current_time + self.interval;
        }

        let previous_heartbeat = self.current_time - self.current_time % self.interval;
        previous_heartbeat + self.interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_previous_heartbeat_time() {
        fn check_prev(current_time: u64, expected: Option<u64>) {
            let heartbeat = Heartbeat::new(current_time, 100, 10).unwrap();
            let msg = format!(
                "current_time: {}, expected: {:?}, actual: {:?}",
                current_time,
                expected,
                heartbeat.previous_heartbeat()
            );
            assert_eq!(heartbeat.previous_heartbeat(), expected, "{}", msg);
        }

        check_prev(0, None);
        check_prev(99, Some(0));
        check_prev(100, Some(0));
        check_prev(101, Some(100));
    }

    #[test]
    fn test_next_heartbeat_time() {
        fn check_next(current_time: u64, expected: u64) {
            let heartbeat = Heartbeat::new(current_time, 100, 10).unwrap();
            let msg = format!(
                "current_time: {}, expected: {}, actual: {}",
                current_time,
                expected,
                heartbeat.next_heartbeat()
            );
            assert_eq!(heartbeat.next_heartbeat(), expected, "{}", msg);
        }

        check_next(0, 100);
        check_next(99, 100);
        check_next(100, 200);
    }
}