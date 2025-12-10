use super::XtremZebra;
use crate::MachineAct;
use crate::MachineMessage;
use std::time::{Duration, Instant};

use beas_bsl::WeightedItem;

impl XtremZebra {
    pub fn check_for_weighted_item(&self) -> Option<WeightedItem> {
        let res = self.item_rx.try_recv();
        match res {
            Ok(item) => Some(item),
            Err(_e) => None,
        }
    }
}

impl MachineAct for XtremZebra {
    fn act(&mut self, now: Instant) {
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        self.update();

        let it = self.check_for_weighted_item();
        
        if it.is_some() {
            println!("Got New Item: {:?}", it);
            self.weighted_item = it.unwrap();
            self.emit_state();
            self.zero_counters();
            self.clear_lights();
            self.emit_state();
        }

        // IF the current one is finished check for a new one
        // let item = self.check_for_weighted_item();

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => match &mut self.namespace.namespace {
                Some(namespace) => {
                    tracing::info!("UnsubscribeNamespace");
                    namespace.socket_queue_tx.close();
                    namespace.sockets.clear();
                    namespace.events.clear();
                }
                None => todo!(),
            },
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) =>
            /*Doesn't connect to any Machine do nothing*/
            {
                ()
            }
            MachineMessage::DisconnectMachine(_machine_connection) =>
            /*Doesn't connect to any Machine do nothing*/
            {
                ()
            }
        }
    }
}
