#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpeedContgrollerParams {
    pub kp: f64,
    pub kd: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpeedController {
    kp: f64,
    kd: f64,
    previous_error: f64,
}

impl SpeedController {
    pub fn new(params: &SpeedContgrollerParams) -> Self {
        SpeedController {
            kp: params.kp,
            kd: params.kd,
            previous_error: 0.0,
        }
    }

    /// target_speed: desired speed in /s
    /// current_speed: current speed in /s
    /// current_acceleration: current acceleration in /s^2
    /// delta_t: time since last update in s
    /// returns: new acceleration in /s^2
    pub fn update(
        &mut self,
        target_speed: f64,
        current_speed: f64,
        current_acceleration: f64,
        dt: f64,
    ) -> f64 {
        if dt <= 0.0 {
            return current_acceleration; // Avoid division by zero
        }

        let speed_error = target_speed - current_speed;
        let derivative = (speed_error - self.previous_error) / dt;
        self.previous_error = speed_error;

        // PD formula
        let output = self.kp * speed_error + self.kd * derivative;

        // Calculate new acceleration
        let new_acceleration = current_acceleration + output;
        new_acceleration
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PositionControllerParams {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PositionController {
    kp: f64,             // Proportional gain
    ki: f64,             // Integral gain
    kd: f64,             // Derivative gain
    previous_error: f64, // Previous error for derivative calculation
    integral: f64,       // Integral of the error
}

impl PositionController {
    pub fn new(params: &PositionControllerParams) -> Self {
        PositionController {
            kp: params.kp,
            ki: params.ki,
            kd: params.kd,
            previous_error: 0.0,
            integral: 0.0,
        }
    }

    /// target_position: desired position
    /// current_position: current position
    /// dt: time since last update in seconds
    /// returns: control output
    pub fn update(&mut self, target_position: i128, current_position: i128, dt: f64) -> f64 {
        if dt <= 0.0 {
            return 0.0; // Avoid division by zero
        }

        // Calculate the error
        let position_error = (target_position - current_position) as f64;

        // Update the integral term
        self.integral += position_error * dt;

        // Calculate the derivative term
        let derivative = (position_error - self.previous_error) / dt;

        // Update the previous error
        self.previous_error = position_error;

        // PID formula
        let output = self.kp * position_error + self.ki * self.integral + self.kd * derivative;

        output
    }
}
