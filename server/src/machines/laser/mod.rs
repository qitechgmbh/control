use crate::machines::{MACHINE_LASER_V1, VENDOR_QITECH};

#[cfg(not(feature = "laser-mock"))]
use crate::serial::devices::laser::Laser;

#[cfg(feature = "laser-mock")]
use crate::serial::devices::mock_laser::MockLaserDevice;

use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::{
    machines::identification::MachineIdentification, socketio::namespace::NamespaceCacheingLogic,
};
use control_core_derive::Machine;
use smol::lock::RwLock;
use std::{sync::Arc, time::Instant};
use uom::si::{f64::Length, length::millimeter};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug, Machine)]
pub struct LaserMachine {
    // drivers
    #[cfg(not(feature = "laser-mock"))]
    laser: Arc<RwLock<Laser>>,
    
    #[cfg(feature = "laser-mock")]
    laser: Arc<RwLock<MockLaserDevice>>,

    // socketio
    namespace: LaserMachineNamespace,
    last_measurement_emit: Instant,

    // laser values
    diameter: Length,
    x_diameter: Option<Length>,
    y_diameter: Option<Length>,
    roundness: Option<f64>,

    //laser target configuration
    laser_target: LaserTarget,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

impl LaserMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_LASER_V1,
    };
}

impl LaserMachine {
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
            higher_tolerance: self.laser_target.higher_tolerance.get::<millimeter>(),
            lower_tolerance: self.laser_target.lower_tolerance.get::<millimeter>(),
            target_diameter: self.laser_target.diameter.get::<millimeter>(),
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
            },
        };

        self.namespace.emit(LaserEvents::State(state.build()));
    }

    pub fn set_higher_tolerance(&mut self, higher_tolerance: f64) {
        self.laser_target.higher_tolerance = Length::new::<millimeter>(higher_tolerance);
        self.emit_state();
    }

    pub fn set_lower_tolerance(&mut self, lower_tolerance: f64) {
        self.laser_target.lower_tolerance = Length::new::<millimeter>(lower_tolerance);
        self.emit_state();
    }

    pub fn set_target_diameter(&mut self, target_diameter: f64) {
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
    }
}

#[derive(Debug, Clone)]
pub struct LaserTarget {
    diameter: Length,
    lower_tolerance: Length,
    higher_tolerance: Length,
}
