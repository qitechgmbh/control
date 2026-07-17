use crate::{MACHINE_LASER_V1, MachineMessage, QiTechMachine, VENDOR_QITECH};
use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::MachineIdentification;
use control_core::MachineIdentificationUnique;
use control_core::data::ConfigMutationOrigin;
use control_core::data::EventRecorderHandle;
use control_core::machine::ConfigProperty;
use control_core::machine::MachineError;
use control_core::machine::Measurement;
use control_core::machine::StateProperty;
use control_core_legacy::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    modbus::{
        ModbusDevice,
        devices::qitech_laser::{LaserDevice, LaserError},
    },
    units::{Length, length::millimeter},
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
pub mod build;

pub struct LaserMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: LaserMachineNamespace,
    last_measurement_emit: Instant,
    last_request: Instant,
    laser: Rc<RefCell<LaserDevice>>,
    error: Option<MachineError>,

    // config
    config_diameter: DiameterConfig,
    global_warning: bool,

    // state
    in_tolerance: StateProperty<bool>,

    // measurements
    diameter: Measurement<Length, millimeter>,
    diameter_x: Measurement<Option<Length>, millimeter>,
    diameter_y: Measurement<Option<Length>, millimeter>,
    roundness: Measurement<Option<f64>>,

    // --- events ---
    out_of_tolerance: EventRecorderHandle<()>,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
    did_change_state: bool,
}

impl LaserMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_LASER_V1,
    };

    pub fn get_live_values(&self) -> LiveValuesEvent {
        LiveValuesEvent {
            diameter: self.diameter.get_native(),
            x_diameter: self.diameter_x.get_native(),
            y_diameter: self.diameter_y.get_native(),
            roundness: self.roundness.get(),
        }
    }

    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(LaserEvents::LiveValues(event));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let laser = LaserState {
            higher_tolerance: self.config_diameter.tolerance_higher.get_native(),
            lower_tolerance: self.config_diameter.tolerance_lower.get_native(),
            target_diameter: self.config_diameter.target.get_native(),
            in_tolerance: *self.in_tolerance.get(),
            global_warning: self.global_warning,
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
                higher_tolerance: self.config_diameter.tolerance_higher.get_native(),
                lower_tolerance: self.config_diameter.tolerance_lower.get_native(),
                target_diameter: self.config_diameter.target.get_native(),
                in_tolerance: *self.in_tolerance.get(),
                global_warning: self.global_warning,
            },
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(LaserEvents::State(event));
        self.did_change_state = false;
        self.emitted_default_state = true;
    }

    pub fn set_higher_tolerance(&mut self, value: f64) {
        self.config_diameter.tolerance_higher.set_as::<millimeter>(
            value, 
            ConfigMutationOrigin::User { request_id: 0 }
        );
        
        self.emit_state();
    }

    pub fn set_lower_tolerance(&mut self, value: f64) {
        self.config_diameter.tolerance_lower.set_as::<millimeter>(
            value, 
            ConfigMutationOrigin::User { request_id: 0 }
        );

        self.emit_state();
    }

    pub fn set_target_diameter(&mut self, value: f64) {
        self.config_diameter.tolerance_lower.set_as::<millimeter>(
            value, 
            ConfigMutationOrigin::User { request_id: 0 }
        );

        self.emit_state();
    }

    pub fn set_global_warning(&mut self, toggle: bool) {
        self.global_warning = toggle;
        self.emit_state();
    }

    /// Roundness = min(x, y) / max(x, y)
    fn calculate_roundness(&mut self) -> Option<f64> {
        match (self.diameter_x.get(), self.diameter_y.get()) {
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
    }

    ///
    /// Calculates if the current diameter is inside of the tolerance
    ///
    fn update_in_tolerance(&mut self) {
        let diameter_epsilon: f64 = 0.0001; // 0.0001 mm
        // early return true if the diameter is 0 to prevent warning happening before start
        if self.diameter.get_as::<millimeter>() < diameter_epsilon {
            self.in_tolerance.set(true);
            return;
        }

        let target = self.config_diameter.target.get_as::<millimeter>();
        let diameter = self.diameter.get_as::<millimeter>();
        let tolerance_higher = self.config_diameter.tolerance_higher.get_as::<millimeter>();
        let tolerance_lower = self.config_diameter.tolerance_lower.get_as::<millimeter>();

        let top = target + tolerance_higher;
        let bottom = target - tolerance_lower;

        let new_value = diameter < bottom || diameter > top;
        self.in_tolerance.set(new_value);
    }

    pub fn update(&mut self) {
        let mut laser = self.laser.borrow_mut();
        let now = std::time::Instant::now();

        // Check for incoming responses on every tick
        let res = laser.handle_response();
        match res {
            Ok(_) => (),
            Err(e) => {
                if let Some(laser_error) = e.downcast_ref::<LaserError>() {
                    match laser_error {
                        LaserError::IoErr() => {
                            self.error = Some(MachineError::IrrecoverableFailure(
                                "Physical hardware I/O broke. Dropping machine permanently."
                                    .to_owned(),
                            ));
                        }
                        _ => (),
                    }
                }
            }
        }

        if now.duration_since(self.last_request) > Duration::from_millis(6) {
            self.last_request = now;
            let res = laser.send_next_request();
            if res.is_err() {
                println!("send_next_request {:?}", res);
            }
        }

        match &laser.measurement {
            Some(m) => {
                fn convert(value: u16) -> f64 {
                    value as f64 / 1000.0
                }

                self.diameter.set_as::<millimeter>(convert(m.diameter));
                self.diameter_x.set_as::<millimeter>(Some(convert(m.x_axis)));
                self.diameter_y.set_as::<millimeter>(Some(convert(m.y_axis)));
            }
            None => (),
        };
        drop(laser);

        let roundness = self.calculate_roundness();
        self.roundness.set(roundness);

        let prev_in_tolerance = self.in_tolerance.get();
        if prev_in_tolerance != self.in_tolerance.get() {
            self.did_change_state = true;
        }
    }
}

pub enum LaserRequestState {
    Waiting(Instant),
    NotWaiting,
}

pub struct DiameterConfig {
    target: ConfigProperty<Length, millimeter>,
    tolerance_lower: ConfigProperty<Length, millimeter>,
    tolerance_higher: ConfigProperty<Length, millimeter>,
}

impl QiTechMachine for LaserMachine {}
