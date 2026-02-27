use std::time::Duration;
use std::{fmt::Debug, time::Instant};
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, 
    Machine, 
    MachineAct, 
    MachineConnection, 
    MachineMessage, 
    MachineValues, 
    VENDOR_QITECH
};

use crate::machine_identification::{
    MachineIdentification, 
    MachineIdentificationUnique
};

use super::machine_base::{
    MachineImpl,
    MAX_CONNECTIONS,
    MAX_MESSAGES,
    MACHINE_ID,
};

#[derive(Debug)]
pub struct MachineBase 
{
    pub machine_uid:  MachineIdentificationUnique,
    pub main_sender:  Option<Sender<AsyncThreadMessage>>,
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender:   Sender<MachineMessage>,

    connected_machines_buf: [MachineConnection; MAX_CONNECTIONS],
    connected_machines_len: usize,

    // socketio
    pub namespace: Namespace,
    pub last_measurement_emit: Instant,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    pub emitted_default_state: bool,
}

impl MachineBase
{
    pub fn new()
    {

    }
}

impl MachineImpl
{
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification 
    {
        vendor:  VENDOR_QITECH,
        machine: MACHINE_ID,
    };
}

impl Machine for MachineImpl 
{
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique 
    {
        self.base.machine_uid.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> 
    {
        self.base.main_sender.clone()
    }
}

impl MachineAct for MachineImpl 
{
    fn act(&mut self, now: Instant) 
    {
        for _ in 0..MAX_MESSAGES 
        {
            match self.base.api_receiver.try_recv() 
            {
                Ok(msg) => self.act_machine_message(msg),
                Err(_) => break,
            }
        }

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.base.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) 
        {
            self.emit_live_values();
            self.base.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) 
    {
        match msg 
        {
            MachineMessage::SubscribeNamespace(namespace) => 
            {
                self.base.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.base.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => 
            {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(machine_connection) => 
            {
                if self.base.connected_machines_len >= MAX_CONNECTIONS 
                {
                    tracing::debug!(
                        "Refusing to add Machine Connection {:?}, reached limit of {:?}",
                        machine_connection,
                        MAX_CONNECTIONS
                    );
                    return;
                }

                todo!("Add connection")
                //self.connected_machines.push(machine_connection);
            }
            MachineMessage::DisconnectMachine(_machine_connection) => 
            {
                todo!("Remove proper connection")
                //self.connected_machines.clear();
            }
            MachineMessage::RequestValues(sender) => 
            {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.get_state_event())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
                sender.close();
            }
            MachineMessage::ReceiveLiveValues(any_live_values) => 
            {
                todo!("Call impl function");
            }
        }
    }
}