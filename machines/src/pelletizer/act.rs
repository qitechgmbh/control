use std::time::{Duration, Instant};

use units::frequency::hertz;

use crate::MachineAct;
use crate::MachineMessage;
use crate::MachineValues;

use super::Pelletizer;

impl MachineAct for Pelletizer 
{
    fn act(&mut self, now: Instant) 
    {
        if let Ok(msg) = self.api_receiver.try_recv() 
        {
            self.act_machine_message(msg);
        };

        let should_emit =
            now.duration_since(self.last_measurement_emit)
                > Duration::from_secs_f64(1.0 / 30.0);

        {
            let mut inverter = smol::block_on(async {
                self.inverter.write().await
            });

            if let Some(value) = self.mutation_request.frequency 
            {
                inverter.set_frequency_target(units::Frequency::new::<hertz>(value as f64));
                self.mutation_request.frequency = None;
            }

            if should_emit {
                // inverter.refresh_status();
            }

            inverter.update();
        } // drop lock

        if should_emit 
        {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) 
    {
        match msg 
        {
            MachineMessage::SubscribeNamespace(namespace) => 
            {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            
            MachineMessage::UnsubscribeNamespace => match &mut self.namespace.namespace 
            {
                Some(namespace) => 
                {
                    tracing::info!("UnsubscribeNamespace");
                    namespace.socket_queue_tx.close();
                    namespace.sockets.clear();
                    namespace.events.clear();
                }
                None => todo!(),
            },
            MachineMessage::HttpApiJsonRequest(value) => 
            {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection)  => {}
            MachineMessage::DisconnectMachine(_machine_connection) => {}
            MachineMessage::RequestValues(sender) => 
            {
                tracing::error!("REQUESTED VALUES");
                
                let state = serde_json::to_value(self.create_state_event()).expect("Failed to serialize state");
                
                let live_values = serde_json::to_value(self.create_live_values_event()).expect("Failed to serialize live values");

                sender.send_blocking(MachineValues{ state, live_values }).expect("Failed to send values");
                sender.close();
            }
        }
    }
}