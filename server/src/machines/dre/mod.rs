use crate::serial::devices::dre::Dre;
use api::{DiameterEvent, DreEvents, DreMachineNamespace, DreStateEvent};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use smol::lock::RwLock;
use std::{sync::Arc, time::Instant};
use uom::si::{f64::Length, length::millimeter};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct DreMachine {
    // drivers
    dre: Arc<RwLock<Dre>>,

    // socketio
    namespace: DreMachineNamespace,
    last_measurement_emit: Instant,

    //dre target configuration
    dre_target: DreTarget,
}

impl Machine for DreMachine {}

impl DreMachine {
    ///diameter in mm
    pub fn emit_diameter(&mut self) {
        let diameter = smol::block_on(async {
            self.dre
                .read()
                .await
                .get_data()
                .await
                .map(|dre_data| dre_data.diameter.get::<millimeter>())
        });
        let diameter_event = DiameterEvent {
            diameter: diameter.unwrap_or(0.0),
        };
        self.namespace
            .emit_cached(DreEvents::Diameter(diameter_event.build()));
    }

    pub fn emit_dre_state(&mut self) {
        let dre_state_event = DreStateEvent {
            higher_tolerance: self.dre_target.higher_tolerance.get::<millimeter>(),
            lower_tolerance: self.dre_target.lower_tolerance.get::<millimeter>(),
            target_diameter: self.dre_target.diameter.get::<millimeter>(),
        };

        self.namespace
            .emit_cached(DreEvents::DreState(dre_state_event.build()));
    }

    pub fn target_set_higher_tolerance(&mut self, higher_tolerance: f64) {
        self.dre_target.higher_tolerance = Length::new::<millimeter>(higher_tolerance);
    }
    pub fn target_set_lower_tolerance(&mut self, lower_tolerance: f64) {
        self.dre_target.lower_tolerance = Length::new::<millimeter>(lower_tolerance);
    }
    pub fn target_set_target_diameter(&mut self, target_diameter: f64) {
        self.dre_target.diameter = Length::new::<millimeter>(target_diameter);
    }
}
#[derive(Debug, Clone)]
pub struct DreTarget {
    diameter: Length,
    lower_tolerance: Length,
    higher_tolerance: Length,
}
