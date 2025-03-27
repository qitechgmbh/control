/// Translates the encoder value to a larger space and handle underflow/overflow situations
#[derive(Debug)]
pub struct Encoder {
    pub position: i64,
    pub last_encoder_position: u32,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            position: 0,
            last_encoder_position: 0,
        }
    }

    pub fn update(&mut self, counter_value: u32, counter_overflow: bool, counter_underflow: bool) {
        // Handle overflow case
        if counter_overflow {
            // When overflow occurs, the counter wraps from MAX to 0
            // Calculate how much it moved past MAX
            let delta = counter_value as i64;
            // Add the remaining distance from last position to MAX, plus the overflow amount
            self.position += (u32::MAX - self.last_encoder_position) as i64 + 1 + delta;
        }
        // Handle underflow case
        else if counter_underflow {
            // When underflow occurs, the counter wraps from 0 to MAX
            // Calculate how much it moved below 0
            let delta = (u32::MAX - counter_value) as i64;
            // Subtract the distance from last position to 0, plus the underflow amount
            self.position -= self.last_encoder_position as i64 + 1 + delta;
        }
        // Normal case - no overflow/underflow
        else {
            // Calculate the signed difference between current and last position
            let delta = counter_value as i64 - self.last_encoder_position as i64;
            // Update the position
            self.position += delta;
        }

        // Update the last encoder position for next time
        self.last_encoder_position = counter_value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the position tracker correctly handles normal forward movement
    /// without any overflow conditions.
    ///
    /// This test verifies:
    /// 1. Initial state is zeroed
    /// 2. Moving forward updates both position and last_encoder_position
    /// 3. Consecutive forward movements accumulate correctly
    #[test]
    fn test_normal_forward_movement() {
        let mut tracker = Encoder::new();

        // Initial state
        assert_eq!(tracker.position, 0);
        assert_eq!(tracker.last_encoder_position, 0);

        // Move forward by 100
        tracker.update(100, false, false);
        assert_eq!(tracker.position, 100);
        assert_eq!(tracker.last_encoder_position, 100);

        // Move forward by another 50
        tracker.update(150, false, false);
        assert_eq!(tracker.position, 150);
        assert_eq!(tracker.last_encoder_position, 150);
    }

    /// Tests that the position tracker correctly handles normal backward movement
    /// without any underflow conditions.
    ///
    /// This test verifies:
    /// 1. Starting from a non-zero position
    /// 2. Moving backward updates both position and last_encoder_position
    /// 3. Consecutive backward movements accumulate correctly
    #[test]
    fn test_normal_backward_movement() {
        let mut tracker = Encoder::new();

        // Start at position 1000
        tracker.position = 1000;
        tracker.last_encoder_position = 1000;

        // Move backward by 100
        tracker.update(900, false, false);
        assert_eq!(tracker.position, 900);
        assert_eq!(tracker.last_encoder_position, 900);

        // Move backward by another 50
        tracker.update(850, false, false);
        assert_eq!(tracker.position, 850);
        assert_eq!(tracker.last_encoder_position, 850);
    }

    /// Tests that the position tracker correctly handles overflow conditions
    /// where the encoder counter wraps from near its maximum value to a small value.
    ///
    /// This test verifies:
    /// 1. When overflow flag is set, the tracker correctly calculates the true movement
    /// 2. Position is updated to account for the full distance traveled
    /// 3. last_encoder_position is updated to the new counter value
    #[test]
    fn test_overflow() {
        let mut tracker = Encoder::new();

        // Set position near the maximum u32 value
        tracker.position = 100;
        tracker.last_encoder_position = u32::MAX - 10;

        // Overflow: counter goes from (MAX-10) to 5
        // This means we moved forward by 10 steps to MAX, then 1 step to overflow,
        // then 5 more steps to reach the current position
        tracker.update(5, true, false);

        // Expected: position += (10 + 1 + 5) = position += 16
        assert_eq!(tracker.position, 116);
        assert_eq!(tracker.last_encoder_position, 5);
    }

    /// Tests that the position tracker correctly handles underflow conditions
    /// where the encoder counter wraps from near zero to a value near its maximum.
    ///
    /// This test verifies:
    /// 1. When underflow flag is set, the tracker correctly calculates the true movement
    /// 2. Position is updated to account for the full distance traveled (in negative direction)
    /// 3. last_encoder_position is updated to the new counter value
    #[test]
    fn test_underflow() {
        let mut tracker = Encoder::new();

        // Set position near zero
        tracker.position = 100;
        tracker.last_encoder_position = 10;

        // Underflow: counter goes from 10 to (MAX-5)
        // This means we moved backward by 10 steps to 0, then 1 step to underflow,
        // then 5 more steps to reach the current position
        tracker.update(u32::MAX - 5, false, true);

        // Expected: position -= (10 + 1 + 5) = position -= 16
        assert_eq!(tracker.position, 84);
        assert_eq!(tracker.last_encoder_position, u32::MAX - 5);
    }

    /// Tests that the position tracker maintains correct behavior across
    /// a sequence of different operations including normal movement,
    /// overflow, and underflow.
    ///
    /// This test verifies:
    /// 1. The tracker maintains state correctly between operations
    /// 2. Different types of movements can be handled in sequence
    /// 3. Position calculations remain accurate throughout various scenarios
    #[test]
    fn test_multiple_operations() {
        let mut tracker = Encoder::new();

        // Normal forward
        tracker.update(100, false, false);
        assert_eq!(tracker.position, 100);

        // Normal backward
        tracker.update(50, false, false);
        assert_eq!(tracker.position, 50);

        // Overflow
        tracker.last_encoder_position = u32::MAX - 5;
        tracker.position = 50; // Reset for clarity
        tracker.update(10, true, false);
        assert_eq!(tracker.position, 66); // 50 + (5 + 1 + 10)

        // Underflow
        tracker.last_encoder_position = 5;
        tracker.position = 100; // Reset for clarity
        tracker.update(u32::MAX - 10, false, true);
        assert_eq!(tracker.position, 84); // 100 - (5 + 1 + 10)
    }

    /// Tests that the position tracker can handle multiple overflow and underflow
    /// events, maintaining an accurate position count that exceeds the u32 range.
    ///
    /// This test verifies:
    /// 1. Multiple overflow events accumulate correctly in the i64 position
    /// 2. Multiple underflow events correctly subtract from the position
    /// 3. The i64 position field properly extends the range beyond u32 limits
    #[test]
    fn test_large_movements() {
        let mut tracker = Encoder::new();

        // Test tracking beyond u32 limits with multiple overflows
        for _ in 0..5 {
            // Simulate overflow: move from (MAX-100) to 100
            tracker.last_encoder_position = u32::MAX - 100;
            tracker.update(100, true, false);

            // Each iteration adds (100 + 1 + 100) = 201 to position
            // We move 100 steps to MAX, 1 step to overflow, and 100 more steps to reach 100
        }

        // After 5 iterations, position should be 5 * 201 = 1005
        assert_eq!(tracker.position, 1005);

        // Test tracking with multiple underflows
        for _ in 0..3 {
            // Simulate underflow: move from 100 to (MAX-100)
            tracker.last_encoder_position = 100;
            tracker.update(u32::MAX - 100, false, true);

            // Each iteration subtracts (100 + 1 + 100) = 201 from position
            // We move 100 steps to 0, 1 step to underflow, and 100 more steps to reach MAX-100
        }

        // Final position: 1005 - (3 * 201) = 1005 - 603 = 402
        assert_eq!(tracker.position, 402);
    }
}
