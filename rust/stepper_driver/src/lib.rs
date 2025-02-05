use std::{task::Poll, time::SystemTime};

use measurements::{Acceleration, Positional, Speed};
use pid::{PositionController, PositionControllerParams, SpeedContgrollerParams, SpeedController};

pub mod error;
pub mod measurements;
pub mod pid;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StepperDriver {
    // === kinematics config ===
    /// step/s^2
    pub max_acceleration: f64,

    /// step/s
    pub max_speed: f64,

    /// step
    pub min_position: i128,

    /// step
    pub max_position: i128,

    // === kinematics ===
    /// steps a float
    ///
    /// position is written by the update function
    pub position: i128,

    /// step/s
    pub speed: f64,

    /// step/s^2
    pub acceleration: f64,

    // === kinematics update ===
    pub mode: Mode,

    pub last_t: SystemTime,

    // === io config ===
    /// pulse duty cycle in seconds (seconds)
    pub pulse_time: f64,

    /// time after a duty cycle that no new pulse can be sent (seconds)
    pub min_pulse_offset: f64,

    /// time after a direction change that no new pulse can be sent (seconds)
    pub min_direction_offset: f64,

    // === io state ===
    /// time of last pulse rising edge
    pub last_pulse_t: SystemTime,

    /// time of last direction change edge
    pub last_direction_t: SystemTime,

    /// last pulse state
    pub last_output: Output,

    // === environment config ===
    /// steps per revolution
    /// Needed for conversion between different units to steps
    pub steps_per_revolution: Option<i128>,

    /// radius in meters
    /// Needed for conversion between different units to steps
    pub radius: Option<f64>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Output {
    /// pulse true = high, false = low
    pub pulse: bool,
    /// direction true = forward, false = backward
    pub direction: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    Speed {
        target_speed: f64,
        pd: SpeedController,
    },
    Position {
        target_position: i128,
        pid: PositionController,
    },
}

impl StepperDriver {
    pub fn new(
        // physics config
        max_acceleration: f64,
        max_speed: f64,
        min_position: i128,
        max_position: i128,
        // io config
        pulse_time: f64,
        min_pulse_offset: f64,
        min_direction_offset: f64,
        // environemt config
        steps_per_revolution: Option<i128>,
        radius: Option<f64>,
    ) -> Self {
        StepperDriver {
            // physics config
            max_acceleration,
            max_speed,
            min_position,
            max_position,
            // physics
            position: 0,
            speed: 0.0,
            acceleration: 0.0,
            // update
            mode: Mode::Speed {
                target_speed: 0.0,
                pd: SpeedController::new(&SpeedContgrollerParams { kp: 0.0, kd: 0.0 }),
            },
            last_t: SystemTime::now(),
            // io config
            pulse_time,
            min_pulse_offset,
            min_direction_offset,
            // io state
            last_pulse_t: SystemTime::now(),
            last_direction_t: SystemTime::now(),
            last_output: Output {
                pulse: false,
                direction: false,
            },
            // environemt config
            steps_per_revolution,
            radius,
        }
    }

    // === STEP COMMANDS ===

    /// Move to an absolute position
    pub fn move_absolute(
        &mut self,
        value: &Positional,
        params: &PositionControllerParams,
    ) -> Result<(), crate::error::Error> {
        let new_position = value.to_steps(self.steps_per_revolution, self.radius)?;

        // check if new position is within limits
        if new_position > self.max_position || new_position < self.min_position {
            return Err(crate::error::Error::ExceedsLimits);
        }

        self.mode = Mode::Position {
            target_position: new_position,
            pid: PositionController::new(params),
        };
        Ok(())
    }

    /// Move to a relative position
    pub fn move_relative(
        &mut self,
        value: &Positional,
        params: &PositionControllerParams,
    ) -> Result<(), crate::error::Error> {
        let new_position =
            self.position + value.to_steps(self.steps_per_revolution, self.radius)?;

        // check if new position is within limits
        if new_position > self.max_position || new_position < self.min_position {
            return Err(crate::error::Error::ExceedsLimits);
        }

        self.mode = Mode::Position {
            target_position: new_position,
            pid: PositionController::new(params),
        };
        Ok(())
    }

    /// Move at a constant speed
    pub fn move_speed(
        &mut self,
        value: &Speed,
        params: &SpeedContgrollerParams,
    ) -> Result<(), crate::error::Error> {
        let speed = value.to_steps_per_seconds(self.steps_per_revolution, self.radius)?;

        // check if speed is within limits
        if speed > self.max_speed {
            return Err(crate::error::Error::ExceedsMaxSpeed);
        }

        self.mode = Mode::Speed {
            target_speed: speed,
            pd: SpeedController::new(params),
        };
        Ok(())
    }

    /// Sets the maximum speed
    pub fn set_max_speed(&mut self, value: &Speed) -> Result<(), crate::error::Error> {
        self.max_speed = value.to_steps_per_seconds(self.steps_per_revolution, self.radius)?;
        Ok(())
    }

    /// Sets the maximum acceleration
    pub fn set_max_acceleration(
        &mut self,
        value: &Acceleration,
    ) -> Result<(), crate::error::Error> {
        self.max_acceleration =
            value.to_steps_per_seconds_squared(self.steps_per_revolution, self.radius)?;
        Ok(())
    }

    /// Sets the maximum position
    ///
    /// Attention: Stepper wont move when it currently outside of new limit
    pub fn set_max_position(&mut self, value: &Positional) -> Result<(), crate::error::Error> {
        // check if currently outside of limits
        let limit = value.to_steps(self.steps_per_revolution, self.radius)?;
        if self.position > limit {
            return Err(crate::error::Error::StepperOutsideOfLimits);
        }

        self.max_position = value.to_steps(self.steps_per_revolution, self.radius)?;
        Ok(())
    }

    /// Sets the minimum position
    ///
    /// Attention: Stepper wont move when it currently outside of new limit
    pub fn set_min_position(&mut self, value: &Positional) -> Result<(), crate::error::Error> {
        // check if currently outside of limits
        let limit = value.to_steps(self.steps_per_revolution, self.radius)?;
        if self.position < limit {
            return Err(crate::error::Error::StepperOutsideOfLimits);
        }

        self.min_position = value.to_steps(self.steps_per_revolution, self.radius)?;
        Ok(())
    }

    /// will set the current position and also reduce the target position by the same amount in positional mode
    pub fn set_position(&mut self, value: &Positional) -> Result<(), crate::error::Error> {
        let new_position = value.to_steps(self.steps_per_revolution, self.radius)?;
        let diff = self.position - new_position;
        self.position = new_position;
        // reduce target position relative to current position
        match &mut self.mode {
            Mode::Position {
                target_position,
                pid: _,
            } => {
                *target_position -= diff;
            }
            _ => {}
        }
        Ok(())
    }

    /// if the stepper is at a position within the tolerance
    pub fn has_position(&self, value: &Positional, tolerance: &Positional) -> bool {
        let target = value
            .to_steps(self.steps_per_revolution, self.radius)
            .unwrap();
        let tolerance = tolerance
            .to_steps(self.steps_per_revolution, self.radius)
            .unwrap();
        let position = self.position;
        position >= target - tolerance && position <= target + tolerance
    }

    /// if the stepper is at a position within the tolerance
    pub fn wait_for_position(
        &self,
        value: &Positional,
        tolerance: &Positional,
    ) -> Poll<Result<(), crate::error::Error>> {
        if self.has_position(value, tolerance) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    /// if the stepper is at a speed within the tolerance
    pub fn has_speed(&self, value: &Speed, tolerance: &Speed) -> bool {
        let target = value
            .to_steps_per_seconds(self.steps_per_revolution, self.radius)
            .unwrap();
        let tolerance = tolerance
            .to_steps_per_seconds(self.steps_per_revolution, self.radius)
            .unwrap();
        let speed = self.speed;
        speed >= target - tolerance && speed <= target + tolerance
    }

    /// if the stepper is at a speed within the tolerance
    pub fn wait_for_speed(
        &self,
        value: &Speed,
        tolerance: &Speed,
    ) -> Poll<Result<(), crate::error::Error>> {
        if self.has_speed(value, tolerance) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    /// returns the output for the current time
    /// should be polled at a higher frequency that the pulse time to ensure smooth movement and correct timing
    /// should be polled at incrementing timestamps
    ///
    /// pulse and direction never change at the same time
    pub fn output(&mut self, t: &SystemTime) -> Output {
        let dt: f64 = t.duration_since(self.last_t).unwrap().as_secs_f64();
        self.update_kinematics(dt);

        // check if we can end the pulse
        let pulse_dt = t.duration_since(self.last_pulse_t).unwrap().as_secs_f64();
        if self.last_output.pulse {
            if pulse_dt > self.pulse_time {
                // end pulse
                return Output {
                    pulse: false,
                    direction: self.last_output.direction,
                };
            } else {
                // continue pulse
                return self.last_output.clone();
            }
        }

        // check if we have to wait for pulse offset
        if pulse_dt < self.min_pulse_offset {
            return self.last_output.clone();
        }

        // check if we have to wait for direction change
        let direction_dt = t
            .duration_since(self.last_direction_t)
            .unwrap()
            .as_secs_f64();
        if direction_dt < self.min_direction_offset {
            return self.last_output.clone();
        }

        // check if we should change direction
        // we have to do a direction change if last_output.direction != self.speed > 0
        if self.speed > 0.0 && !self.last_output.direction {
            self.last_direction_t = t.clone();
            return Output {
                pulse: false,
                direction: true,
            };
        } else if self.speed < 0.0 && self.last_output.direction {
            self.last_direction_t = t.clone();
            return Output {
                pulse: false,
                direction: false,
            };
        }

        // invert speed from steps/s to s/step
        let sec_per_step = match self.speed {
            0.0 => f64::INFINITY,
            _ => 1.0 / self.speed.abs(),
        };

        // check if we can start a new pulse
        if dt > sec_per_step {
            // check if we are inside min max position
            let new_position = match self.last_output.direction {
                true => self.position + 1,
                false => self.position - 1,
            };
            if new_position > self.max_position || new_position < self.min_position {
                return self.last_output.clone();
            }

            // do pulse
            self.last_pulse_t = t.clone();
            self.position = new_position;
            return Output {
                pulse: true,
                direction: self.last_output.direction,
            };
        }

        // no change
        self.last_output.clone()
    }

    /// updates the kinematics based on distance to target speed or position
    fn update_kinematics(&mut self, dt: f64) {
        let new_speed = self.speed + self.acceleration * dt;
        let new_position = self.position + (self.speed * dt * self.acceleration * dt * dt) as i128;
        let new_acceleration = match &mut self.mode {
            Mode::Speed { target_speed, pd } => {
                let current_speed = self.speed;
                let current_acceleration = self.acceleration;
                pd.update(*target_speed, current_speed, current_acceleration, dt)
            }
            Mode::Position {
                target_position,
                pid,
            } => pid.update(*target_position, self.position, dt),
        };

        // clamp speed
        let new_speed = clamp_f64(new_speed, -self.max_speed, self.max_speed);

        // clamp position
        let new_position = clamp_i128(new_position, self.min_position, self.max_position);

        // clamp acceleration
        let new_acceleration = clamp_f64(
            new_acceleration,
            -self.max_acceleration,
            self.max_acceleration,
        );

        // set kinematics
        self.speed = new_speed;
        self.position = new_position;
        self.acceleration = new_acceleration;
    }
}

fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn clamp_i128(value: i128, min: i128, max: i128) -> i128 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
