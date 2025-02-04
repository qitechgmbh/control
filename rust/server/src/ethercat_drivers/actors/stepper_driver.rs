use crate::ethercat_drivers::{
    actor::Actor, io::digital_output::DigitalOutput, utils::traits::ArcRwLock,
};
use std::{future::Future, pin::Pin, time::Duration};

enum StepperDriverTransition {
    Linear,
}

#[derive(PartialEq)]
enum Action {
    Idle,
    MoveToPosition(ActionMoveToPosition),
    MoveAtSpeed(ActionMoveAtSpeed),
}

#[derive(PartialEq)]
struct ActionMoveToPosition {
    position_start: i64,
    position_end: i64,
    duration: Duration,
}

#[derive(PartialEq)]
struct ActionMoveAtSpeed {
    /// steps per second
    cycle_duration: Duration,
    direction: bool,
}

/// Set a digital output high and low with a given interval
pub struct StepperDriver {
    // Context
    /// Steps on the motor per revolution
    steps: u16,

    // Hardware
    /// Digital output to control the motor
    pulse: DigitalOutput,
    /// Digital output to control the direction of the motor
    direction: DigitalOutput,

    // State
    /// position as in steps
    position: i64,
    action: Action,

    /// Time of the last step in nanoseconds
    last_step: u64,
}

pub enum StepperSpeed {
    StepsPerSecond(f64),
    RevolutionsPerMinute(f64),
    NanosecondsPerStep(i64),
}

impl StepperSpeed {
    fn step_duration(&self, _stepper_driver: &StepperDriver) -> Duration {
        match self {
            StepperSpeed::StepsPerSecond(sps) => {
                // steps pers second to nanoseconds per step
                let ns_per_step = (1_000_000_000.0 / sps) as u64;
                Duration::from_nanos(ns_per_step)
            }
            StepperSpeed::RevolutionsPerMinute(rpm) => {
                // steps per minute to nanoseconds per step
                let ns_per_step = (60_000_000_000.0 / rpm) as u64;
                Duration::from_nanos(ns_per_step)
            }
            StepperSpeed::NanosecondsPerStep(ns) => Duration::from_nanos(*ns as u64),
        }
    }
}

impl StepperDriver {
    fn new(steps: u16, pulse: DigitalOutput, direction: DigitalOutput) -> Self {
        Self {
            steps,
            position: 0,
            action: Action::Idle,
            last_step: 0,
            pulse,
            direction,
        }
    }

    // fn move_to_position(&mut self, steps: i64, duration: Duration) {
    //     self.action = Action::MoveToPosition(ActionMoveToPosition {
    //         position_start: self.position,
    //         position_end: steps,
    //         duration,
    //     });
    // }

    fn move_at_speed(&mut self, speed: StepperSpeed, direction: bool) {
        self.action = Action::MoveAtSpeed(ActionMoveAtSpeed {
            cycle_duration: speed.step_duration(self),
            direction,
        });
    }
}

impl Actor for StepperDriver {
    async fn act(&mut self, now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let pulse = (self.pulse.state)().await;
            match pulse.value {
                true => {
                    // check when to low the signal
                    let pulse_duration = Duration::from_millis(1).as_nanos() as u64;
                    if (now_ts - self.last_step) > pulse_duration {
                        // low the signal
                        (self.pulse.write)(false).await;
                    }
                }
                false => {
                    // check if we are at the max speed
                    let min_cycle_duration = Duration::from_millis(2).as_nanos() as u64;
                    if (now_ts - self.last_step) < min_cycle_duration {
                        return;
                    }

                    // increment the position
                }
            }
        })
    }
}

impl ArcRwLock for StepperDriver {}
