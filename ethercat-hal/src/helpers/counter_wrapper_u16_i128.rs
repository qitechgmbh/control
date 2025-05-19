const U16_MAX: i128 = std::u16::MAX as i128;

/// This is a wrapper for a counter that stores a u16 that frequently overflows or underflows
///
/// We convert the overflows and underflows to an i128 value for easier calculations.
///
/// When overriding the counter we don't set the valeu direclty but schedula it wiht the [`push_override`] so it can be synced with [`pop_override`] to an EtherCAT device.
#[derive(Debug)]
pub struct CounterWrapperU16U128 {
    counter: i128,
    set_counter: Option<i128>,
}

impl CounterWrapperU16U128 {
    pub fn new(counter: i128) -> Self {
        Self {
            counter,
            set_counter: None,
        }
    }

    pub fn update(&mut self, counter: u16, counter_underflow: bool, counter_overflow: bool) {
        self.counter +=
            counter_u16_to_i128(self.counter, counter, counter_underflow, counter_overflow);
    }

    pub fn current(&self) -> i128 {
        self.counter
    }

    /// Schedules a counter override
    ///
    /// The value is only set when `pop_set` is called.
    pub fn push_override(&mut self, new_counter: i128) {
        self.set_counter = Some(new_counter);
    }

    /// Return the override value as an u16 and overrides the current counter.
    pub fn pop_override(&mut self) -> Option<u16> {
        match self.set_counter {
            Some(counter) => {
                // set our coutner to the new value
                self.counter = counter;

                // Convert the i128 counter to u16
                let u16_counter = set_counter_u16_to_i128(counter);

                // Clear the set counter
                self.set_counter = None;

                Some(u16_counter)
            }
            None => None, // No set counter available
        }
    }

    /// Returns the override value as an i128
    pub fn get_override(&self) -> Option<i128> {
        self.set_counter
    }
}

fn counter_u16_to_i128(
    last_counter: i128,
    counter: u16,
    counter_underflow: bool,
    counter_overflow: bool,
) -> i128 {
    if counter_overflow {
        // Counter overflowed: it went from MAX_VALUE to 0
        // The actual change is: (counter) - MAX_VALUE - 1 + MAX_VALUE + 1
        // Simplified to: counter + 1
        return (counter as i128) - last_counter + U16_MAX + 1;
    } else if counter_underflow {
        // Counter underflowed: it went from 0 to MAX_VALUE
        // The actual change is: (counter) - 0 - (MAX_VALUE + 1)
        // Simplified to: counter - (MAX_VALUE + 1)
        return (counter as i128) - last_counter - (U16_MAX + 1);
    } else {
        // Normal case: just return the difference
        return (counter as i128) - last_counter;
    }
}

fn set_counter_u16_to_i128(new_counter: i128) -> u16 {
    // Use modulo arithmetic to handle both positive and negative values
    let modulo = (new_counter % (U16_MAX + 1)) as i32;

    if modulo < 0 {
        // If negative, wrap around from the end
        return (modulo + (U16_MAX + 1) as i32) as u16;
    } else {
        return modulo as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_counter_u16_to_i128() {
        // Test normal values within u16 range
        assert_eq!(set_counter_u16_to_i128(0), 0);
        assert_eq!(set_counter_u16_to_i128(1000), 1000);
        assert_eq!(set_counter_u16_to_i128(65535), 65535);

        // Test overflow values
        assert_eq!(set_counter_u16_to_i128(65536), 0);
        assert_eq!(set_counter_u16_to_i128(65537), 1);
        assert_eq!(set_counter_u16_to_i128(65536 + 1000), 1000);
        assert_eq!(set_counter_u16_to_i128(65536 * 2 + 5), 5);

        // Test negative values
        assert_eq!(set_counter_u16_to_i128(-1), 65535);
        assert_eq!(set_counter_u16_to_i128(-2), 65534);
        assert_eq!(set_counter_u16_to_i128(-1000), 64536);
        assert_eq!(set_counter_u16_to_i128(-65536), 0);
        assert_eq!(set_counter_u16_to_i128(-65537), 65535);

        // Test large values
        assert_eq!(set_counter_u16_to_i128(65536 * 1000 + 42), 42);
        assert_eq!(set_counter_u16_to_i128(-65536 * 1000 - 42), 65494);
    }

    #[test]
    fn test_set_counter_u16_to_i128_roundtrip() {
        // Test that converting to u16 and back to i128 gives consistent results
        for i in -100..100 {
            let u16_value = set_counter_u16_to_i128(i);
            let i128_value = i % 65536;
            let normalized_i128 = if i128_value < 0 {
                i128_value + 65536
            } else {
                i128_value
            };
            assert_eq!(u16_value, normalized_i128 as u16);
        }
    }

    #[test]
    fn test_normal_increment() {
        // Simple increment with no overflow/underflow
        assert_eq!(counter_u16_to_i128(100, 105, false, false), 5);
    }

    #[test]
    fn test_normal_decrement() {
        // Simple decrement with no overflow/underflow
        assert_eq!(counter_u16_to_i128(100, 95, false, false), -5);
    }

    #[test]
    fn test_overflow() {
        // Test overflow: counter went from 65535 to 2
        // The difference should be 2 - 65535 + 65536 = 3
        assert_eq!(counter_u16_to_i128(65535, 2, false, true), 3);

        // Test overflow: counter went from 65535 to 0
        // The difference should be 0 - 65535 + 65536 = 1
        assert_eq!(counter_u16_to_i128(65535, 0, false, true), 1);
    }

    #[test]
    fn test_underflow() {
        // Test underflow: counter went from 0 to 65533
        // The difference should be 65533 - 0 - 65536 = -3
        assert_eq!(counter_u16_to_i128(0, 65533, true, false), -3);

        // Test underflow: counter went from 0 to 65535
        // The difference should be 65535 - 0 - 65536 = -1
        assert_eq!(counter_u16_to_i128(0, 65535, true, false), -1);
    }

    #[test]
    fn test_large_changes() {
        // Test a large forward jump (no overflow)
        assert_eq!(counter_u16_to_i128(1000, 5000, false, false), 4000);

        // Test a large backward jump (no underflow)
        assert_eq!(counter_u16_to_i128(5000, 1000, false, false), -4000);
    }

    #[test]
    fn test_edge_cases() {
        // Test when last_counter is already large (i128)
        let large_last_counter: i128 = 1_000_000_000_000;
        assert_eq!(
            counter_u16_to_i128(large_last_counter, 100, false, false),
            100 - large_last_counter
        );

        // Test consecutive overflows
        // First overflow: 65535 -> 10
        let diff1 = counter_u16_to_i128(65535, 10, false, true);
        assert_eq!(diff1, 11);

        // Second overflow: using the new position (65535 + 11 = 65546) -> 20
        let diff2 = counter_u16_to_i128(65535 + 11, 20, false, true);
        assert_eq!(diff2, 10);
    }

    #[test]
    fn test_integration() {
        // Simulate a full rotation: 0 -> 65535 -> 0
        let mut position: i128 = 0;

        // Increment to 65530
        position += counter_u16_to_i128(position, 65530, false, false);
        assert_eq!(position, 65530);

        // Increment to 65535
        position += counter_u16_to_i128(position, 65535, false, false);
        assert_eq!(position, 65535);

        // Overflow to 5
        position += counter_u16_to_i128(position, 5, false, true);
        assert_eq!(position, 65541); // 65535 + 6

        // Normal decrement to 65530 (no underflow)
        position += counter_u16_to_i128(position, 65530, false, false);
        assert_eq!(position, 65530);

        // Now test a true underflow: position 5 -> 65535
        position = 5; // Reset position to 5
        position += counter_u16_to_i128(position, 65535, true, false);
        assert_eq!(position, -1); // 5 + (65535 - 5 - 65536) = 5 - 6 = -1
    }
}
