use crate::{MACHINE_LASER_V1, MachineMessage, QiTechMachine, VENDOR_QITECH};
use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use postcard::from_bytes;
use postcard::to_slice;
use qitech_lib::{
    machines::{
        ConvertMachineData, MachineData, MachineError, MachineIdentification,
        MachineIdentificationUnique,
    },
    modbus::{
        ModbusDevice,
        devices::qitech_laser::{LaserDevice, LaserError},
    },
    units::{Length, length::millimeter},
};
use serde::{Deserialize, Serialize};
use std::{
    any::TypeId,
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LaserData {
    pub live_values: LiveValuesEvent,
    pub state: StateEvent,
}

impl ConvertMachineData for LaserData {
    fn to_machine_data(&self, data: &mut MachineData) -> Result<(), &'static str> {
        let serialized_bytes =
            to_slice(self, &mut data.data).map_err(|_| "Postcard serialization failed")?;
        data.type_id = TypeId::of::<Self>();
        data.length = serialized_bytes.len();
        Ok(())
    }

    fn from_machine_data(machine_data: &MachineData, out: &mut Self) -> Result<(), &'static str> {
        if machine_data.type_id != TypeId::of::<Self>() {
            return Err("Typeid Mismatch");
        }
        if machine_data.length == 0 {
            return Err("Empty buffer data");
        }
        let deserialized: Self =
            from_bytes(&machine_data.data).map_err(|_| "Postcard deserialization failed")?;
        *out = deserialized;
        Ok(())
    }
}

pub enum LaserRequestState {
    Waiting(Instant),
    NotWaiting,
}

pub struct LaserMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    mutation_counter: u64,
    namespace: LaserMachineNamespace,
    last_measurement_emit: Instant,
    last_request: Instant,
    laser: Rc<RefCell<LaserDevice>>,
    error: Option<MachineError>,
    // laser values
    diameter: Length,
    x_diameter: Option<Length>,
    y_diameter: Option<Length>,
    roundness: Option<f64>,

    target_diameter: Length,
    higher_tolerance: Length,
    lower_tolerance: Length,
    in_tolerance: bool,
    global_warning: bool,

    //laser target configuration
    laser_target: LaserTarget,

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
        let diameter = self.diameter.get::<millimeter>();
        let x_diameter = self.x_diameter.map(|x| x.get::<millimeter>());
        let y_diameter = self.y_diameter.map(|y| y.get::<millimeter>());
        let roundness = self.roundness;

        LiveValuesEvent {
            diameter,
            x_diameter,
            y_diameter,
            roundness,
        }
    }

    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(LaserEvents::LiveValues(event));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let laser = LaserState {
            higher_tolerance: self.higher_tolerance.get::<millimeter>(),
            lower_tolerance: self.lower_tolerance.get::<millimeter>(),
            target_diameter: self.laser_target.diameter.get::<millimeter>(),
            in_tolerance: self.in_tolerance,
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
                higher_tolerance: self.laser_target.higher_tolerance.get::<millimeter>(),
                lower_tolerance: self.laser_target.lower_tolerance.get::<millimeter>(),
                target_diameter: self.laser_target.diameter.get::<millimeter>(),
                in_tolerance: self.in_tolerance,
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

    pub fn set_higher_tolerance(&mut self, higher_tolerance: f64) {
        self.higher_tolerance = Length::new::<millimeter>(higher_tolerance);
        self.laser_target.higher_tolerance = self.higher_tolerance;
        self.mutation_counter += 1;
        self.emit_state();
    }

    pub fn set_lower_tolerance(&mut self, lower_tolerance: f64) {
        self.lower_tolerance = Length::new::<millimeter>(lower_tolerance);
        self.laser_target.lower_tolerance = self.lower_tolerance;
        self.mutation_counter += 1;
        self.emit_state();
    }

    pub fn set_target_diameter(&mut self, target_diameter: f64) {
        self.target_diameter = Length::new::<millimeter>(target_diameter);
        self.laser_target.diameter = Length::new::<millimeter>(target_diameter);
        self.mutation_counter += 1;
        self.emit_state();
    }

    pub fn set_global_warning(&mut self, toggle: bool) {
        self.global_warning = toggle;
        self.mutation_counter += 1;
        self.emit_state();
    }

    /// Roundness = min(x, y) / max(x, y)
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
    }

    ///
    /// Calculates if the current diameter is inside of the tolerance
    ///
    fn calculate_in_tolerance(&mut self) -> bool {
        let diameter_epsilon: f64 = 0.0001; // 0.0001 mm
        // early return true if the diameter is 0 to prevent warning happening before start
        if self.diameter.get::<millimeter>() < diameter_epsilon {
            self.in_tolerance = true;
            return true;
        }

        let top = self.target_diameter + self.higher_tolerance;
        let bottom = self.target_diameter - self.lower_tolerance;

        if self.diameter > top || self.diameter < bottom {
            self.in_tolerance = false;
        } else {
            self.in_tolerance = true;
        }

        self.in_tolerance
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
                self.x_diameter = Some(Length::new::<millimeter>(m.x_axis as f64 / 1000.0));
                self.y_diameter = Some(Length::new::<millimeter>(m.y_axis as f64 / 1000.0));
                self.diameter = Length::new::<millimeter>(m.diameter as f64 / 1000.0);
            }
            None => (),
        };
        drop(laser);
        self.roundness = self.calculate_roundness();

        if self.in_tolerance != self.calculate_in_tolerance() {
            self.did_change_state = true;
        }
    }
}

#[derive(Debug, Clone)]
pub struct LaserTarget {
    diameter: Length,
    lower_tolerance: Length,
    higher_tolerance: Length,
}

impl QiTechMachine for LaserMachine {}
