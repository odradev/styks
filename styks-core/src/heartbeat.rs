#[derive(Debug)]
pub enum HeartbeatError {
    TolaranceShouldBeLessThanHalfOfInterval,
    IntervalShouldBeGreaterThanZero,
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatWindow {
    pub start: u64,
    pub middle: u64,
    pub end: u64,
}

impl HeartbeatWindow {
    pub fn is_in_window(&self, time: u64) -> bool {
        time >= self.start && time <= self.end
    }

    pub fn time_till_middle(&self, current_time: u64) -> u64 {
        if self.middle > current_time {
            self.middle - current_time
        } else {
            0
        }
    }
}

// This is a triplet of windows: (previous, current, next).
// Previous may be None if current_time is 0.
// Current may be None if current_time is not within the time window.
#[derive(Debug, PartialEq)]
pub struct HeartbeatStatus {
    pub previous: Option<HeartbeatWindow>,
    pub current: Option<HeartbeatWindow>,
    pub next: HeartbeatWindow,
}

#[derive(Debug)]
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

    // Note: Method implemented by Cloude Sonnet 4.
    pub fn current_state(&self) -> HeartbeatStatus {
        // Heartbeats occur at multiples of interval: 0, interval, 2*interval, etc.
        // Each heartbeat has a tolerance window: [heartbeat_time - tolerance, heartbeat_time + tolerance]
        // But the start time cannot be negative, so it's max(0, heartbeat_time - tolerance)

        // Find all possible heartbeat times around current_time
        let heartbeat_index = self.current_time / self.interval;

        // Check current heartbeat (at heartbeat_index * interval)
        let current_heartbeat_time = heartbeat_index * self.interval;
        let current_start = if current_heartbeat_time >= self.tolerance {
            current_heartbeat_time - self.tolerance
        } else {
            0
        };
        let current_end = current_heartbeat_time + self.tolerance;

        // Check next heartbeat (at (heartbeat_index + 1) * interval)
        let next_heartbeat_time = (heartbeat_index + 1) * self.interval;
        let next_start = if next_heartbeat_time >= self.tolerance {
            next_heartbeat_time - self.tolerance
        } else {
            0
        };
        let next_end = next_heartbeat_time + self.tolerance;

        // Determine which window we're in
        let (current, previous, next) = if self.current_time >= current_start
            && self.current_time <= current_end
        {
            // We're in the current heartbeat window
            let current_window = HeartbeatWindow {
                start: current_start,
                middle: current_heartbeat_time,
                end: current_end,
            };

            let previous_window = if current_heartbeat_time > 0 {
                let prev_time = current_heartbeat_time - self.interval;
                let prev_start = if prev_time >= self.tolerance {
                    prev_time - self.tolerance
                } else {
                    0
                };
                Some(HeartbeatWindow {
                    start: prev_start,
                    middle: prev_time,
                    end: prev_time + self.tolerance,
                })
            } else {
                None
            };

            let next_window = HeartbeatWindow {
                start: next_start,
                middle: next_heartbeat_time,
                end: next_end,
            };

            (Some(current_window), previous_window, next_window)
        } else if self.current_time >= next_start && self.current_time <= next_end {
            // We're in the next heartbeat window
            let current_window = HeartbeatWindow {
                start: next_start,
                middle: next_heartbeat_time,
                end: next_end,
            };

            let previous_window = Some(HeartbeatWindow {
                start: current_start,
                middle: current_heartbeat_time,
                end: current_end,
            });

            let next_next_time = (heartbeat_index + 2) * self.interval;
            let next_next_start = if next_next_time >= self.tolerance {
                next_next_time - self.tolerance
            } else {
                0
            };
            let next_window = HeartbeatWindow {
                start: next_next_start,
                middle: next_next_time,
                end: next_next_time + self.tolerance,
            };

            (Some(current_window), previous_window, next_window)
        } else {
            // We're not in any heartbeat window
            // The previous window is the most recent heartbeat that has passed
            let previous_window = if current_heartbeat_time > 0 || self.current_time > current_end {
                Some(HeartbeatWindow {
                    start: current_start,
                    middle: current_heartbeat_time,
                    end: current_end,
                })
            } else {
                None
            };

            let next_window = HeartbeatWindow {
                start: next_start,
                middle: next_heartbeat_time,
                end: next_end,
            };

            (None, previous_window, next_window)
        };

        HeartbeatStatus {
            previous,
            current,
            next,
        }
    }

    pub fn count_missed_heartbeats_since(&self, last_heartbeat_time: u64) -> u64 {
        // Calculate the index of the last recorded heartbeat
        let last_index = last_heartbeat_time / self.interval;
        // Determine the highest heartbeat index whose window has fully ended
        let max_index = if self.current_time > self.tolerance {
            (self.current_time - self.tolerance - 1) / self.interval
        } else {
            0
        };
        // Number of missed heartbeats is the difference, saturating at zero
        max_index.saturating_sub(last_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_previous_heartbeat_time() {
        fn check(
            current_time: u64,
            prev: Option<(u64, u64, u64)>,
            current: Option<(u64, u64, u64)>,
            next: (u64, u64, u64),
        ) {
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
        check(
            190,
            Some((90, 100, 110)),
            Some((190, 200, 210)),
            (290, 300, 310),
        );
        check(
            210,
            Some((90, 100, 110)),
            Some((190, 200, 210)),
            (290, 300, 310),
        );

        // When is after 4th heartbeat window.
        check(311, Some((290, 300, 310)), None, (390, 400, 410));
        check(389, Some((290, 300, 310)), None, (390, 400, 410));
    }

    #[test]
    fn test_count_missed_heartbeats_since() {
        fn check(
            last_heartbeat_time: u64,
            current_time: u64,
            expected_missed: u64,
        ) {
            let heartbeat = Heartbeat::new(current_time, 100, 10).unwrap();
            let missed = heartbeat.count_missed_heartbeats_since(last_heartbeat_time);
            let msg = format!(
                "Expected {} missed heartbeats, but got {}. Last time: {}, Current time: {}",
                expected_missed, missed, last_heartbeat_time, current_time
            );
            assert_eq!(missed, expected_missed, "{}", msg);
        }

        check(0, 0, 0);
        check(0, 50, 0);
        check(0, 89, 0);
        check(0, 100, 0);
        check(0, 110, 0);
        check(0, 111, 1);
        check(0, 200, 1);
        check(0, 210, 1);
        check(100, 300, 1);
        check(100, 420, 3);
    }
}
