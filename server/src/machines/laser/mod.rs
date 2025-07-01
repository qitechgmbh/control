use crate::serial::devices::laser::Laser;
use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use smol::lock::RwLock;
use std::{sync::Arc, time::Instant};
use uom::si::{f64::Length, length::millimeter};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct LaserMachine {
    // drivers
    laser: Arc<RwLock<Laser>>,

    // socketio
    namespace: LaserMachineNamespace,
    last_measurement_emit: Instant,

    //laser target configuration
    laser_target: LaserTarget,
}

impl Machine for LaserMachine {}

impl LaserMachine {
    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let diameter = smol::block_on(async {
            self.laser
                .read()
                .await
                .get_data()
                .await
                .map(|laser_data| laser_data.diameter.get::<millimeter>())
        });
        let live_values = LiveValuesEvent {
            diameter: diameter.unwrap_or(0.0),
        };
        self.namespace
            .emit(LaserEvents::LiveValues(live_values.build()));
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            laser_state: LaserState {
                higher_tolerance: self.laser_target.higher_tolerance.get::<millimeter>(),
                lower_tolerance: self.laser_target.lower_tolerance.get::<millimeter>(),
                target_diameter: self.laser_target.diameter.get::<millimeter>(),
            },
        };

        self.namespace
            .emit(LaserEvents::State(state.build()));
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
}
#[derive(Debug, Clone)]
pub struct LaserTarget {
    diameter: Length,
    lower_tolerance: Length,
    higher_tolerance: Length,
}
