use super::VacuumMachine;
use crate::{MachineAct, MachineMessage, MachineValues};
use std::{ops::Add, time::{Duration, Instant}};

impl MachineAct for VacuumMachine 
{
    fn act(&mut self, now: Instant) 
    {
        if let Ok(msg) = self.api_receiver.try_recv() 
        {
            self.act_machine_message(msg);
        }

        // Emit state at 30 Hz
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) 
        {

            match self.mode 
            {
                super::Mode::Idle => {},
                super::Mode::On => {},
                super::Mode::Auto => {},
                super::Mode::Interval => 
                {


                    match self.interval_state
                    {
                        true =>
                        {
                            if Instant::now() >= self.interval_expiry 
                            {
                                self.interval_expiry = Instant::now().add(Duration::from_secs_f64(self.interval_time_off));
                                self.interval_state = false;
                                self.set_running(false);
                            }
                        },
                        false => 
                        {
                            if Instant::now() >= self.interval_expiry 
                            {
                                self.interval_expiry = Instant::now().add(Duration::from_secs_f64(self.interval_time_on));
                                self.interval_state = true;
                                self.set_running(true);
                            }
                        },
                    }
                },
            }

            self.emit_state();
            self.last_state_emit = now;
        }

        // Emit live values at 10 Hz
        if now.duration_since(self.last_live_values_emit) > Duration::from_secs_f64(1.0 / 10.0) 
        {
            self.emit_live_values();
            self.last_live_values_emit = now;
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
                self.emit_live_values();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => 
            {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => 
            {
                // Does not connect to any Machine; do nothing
            }
            MachineMessage::DisconnectMachine(_machine_connection) => 
            {
                // Does not connect to any Machine; do nothing
            }
            MachineMessage::RequestValues(sender) => 
            {
                sender
                    .send_blocking(MachineValues 
                    {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
                    
                sender.close();
            }
        }
    }
}
