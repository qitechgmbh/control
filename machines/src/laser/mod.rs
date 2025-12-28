use crate::serial::devices::laser::Laser;
use crate::{
    MACHINE_LASER_V1, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use crate::{Machine, MachineMessage};
use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::{
    channel::{Receiver, Sender},
    lock::RwLock,
};
use socketioxide::extract::SocketRef;
use units::Length;

use crate::AsyncThreadMessage;
use std::{sync::Arc, time::Instant};
use units::length::millimeter;

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct LaserMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    // drivers
    laser: Arc<RwLock<Laser>>,

    // socketio
    namespace: LaserMachineNamespace,
    last_measurement_emit: Instant,

    // laser values
    diameter: Length,
    x_diameter: Option<Length>,
    y_diameter: Option<Length>,
    roundness: Option<f64>,

    target_diameter: Length,
    higher_tolerance: Length,
    lower_tolerance: Length,
    in_tolerance: bool,

    //laser target configuration
    laser_target: LaserTarget,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
    did_change_state: bool,
}

impl Machine for LaserMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl LaserMachineNamespace {
    pub async fn disconnect_all(&self) {
        for socket in self.connected_sockets().await {
            let _ = socket.disconnect();
        }
    }

    async fn connected_sockets(&self) -> Vec<SocketRef> {
        if self.namespace.is_none() {
            return vec![];
        }
        let sockets = self.namespace.clone().unwrap().sockets.clone();
        sockets
    }
}

impl Drop for LaserMachine {
    fn drop(&mut self) {
        tracing::info!(
            "[LaserMachine::{:?}] Dropping machine and disconnecting clients...",
            self.machine_identification_unique
        );
        smol::block_on(self.namespace.disconnect_all());
    }
}

impl LaserMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_LASER_V1,
    };

    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let diameter = self.diameter.get::<millimeter>();
        let x_diameter = self.x_diameter.map(|x| x.get::<millimeter>());
        let y_diameter = self.y_diameter.map(|y| y.get::<millimeter>());
        let roundness = self.roundness;

        let live_values = LiveValuesEvent {
            diameter,
            x_diameter,
            y_diameter,
            roundness,
        };

        self.namespace
            .emit(LaserEvents::LiveValues(live_values.build()));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let laser = LaserState {
            higher_tolerance: self.higher_tolerance.get::<millimeter>(),
            lower_tolerance: self.lower_tolerance.get::<millimeter>(),
            target_diameter: self.laser_target.diameter.get::<millimeter>(),
            in_tolerance: self.in_tolerance,
        };

        StateEvent {
            is_default_state: false,
            laser_state: laser,
        }
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            laser_state: LaserState {
                higher_tolerance: self.laser_target.higher_tolerance.get::<millimeter>(),
                lower_tolerance: self.laser_target.lower_tolerance.get::<millimeter>(),
                target_diameter: self.laser_target.diameter.get::<millimeter>(),
                in_tolerance: self.in_tolerance,
            },
        };

        self.namespace.emit(LaserEvents::State(state.build()));
        self.did_change_state = false;
    }

    pub fn set_higher_tolerance(&mut self, higher_tolerance: f64) {
        self.higher_tolerance = Length::new::<millimeter>(higher_tolerance);
        self.laser_target.higher_tolerance = self.higher_tolerance;
        self.emit_state();
    }

    pub fn set_lower_tolerance(&mut self, lower_tolerance: f64) {
        self.lower_tolerance = Length::new::<millimeter>(lower_tolerance);
        self.laser_target.lower_tolerance = self.lower_tolerance;
        self.emit_state();
    }

    pub fn set_target_diameter(&mut self, target_diameter: f64) {
        self.target_diameter = Length::new::<millimeter>(target_diameter);
        self.laser_target.diameter = Length::new::<millimeter>(target_diameter);
        self.emit_state();
    }

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
        let laser_data = smol::block_on(async { self.laser.read().await.get_data().await });
        self.diameter = Length::new::<millimeter>(
            laser_data
                .as_ref()
                .map(|data| data.diameter.get::<millimeter>())
                .unwrap_or(0.0),
        );

        self.x_diameter = laser_data
            .as_ref()
            .and_then(|data| data.x_axis.as_ref())
            .cloned();

        self.y_diameter = laser_data
            .as_ref()
            .and_then(|data| data.y_axis.as_ref())
            .cloned();

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
