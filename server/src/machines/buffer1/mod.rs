pub mod act;
pub mod api;
pub mod buffer_lift_controller;
pub mod new;
pub mod puller_speed_controller;

use api::{Buffer1Namespace, BufferV1Events, LiveValuesEvent, ModeState, StateEvent};
use buffer_lift_controller::BufferLiftController;
use control_core::{
    converters::linear_step_converter::LinearStepConverter,
    machines::{
        identification::{MachineIdentification, MachineIdentificationUnique},
        manager::MachineManager,
    },
    socketio::namespace::NamespaceCacheingLogic,
    uom_extensions::velocity::meter_per_minute,
};
use ethercat_hal::io::{
    digital_input::DigitalInput, stepper_velocity_el70x1::StepperVelocityEL70x1,
};
use futures::executor::block_on;
use puller_speed_controller::PullerRegulationMode;
use serde::{Deserialize, Serialize};
use smol::lock::{Mutex, RwLock};
use std::{
    sync::{Arc, Weak},
    time::Instant,
};
use uom::si::{
    f64::{Length, Velocity},
    length::millimeter,
};

use crate::machines::{
    MACHINE_BUFFER_V1, VENDOR_QITECH,
    buffer1::{
        api::{ConnectedMachineState, CurrentInputSpeedState, LiftState, PullerState},
        puller_speed_controller::PullerSpeedController,
    },
    winder2::{BufferState, Winder2, Winder2Mode},
};

#[derive(Debug, Machine)]
pub struct BufferV1 {
    // drivers
    pub lift: StepperVelocityEL70x1,
    pub lift_end_stop: DigitalInput,
    pub puller: StepperVelocityEL70x1,

    // controllers
    pub buffer_lift_controller: BufferLiftController,
    pub puller_speed_controller: PullerSpeedController,

    pub lift_step_converter: LinearStepConverter,

    // socketio
    namespace: Buffer1Namespace,
    last_measurement_emit: Instant,

    // machine connection
    pub machine_manager: Weak<RwLock<MachineManager>>,
    pub machine_identification_unique: MachineIdentificationUnique,

    // connected machines
    pub connected_winder: MachineCrossConnection<BufferV1, Winder2>,

    // mode
    mode: BufferV1Mode,
}

impl CrossConnectableMachine<BufferV1, Winder2> for BufferV1 {
    fn get_cross_connection(&mut self) -> &mut MachineCrossConnection<BufferV1, Winder2> {
        &mut self.connected_winder
    }
}

impl std::fmt::Display for BufferV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferV1")
    }
}

impl BufferV1 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_BUFFER_V1,
    };
    pub fn emit_live_values(&mut self) {
        // Calculate puller speed from current motor steps
        let steps_per_second = self.puller.get_speed();
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(steps_per_second as f64);
        let puller_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);

        // live values to be emittet
        let live_values = LiveValuesEvent {
            lift_position: self
                .buffer_lift_controller
                .get_current_position()
                .map(|x| x.get::<millimeter>()),
            puller_speed: puller_speed.get::<meter_per_minute>(),
        };

        let event = live_values.build();
        self.namespace.emit(BufferV1Events::LiveValues(event));
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            connected_machine_state: self.connected_winder.to_state(),
            lift_state: LiftState {
                limit_top: self
                    .buffer_lift_controller
                    .get_limit_inner()
                    .get::<millimeter>(),
                limit_bottom: self
                    .buffer_lift_controller
                    .get_limit_outer()
                    .get::<millimeter>(),
                position_in: self
                    .buffer_lift_controller
                    .get_limit_inner()
                    .get::<millimeter>(),
                position_out: self
                    .buffer_lift_controller
                    .get_limit_outer()
                    .get::<millimeter>(),
                is_going_up: self.buffer_lift_controller.is_going_in(),
                is_going_down: self.buffer_lift_controller.is_going_out(),
                is_homed: self.buffer_lift_controller.is_homed(),
                is_going_home: self.buffer_lift_controller.is_going_home(),
                is_buffering: self.buffer_lift_controller.is_traversing(),
                step_size: self
                    .buffer_lift_controller
                    .get_step_size()
                    .get::<millimeter>(),
                padding: self
                    .buffer_lift_controller
                    .get_padding()
                    .get::<millimeter>(),
                can_go_top: self.can_go_in(),
                can_go_bottom: self.can_go_out(),
                can_go_home: self.can_go_home(),
            },
            current_input_speed_state: CurrentInputSpeedState {
                current_input_speed: self
                    .buffer_lift_controller
                    .get_current_input_speed()
                    .get::<meter_per_minute>(),
            },
            puller_state: PullerState {
                regulation: self.puller_speed_controller.regulation_mode.clone(),
                target_speed: self
                    .puller_speed_controller
                    .target_speed
                    .get::<meter_per_minute>(),
                target_diameter: self
                    .puller_speed_controller
                    .target_diameter
                    .get::<millimeter>(),
                forward: self.puller_speed_controller.forward,
            },
        };

        let event = state.build();
        self.namespace.emit(BufferV1Events::State(event));
    }

impl BufferV1 {
    fn fill_buffer(&mut self) {
        // stop the winder until the buffer is ful
        self.update_winder2_mode(Winder2Mode::Hold);
        self.update_winder2_buffer_state(BufferState::Buffering);
    }

    fn empty_buffer(&mut self) {
        // Set the winder2 to a mode where its faster than before to empty the buffer slowly
        self.update_winder2_mode(Winder2Mode::Pull);
        self.update_winder2_buffer_state(BufferState::Emptying);
    }

    fn update_winder2_mode(&mut self, mode: Winder2Mode) {
        self.get_winder(|winder2| {
            if winder2.mode != mode {
                winder2.mode = mode;
            }
        });
    }

    fn update_winder2_buffer_state(&mut self, state: BufferState) {
        self.get_winder(|winder2| {
            if winder2.buffer_state != state {
                winder2.set_buffer_state(state);
            }
        });
    }

    // Turn off motor and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => (),
            BufferV1Mode::Hold => {}
            BufferV1Mode::Filling => {}
            BufferV1Mode::Emptying => {}
        };
        self.mode = BufferV1Mode::Standby;
        self.buffer_lift_controller.set_enabled(false);
        let _ = self.buffer_lift_controller.stepper_driver.set_speed(0.0);
        self.update_winder2_buffer_state(BufferState::Hold);
    }

    // hold motor
    fn switch_to_hold(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => {}
            BufferV1Mode::Hold => (),
            BufferV1Mode::Filling => {}
            BufferV1Mode::Emptying => {}
        };
        self.mode = BufferV1Mode::Hold;
        self.buffer_lift_controller.set_enabled(false);
        let _ = self.buffer_lift_controller.stepper_driver.set_speed(0.0);
        self.update_winder2_buffer_state(BufferState::Hold);
    }

    // Turn on motor and fill buffer
    fn switch_to_filling(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.fill_buffer(),
            BufferV1Mode::Hold => {}
            BufferV1Mode::Filling => (),
            BufferV1Mode::Emptying => {}
        };
        self.mode = BufferV1Mode::Filling;
        self.buffer_lift_controller.set_forward(true);
        self.buffer_lift_controller.set_enabled(true);
    }

    // Turn on motor reverse and empty buffer
    fn switch_to_emptying(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.empty_buffer(),
            BufferV1Mode::Hold => {}
            BufferV1Mode::Filling => {}
            BufferV1Mode::Emptying => (),
        };
        self.mode = BufferV1Mode::Emptying;
        self.buffer_lift_controller.set_forward(false);
    }

    fn switch_mode(&mut self, mode: BufferV1Mode) {
        if self.mode == mode {
            return;
        }

        match mode {
            BufferV1Mode::Standby => self.switch_to_standby(),
            BufferV1Mode::Hold => self.switch_to_hold(),
            BufferV1Mode::Filling => self.switch_to_filling(),
            BufferV1Mode::Emptying => self.switch_to_emptying(),
        }
    }

    fn set_mode_state(&mut self, mode: BufferV1Mode) {
        self.switch_mode(mode);
        self.emit_state();
    }

    fn set_current_input_speed(&mut self, speed: f64) {
        // speed comes as a f64 represents m/min
        self.buffer_lift_controller.set_current_input_speed(speed);
        self.emit_state();
    }
}

// Implement Puller
impl BufferV1 {
    /// called by `act`
    pub fn sync_puller_speed(&mut self, t: Instant) {
        let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        let steps_per_second = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);
        let _ = self.puller.set_speed(steps_per_second);

        // sync puller speed to lift input speed
        let linear_velocity = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_velocity(angular_velocity);
        self.buffer_lift_controller
            .set_current_input_speed(linear_velocity.get::<meter_per_minute>());
    }

    pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
        self.puller_speed_controller
            .set_regulation_mode(puller_regulation_mode);
        self.emit_state();
    }

    /// Set target speed in m/min
    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        // Convert m/min to velocity
        let target_speed = Velocity::new::<meter_per_minute>(target_speed);
        self.puller_speed_controller.set_target_speed(target_speed);
        self.emit_state();
    }

    /// Set target diameter in mm
    pub fn puller_set_target_diameter(&mut self, target_diameter: f64) {
        // Convert mm to length
        let target_diameter = Length::new::<millimeter>(target_diameter);
        self.puller_speed_controller
            .set_target_diameter(target_diameter);
        self.emit_state();
    }

    /// Set forward direction
    pub fn puller_set_forward(&mut self, forward: bool) {
        self.puller_speed_controller.set_forward(forward);
        self.emit_state();
    }
}

// Implement Lift
impl BufferV1 {
    pub fn sync_lift_speed(&mut self, t: Instant) {
        let linear_velocity =
            self.buffer_lift_controller
                .update_speed(&mut self.lift, &self.lift_end_stop, t);
        if self.can_move() {
            let steps_per_second = self.lift_step_converter.velocity_to_steps(linear_velocity);
            let _ = self.lift.set_speed(steps_per_second);
        } else {
            let _ = self.lift.set_speed(0.0);
        }
    }

    fn can_move(&mut self) -> bool {
        match self.lift_end_stop.get_value() {
            Ok(reached) => {
                if reached {
                    false
                } else {
                    true
                }
            }
            Err(_) => false,
        }
    }

    fn validate_lift_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    pub fn lift_set_limit_top(&mut self, limit: f64) {
        let new_top = Length::new::<millimeter>(limit);
        let current_bottom = self.buffer_lift_controller.get_limit_bottom();

        if !Self::validate_lift_limits(new_top, current_bottom) {
            return;
        }

        self.buffer_lift_controller.set_limit_top(new_top);
        self.emit_state();
    }

    pub fn lift_set_limit_bottom(&mut self, limit: f64) {
        let new_bottom = Length::new::<millimeter>(limit);
        let current_top = self.buffer_lift_controller.get_limit_top();

        if !Self::validate_lift_limits(current_top, new_bottom) {
            return;
        }

        self.buffer_lift_controller.set_limit_bottom(new_bottom);
        self.emit_state();
    }

    pub fn lift_set_step_size(&mut self, step_size: f64) {
        let step_size = Length::new::<millimeter>(step_size);
        self.buffer_lift_controller.set_step_size(step_size);
        self.emit_state();
    }

    pub fn lift_set_padding(&mut self, padding: f64) {
        let padding = Length::new::<millimeter>(padding);
        self.buffer_lift_controller.set_padding(padding);
        self.emit_state();
    }

    pub fn lift_goto_limit_top(&mut self) {
        if self.can_move() {
            self.buffer_lift_controller.goto_limit_top();
        }
        self.emit_state();
    }

    pub fn lift_goto_limit_bottom(&mut self) {
        if self.can_move() {
            self.buffer_lift_controller.goto_limit_bottom();
        }
        self.emit_state();
    }

    pub fn lift_goto_home(&mut self) {
        if self.can_move() {
            self.buffer_lift_controller.goto_home();
        }
        self.emit_state();
    }
}

// Connecting/Disconnecting machine
impl BufferV1 {
    /// set connected winder
    pub fn set_connected_winder(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            Winder2::MACHINE_IDENTIFICATION
        ) {
            return;
        }

        self.connected_winder
            .set_connected_machine(&machine_identification_unique);

        self.emit_state();

        self.connected_winder.reverse_connect();
    }

    /// disconnect winder
    pub fn disconnect_winder(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            Winder2::MACHINE_IDENTIFICATION
        ) {
            return;
        }

        self.connected_winder.disconnect();
        self.emit_state();
    }

    /// This helper function provides an easy way
    /// to get the machine out of the Weak Reference
    ///
    /// Usage:
    ///
    ///    self.get_winder(|winder2| {
    ///        winder2.do_something     // Use the Winder here as usual
    ///    });
    fn get_winder<F, R>(&self, func: F) -> Option<R>
    where
        F: FnOnce(&mut Winder2) -> R,
    {
        self.connected_winder
            .as_ref()?
            .machine
            .upgrade()
            .map(|winder_arc| {
                let mut winder = block_on(winder_arc.lock());
                func(&mut winder)
            })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum BufferV1Mode {
    Standby,
    Hold,
    Filling,
    Emptying,
}
