use std::{sync::Arc, time::Instant};
use api::{DiameterEvent, DreEvents, DreMachineNamespace};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use smol::lock::RwLock;
use uom::si::f64::Length;
use crate::serial::devices::dre::Dre;

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
    dre_target: DreTarget
}

impl Machine for DreMachine {}

impl DreMachine{
    pub async fn emit_dre_data(&mut self) {
        let dre_data = self.dre.read().await.get_data().await;

        let diameter_event = DiameterEvent{
            dre_data
        };
        self.namespace.emit_cached(DreEvents::DiameterEvent(diameter_event.build()));
    }
    pub fn target_set_higher_tolerance(&mut self, higher_tolerance:Length){
        self.dre_target.higher_tolerance=higher_tolerance;
    }
    pub fn target_set_lower_tolerance(&mut self, lower_tolerance:Length){
        self.dre_target.lower_tolerance=lower_tolerance;
    }
    pub fn target_set_target_diameter(&mut self, target_diameter:Length){
        self.dre_target.diameter=target_diameter;
    }
}
#[derive(Debug,Clone)]
pub struct DreTarget{
    diameter: Length,
    lower_tolerance: Length,
    higher_tolerance: Length
}