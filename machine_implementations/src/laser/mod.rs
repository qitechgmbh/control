use property::{BoolProperty, LengthProperty};

use crate::{
    MACHINE_LASER_V1, MachineMessage, QiTechMachine, VENDOR_QITECH,
};
use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    machines::{MachineError, MachineIdentification, MachineIdentificationUnique},
    modbus::{
        ModbusDevice,
        devices::qitech_laser::{LaserDevice, LaserError},
    },
    units::length::millimeter,
};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub mod act;
pub mod api;
pub mod new;

pub enum LaserRequestState {
    Waiting(Instant),
    NotWaiting,
}

pub struct LaserMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: LaserMachineNamespace,
    last_measurement_emit: Instant,
    last_request: Instant,
    laser: Rc<RefCell<LaserDevice>>,
    error: Option<MachineError>,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
    did_change_state: bool,

    // properties
    config: Config,
    diameter: LengthProperty<millimeter>,
    x_diameter: LengthProperty<millimeter>,
    y_diameter: LengthProperty<millimeter>,
    in_tolerance: BoolProperty,
    global_warning: BoolProperty,
    // roundness: Option<f64>,
}

impl LaserMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_LASER_V1,
    };

    pub fn get_live_values(&self) -> LiveValuesEvent {
        let diameter = self.diameter.get_as::<millimeter>();
        let x_diameter = Some(self.x_diameter.get_as::<millimeter>());
        let y_diameter = Some(self.y_diameter.get_as::<millimeter>());
        // let roundness = self.roundness;

        LiveValuesEvent {
            diameter,
            x_diameter,
            y_diameter,
            roundness: None,
        }
    }

    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(LaserEvents::LiveValues(event));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let laser = LaserState {
            higher_tolerance: self.config.higher_tolerance.get().get::<millimeter>(),
            lower_tolerance: self.config.lower_tolerance.get().get::<millimeter>(),
            target_diameter: self.config.target_diameter.get().get::<millimeter>(),
            in_tolerance: self.in_tolerance.get(),
            global_warning: self.global_warning.get(),
        };

        StateEvent {
            is_default_state: false,
            laser_state: laser,
        }
    }

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            is_default_state: !self.emitted_default_state,
            laser_state: LaserState {
                higher_tolerance: self.config.higher_tolerance.get_as::<millimeter>(),
                lower_tolerance: self.config.lower_tolerance.get_as::<millimeter>(),
                target_diameter: self.config.target_diameter.get_as::<millimeter>(),
                in_tolerance: self.in_tolerance.get(),
                global_warning: self.global_warning.get(),
            },
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(LaserEvents::State(event));
        self.did_change_state = false;
        self.emitted_default_state = true;
    }

    pub fn set_higher_tolerance(&mut self, higher_tolerance: f64) {
        self.config
            .higher_tolerance
            .set_as::<millimeter>(higher_tolerance);
        self.emit_state();
    }

    pub fn set_lower_tolerance(&mut self, lower_tolerance: f64) {
        self.config
            .lower_tolerance
            .set_as::<millimeter>(lower_tolerance);
        self.emit_state();
    }

    pub fn set_target_diameter(&mut self, target_diameter: f64) {
        self.config
            .target_diameter
            .set_as::<millimeter>(target_diameter);
        self.emit_state();
    }

    pub fn set_global_warning(&mut self, toggle: bool) {
        self.global_warning.set(toggle);
        self.emit_state();
    }

    /*
    ///
    /// Roundness = min(x, y) / max(x, y)
    ///
    fn calculate_roundness(&mut self) -> Option<f64> {
        match (self.x_diameter, self.y_diameter) {
            (Some(x), Some(y)) => {
                let x_val = x.get::<millimeter>();
                let y_val = y.get::<millimeter>();

                if x_val > 0.0 && y_val > 0.0 {
                    let roundness = f64::min(x_val, y_val) / f64::max(x_val, y_val);
                    Some(roundness)
                } else if x_val == 0.0 && y_val == 0.0 {
                    Some(0.0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }*/

    ///
    /// Calculates if the current diameter is inside of the tolerance
    ///
    fn compute_in_tolerance(&self) -> bool {
        let diameter_epsilon: f64 = 0.0001; // 0.0001 mm

        // early return true if the diameter is 0 to prevent warning happening before start
        if self.diameter.get_as::<millimeter>() < diameter_epsilon {
            return true;
        }

        let top = self.config.target_diameter.get() + self.config.higher_tolerance.get();
        let bottom = self.config.target_diameter.get() - self.config.lower_tolerance.get();

        !(self.diameter.get() > top || self.diameter.get() < bottom)
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let mut laser = self.laser.borrow_mut();

        // Check for incoming responses on every tick
        if let Err(e) = laser.handle_response() {
            let Some(laser_error) = e.downcast_ref::<LaserError>() else {
                return;
            };

            if let LaserError::IoErr() = laser_error {
                self.error = Some(MachineError::IrrecoverableFailure(
                    "Physical hardware I/O broke. Dropping machine permanently.".to_owned(),
                ));
            }
        }

        if now.duration_since(self.last_request) > Duration::from_millis(6) {
            self.last_request = now;
            let res = laser.send_next_request();
            if res.is_err() {
                println!("send_next_request {:?}", res);
            }
        }

        if let Some(m) = &laser.measurement {
            self.diameter.set_as::<millimeter>(m.diameter as f64 / 1000.0);
            self.x_diameter.set_as::<millimeter>(m.x_axis as f64 / 1000.0);
            self.y_diameter.set_as::<millimeter>(m.y_axis as f64 / 1000.0);
        };

        self.in_tolerance.set(self.compute_in_tolerance());

        /*
        if self.in_tolerance.is_dirty() {
            self.did_change_state = true;
        }
        */
    }
}

#[derive(Debug)]
pub struct Config {
    target_diameter: LengthProperty<millimeter>,
    higher_tolerance: LengthProperty<millimeter>,
    lower_tolerance: LengthProperty<millimeter>,
}

impl QiTechMachine for LaserMachine {}
