//use serde::{Deserialize, Serialize};


use crate::{
    MACHINE_PELLETIZER, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

use smol::{
    channel::{Receiver, Sender},
    lock::RwLock,
};
use socketioxide::extract::SocketRef;

use crate::{AsyncThreadMessage, serial::devices::us_3202510::US3202510};

use std::{sync::Arc, time::Instant};

pub mod act;
pub mod api;
pub mod new;
pub mod emit;

use crate::Machine;

use api::{PelletMachineNamespace};

#[derive(Debug)]
pub struct Pelletizer 
{
    // stuff for every machine
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    machine_identification_unique: MachineIdentificationUnique,

    emitted_default_state: bool,

    // socketio
    namespace: PelletMachineNamespace,
    last_measurement_emit: Instant,

    // machine specific
    inverter: Arc<RwLock<US3202510>>,

    mutation_request: MutationRequests
}

#[derive(Debug)]
pub struct MutationRequests
{
    run_mode:          Option<u8>,
    frequency:         Option<u16>,
    accleration_level: Option<u8>,
    decleration_level: Option<u8>,
}

impl Machine for Pelletizer
{
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique 
    {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> 
    {
        self.main_sender.clone()
    }
}

impl PelletMachineNamespace 
{
    pub async fn disconnect_all(&self) 
    {
        for socket in self.connected_sockets().await 
        {
            let _ = socket.disconnect();
        }
    }

    async fn connected_sockets(&self) -> Vec<SocketRef> 
    {
        if self.namespace.is_none()
            { vec![] }
        else
            { self.namespace.clone().unwrap().sockets.clone() }
    }
}

impl Drop for Pelletizer 
{
    fn drop(&mut self) 
    {
        tracing::info!(
            "[Pelletizer::{:?}] Dropping machine and disconnecting clients...",
            self.machine_identification_unique
        );
        smol::block_on(self.namespace.disconnect_all());
    }
}

impl Pelletizer
{
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_PELLETIZER,
    };

    pub fn set_frequency(&mut self, frequency: u16)
    {
        self.mutation_request.frequency = Some(frequency);
    }
    
    pub fn set_direction(&mut self, forward: bool)
    {
        //self.mutation_request.run_mode = Some(frequency.clamp(0, 99));
    }

    pub fn set_run_mode(&mut self, run_mode: u8)
    {
        //self.mutation_request.frequency = Some(frequency.clamp(0, 99));
    }
    
    pub fn set_acceleration_level(&mut self, acceleration_level: u8)
    {
        //self.mutation_request.accleration_level = Some(frequency.clamp(0, 99));
    }
    
    pub fn set_deceleration_level(&mut self, deceleration_level: u8)
    {
        //self.mutation_request.decleration_level = Some(frequency.clamp(0, 99));
    }
    
    pub fn update(&mut self)
    {
        //let laser_data = smol::block_on(async { self.inverter.read().await.get_data().await });
    }
}