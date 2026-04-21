use anyhow::Error;

use crate::io::{
    stepper_velocity_wago_750_671::StepperVelocityWago750671, traverse_axis::TraverseStepperAxis,
};

/// Traverse-focused adapter around the raw 750-671 speed wrapper.
///
/// The goal is to preserve an EL70x1-like application contract:
/// - speeds are expressed in full-steps/second
/// - position is exposed as a logical application position
/// - zeroing is logical, not a raw device rewrite
///
/// WAGO-specific mode and stop controls remain available here so the separate
/// `wago_winder` machine can keep its own lifecycle and debugging behavior.
#[derive(Debug)]
pub struct StepperVelocityWago750671Traverse {
    axis: StepperVelocityWago750671,
    /// Logical traverse position should increase as the carriage moves away
    /// from the home switch, matching the `winder2` application contract.
    /// On this WAGO machine the raw 671 count moves the opposite way, so we
    /// normalize that here.
    position_sign: i8,
}

impl StepperVelocityWago750671Traverse {
    pub fn new(axis: StepperVelocityWago750671) -> Self {
        Self {
            axis,
            position_sign: -1,
        }
    }

    /// Applies the baseline semantics expected by the traverse controller.
    ///
    /// This does not try to perfectly reproduce EL7031 physical feel.
    /// It only aligns the wrapper contract and default scaling.
    pub fn configure_for_traverse_contract(
        &mut self,
        freq_range_sel: u8,
        acc_range_sel: u8,
        acceleration: u16,
    ) {
        self.axis.set_motor_full_steps_per_rev(200);
        self.axis.set_microsteps_per_full_step(64);
        self.axis.set_direction_multiplier(1);
        self.axis.set_speed_scale(1.0);
        self.axis.set_freq_range_sel(freq_range_sel);
        self.axis.set_acc_range_sel(acc_range_sel);
        self.axis.set_acceleration(acceleration);
        self.axis.request_speed_mode();
        self.axis.clear_fast_stop();
    }

    pub fn inner(&self) -> &StepperVelocityWago750671 {
        &self.axis
    }

    pub fn inner_mut(&mut self) -> &mut StepperVelocityWago750671 {
        &mut self.axis
    }

    pub fn set_speed(&mut self, steps_per_second: f64) -> Result<(), Error> {
        self.axis.request_speed_mode();
        self.axis.set_speed(steps_per_second);
        Ok(())
    }

    pub fn get_speed(&self) -> i32 {
        self.axis.get_speed()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.axis.set_enabled(enabled);
    }

    pub fn is_enabled(&self) -> bool {
        self.axis.enabled
    }

    pub fn get_position(&self) -> i128 {
        self.axis.get_position() * i128::from(self.position_sign)
    }

    pub fn set_position(&mut self, position: i128) {
        self.axis
            .set_position(position * i128::from(self.position_sign));
    }

    pub fn get_actual_velocity_register(&self) -> i16 {
        self.axis.get_actual_velocity_register()
    }

    pub fn get_actual_speed_steps_per_second(&self) -> f64 {
        self.axis
            .velocity_register_to_steps_per_second(self.axis.get_actual_velocity_register())
    }

    pub fn set_acceleration(&mut self, acceleration: u16) {
        self.axis.set_acceleration(acceleration);
    }

    pub fn request_speed_mode(&mut self) {
        self.axis.request_speed_mode();
    }

    pub fn request_fast_stop(&mut self) {
        self.axis.request_fast_stop();
    }

    pub fn clear_fast_stop(&mut self) {
        self.axis.clear_fast_stop();
    }

    pub fn request_stop_no_ramp_mailbox(&mut self) {
        self.axis.request_stop_no_ramp_mailbox();
    }

    pub fn request_set_actual_position_mailbox(&mut self, position_microsteps: i128) {
        self.axis.request_set_actual_position_mailbox(
            position_microsteps * i128::from(self.position_sign),
        );
    }

    pub fn request_set_actual_position_zero_mailbox(&mut self) {
        self.axis.request_set_actual_position_zero_mailbox();
    }

    pub fn is_mailbox_active(&self) -> bool {
        self.axis.is_mailbox_active()
    }

    pub fn get_home_switch(&self) -> bool {
        self.axis.get_s3_bit1()
    }

    pub fn get_input1(&self) -> bool {
        self.axis.get_s3_bit0()
    }

    pub fn get_raw_position(&self) -> i128 {
        self.axis.get_raw_position()
    }

    pub fn get_normalized_raw_position(&self) -> i128 {
        self.axis.get_raw_position() * i128::from(self.position_sign)
    }

    pub fn get_status_byte1(&self) -> u8 {
        self.axis.get_status_byte1()
    }

    pub fn get_status_byte2(&self) -> u8 {
        self.axis.get_status_byte2()
    }

    pub fn get_status_byte3(&self) -> u8 {
        self.axis.get_status_byte3()
    }

    pub fn get_control_byte1(&self) -> u8 {
        self.axis.get_control_byte1()
    }

    pub fn get_control_byte2(&self) -> u8 {
        self.axis.get_control_byte2()
    }

    pub fn get_control_byte3(&self) -> u8 {
        self.axis.get_control_byte3()
    }

    pub fn get_target_acceleration(&self) -> u16 {
        self.axis.get_target_acceleration()
    }

    pub fn get_s1_bit3_speed_mode_ack(&self) -> bool {
        self.axis.get_s1_bit3_speed_mode_ack()
    }

    pub fn get_s1_bit5_reference_mode_ack(&self) -> bool {
        self.axis.get_s1_bit5_reference_mode_ack()
    }

    pub fn get_s2_reference_ok(&self) -> bool {
        self.axis.get_s2_reference_ok()
    }

    pub fn get_s2_busy(&self) -> bool {
        self.axis.get_s2_busy()
    }

    pub fn get_s3_bit0(&self) -> bool {
        self.axis.get_s3_bit0()
    }

    pub fn get_s3_bit1(&self) -> bool {
        self.axis.get_s3_bit1()
    }
}

impl TraverseStepperAxis for StepperVelocityWago750671Traverse {
    fn set_speed(&mut self, steps_per_second: f64) -> Result<(), Error> {
        Self::set_speed(self, steps_per_second)
    }

    fn set_enabled(&mut self, enabled: bool) {
        Self::set_enabled(self, enabled);
    }

    fn is_enabled(&self) -> bool {
        Self::is_enabled(self)
    }

    fn get_position(&self) -> i128 {
        Self::get_position(self)
    }

    fn set_position(&mut self, position: i128) {
        Self::set_position(self, position);
    }
}
