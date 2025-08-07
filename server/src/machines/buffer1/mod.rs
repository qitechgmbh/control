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
        ConnectedMachine, ConnectedMachineData, Machine, downcast_machine,
        identification::{MachineIdentification, MachineIdentificationUnique},
        manager::MachineManager,
    },
    socketio::namespace::NamespaceCacheingLogic,
    uom_extensions::velocity::meter_per_minute,
};
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use control_core_derive::Machine;
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
        api::{ConnectedMachineState, CurrentInputSpeedState, PullerState},
        puller_speed_controller::PullerSpeedController,
    },
    winder2::{Winder2, Winder2Mode},
};

#[derive(Debug, Machine)]
pub struct BufferV1 {
    // drivers
    pub lift: StepperVelocityEL70x1,
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
    pub connected_winder: Option<ConnectedMachine<Weak<Mutex<Winder2>>>>,

    // mode
    mode: BufferV1Mode,
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
}

impl BufferV1 {
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
            connected_machine_state: ConnectedMachineState {
                machine_identification_unique: self.connected_winder.as_ref().map(
                    |connected_machine| {
                        ConnectedMachineData::from(connected_machine).machine_identification_unique
                    },
                ),
                is_available: self
                    .connected_winder
                    .as_ref()
                    .map(|connected_machine| {
                        ConnectedMachineData::from(connected_machine).is_available
                    })
                    .unwrap_or(false),
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
}

impl BufferV1 {
    fn fill_buffer(&mut self) {
        // stop the winder until the buffer is ful
        self.update_winder2_mode(Winder2Mode::Hold);
    }

    fn empty_buffer(&mut self) {
        // Set the winder2 to a mode where its faster than before to empty the buffer slowly
        self.update_winder2_mode(Winder2Mode::Pull);
    }

    fn update_winder2_mode(&mut self, mode: Winder2Mode) {
        self.get_winder(|winder2| {
            if winder2.mode != mode {
                winder2.mode = mode;
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
        self.buffer_lift_controller.set_enabled(true);
        self.buffer_lift_controller.set_forward(true);
    }

    // Turn on motor reverse and empty buffer
    fn switch_to_emptying(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.empty_buffer(),
            BufferV1Mode::Hold => {},
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
}

impl BufferV1 {
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
        let linear_velocity = self.buffer_lift_controller.update_speed(t);
        let steps_per_second = self.lift_step_converter.velocity_to_steps(linear_velocity);
        let _ = self.lift.set_speed(steps_per_second);
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
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => return,
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let winder2_weak = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let winder2_weak = match winder2_weak {
            Some(winder2_weak) => winder2_weak,
            None => return,
        };
        let winder2_strong = match winder2_weak.upgrade() {
            Some(winder2_strong) => winder2_strong,
            None => return,
        };

        let winder2: Arc<Mutex<Winder2>> = block_on(downcast_machine::<Winder2>(winder2_strong))
            .expect("failed downcasting machine");

        let machine = Arc::downgrade(&winder2);

        self.connected_winder = Some(ConnectedMachine {
            machine_identification_unique,
            machine,
        });

        self.emit_state();

        self.reverse_connect();
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
        if let Some(connected) = &self.connected_winder {
            if let Some(winder2_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut winder2 = winder2_arc.lock().await;
                    if winder2.connected_buffer.is_some() {
                        winder2.connected_buffer = None;
                        winder2.emit_state();
                    }
                };
                smol::spawn(future).detach();
            }
        }
        self.connected_winder = None;
        self.emit_state();
    }

    /// initiate connection from winder to buffer
    pub fn reverse_connect(&mut self) {
        let machine_identification_unique = self.machine_identification_unique.clone();
        if let Some(connected) = &self.connected_winder {
            if let Some(winder2_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut winder2 = winder2_arc.lock().await;
                    if winder2.connected_buffer.is_none() {
                        winder2.set_connected_buffer(machine_identification_unique);
                    }
                };
                smol::spawn(future).detach();
            }
        }
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
