#[cfg_attr(test, derive(Debug))]
pub enum HeartbeatError {
    TolaranceShouldBeLessThanHalfOfInterval,
    IntervalShouldBeGreaterThanZero,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct HeartbeatWindow {
    start: u64,
    middle: u64,
    end: u64,
}

// This is a triplet of windows: (previous, current, next).
// Previous may be None if current_time is 0.
// Current may be None if current_time is not within the time window.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct HeartbeatStatus {
    previous: Option<HeartbeatWindow>,
    current: Option<HeartbeatWindow>,
    next: HeartbeatWindow,
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

    pub fn current_state(&self) -> HeartbeatStatus {
        todo!()
    }
        
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_previous_heartbeat_time() {
        fn check(current_time: u64, prev: Option<(u64, u64, u64)>, current: Option<(u64, u64, u64)>, next: (u64, u64, u64)) {
            let heartbeat = Heartbeat::new(current_time, 100, 10).unwrap();
            let state = heartbeat.current_state();
            
            // Check previous heartbeat.
            if let Some((start, middle, end)) = prev {
                let prev = HeartbeatWindow { start, middle, end };
                assert_eq!(state.previous, Some(prev));
            } else {
                assert_eq!(state.previous, None);
            }

            // Check current heartbeat.
            if let Some((start, middle, end)) = current {
                let current = HeartbeatWindow { start, middle, end };
                assert_eq!(state.current, Some(current));
            } else {
                assert_eq!(state.current, None);
            }

            // Check next heartbeat.
            let next = HeartbeatWindow {
                start: next.0,
                middle: next.1,
                end: next.2,
            };
            assert_eq!(state.next, next);
        }

        // When current_time is 0, there's no previous heartbeat
        check(0, None, Some((0, 0, 10)), (90, 100, 110));
        
        // When time is within the first heartbeat.
        check(5, None, Some((0, 0, 10)), (90, 100, 110));

        // When time is just after the first heartbeat, current is None.
        check(15, Some((0, 0, 10)), None, (90, 100, 110));

        // When time is within the second heartbeat.
        check(105, Some((0, 0, 10)), Some((90, 100, 110)), (190, 200, 210));

        // When time is at the edge of edge of the window.
        check(190, Some((90, 100, 110)), Some((190, 200, 210)), (290, 300, 310));
        check(210, Some((90, 100, 110)), Some((190, 200, 210)), (290, 300, 310));

        // When is after 4th heartbeat window.
        check(311, Some((290, 300, 310)), None, (390, 400, 410));
        check(389, Some((290, 300, 310)), None, (390, 400, 410));

    }
}