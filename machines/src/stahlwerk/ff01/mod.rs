use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::{
    channel::{Receiver, Sender},
    lock::RwLock,
};
use socketioxide::extract::SocketRef;

use stahlwerk_extension::ff01::{
    Entry, ProxyClient, ProxyTransactionError, Request, Response
};
use stahlwerk_extension::ClientConfig;

use crate::{
    AsyncThreadMessage, MACHINE_FF01, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
    serial::devices::xtrem_zebra::{XtremData, XtremSerial},
};

use api::{
    LiveValuesEvent, StateEvent, XtremZebraEvents, FF01Namespace, XtremZebraState,
};

pub mod act;
pub mod api;
pub mod new;
mod comm;

#[derive(Debug)]
pub struct FF01 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,

    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    // drivers
    xtrem_serial: Arc<RwLock<XtremSerial>>,

    namespace: FF01Namespace,
    last_measurement_emit: Instant,

    // scale values
    total_weight: f64,
    current_weight: f64,
    last_weight: f64,
    cycle_max_weight: f64,
    in_accumulation: bool,

    tolerance: f64,

    plate_counter: u32,

    tare_weight: f64,
    last_raw_weight: f64,

    signal_light: SignalLight,

    // api stuff
    entry: Option<Entry>,
    client: ProxyClient,

    last_request_ts: Instant,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    _emitted_default_state: bool,
}

impl Machine for FF01 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl FF01Namespace {
    pub async fn disconnect_all(&self) {
        for socket in self.connected_sockets().await {
            let _ = socket.disconnect();
        }
    }

    async fn connected_sockets(&self) -> Vec<SocketRef> {
        if self.namespace.is_none() {
            return vec![];
        }
        self.namespace.clone().unwrap().sockets.clone()
    }
}

impl Drop for FF01 {
    fn drop(&mut self) {
        tracing::info!(
            "[XtremZebra::{:?}] Dropping machine and disconnecting clients...",
            self.machine_identification_unique
        );
        smol::block_on(self.namespace.disconnect_all());
    }
}

impl FF01 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_FF01,
    };

    pub fn emit_live_values(&mut self) {
        let live_values = LiveValuesEvent {
            total_weight: self.total_weight,
            current_weight: self.current_weight,
            plate_counter: self.plate_counter,
        };

        self.namespace
            .emit(XtremZebraEvents::LiveValues(live_values.build()));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let xtrem_zebra = XtremZebraState {
            tolerance: self.tolerance,
        };

        StateEvent {
            is_default_state: false,
            xtrem_zebra_state: xtrem_zebra,
            entry: self.entry.clone(),
        }
    }

    pub fn emit_state(&mut self) {
        let state = self.build_state_event();
        self.namespace.emit(XtremZebraEvents::State(state.build()));
    }

    fn calculate_weight_and_counter(&mut self, data: Option<XtremData>) {
        let raw_weight = data.as_ref().map(|d| d.current_weight).unwrap_or(0.0);
        self.last_raw_weight = raw_weight;

        let new_weight = (raw_weight - self.tare_weight).max(0.0);

        let was_zero = self.last_weight == 0.0 && new_weight > 0.0;
        let is_zero = new_weight == 0.0 && self.last_weight > 0.0;

        if !self.in_accumulation && was_zero {
            self.in_accumulation = true;
            self.cycle_max_weight = new_weight;
        }

        if self.in_accumulation {
            if new_weight > self.cycle_max_weight {
                self.cycle_max_weight = new_weight;
            }

            self.total_weight = self.cycle_max_weight;

            if is_zero {
                self.in_accumulation = false;
                let w = self.cycle_max_weight;

                if (w >= self.weighted_item.weight as f64 - self.tolerance)
                    && (w <= self.weighted_item.weight as f64 + self.tolerance)
                {
                    self.signal_light.green_light.set(true);
                    self.signal_light.green_light_on_since = Some(Instant::now());
                    self.plate_counter += 1;
                } else {
                    self.signal_light.red_light.set(true);
                    self.signal_light.red_light_on_since = Some(Instant::now());
                }

                self.total_weight = 0.0;
                self.cycle_max_weight = 0.0;
            }
        } else {
            self.total_weight = 0.0;
        }

        let now = Instant::now();
        let light_duration = Duration::from_millis(500);

        if let Some(t) = self.signal_light.green_light_on_since {
            if now.duration_since(t) > light_duration {
                self.signal_light.green_light.set(false);
                self.signal_light.green_light_on_since = None;
            }
        }

        if let Some(t) = self.signal_light.red_light_on_since {
            if now.duration_since(t) > light_duration {
                self.signal_light.red_light.set(false);
                self.signal_light.red_light_on_since = None;
            }
        }

        self.last_weight = new_weight;
        self.current_weight = new_weight;
    }

    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
        self.emit_state();
    }

    pub fn set_tare(&mut self) {
        self.tare_weight = self.last_raw_weight;
        self.cycle_max_weight = 0.0;
        self.total_weight = 0.0;
        self.current_weight = 0.0;
        self.in_accumulation = false;
        self.emit_state();
    }

    pub fn clear_lights(&mut self) {
        self.signal_light.green_light.set(false);
        self.signal_light.yellow_light.set(false);
        self.signal_light.red_light.set(false);
    }

    pub fn update(&mut self) {
        let xtrem_zebra_data =
            smol::block_on(async { 
                self.xtrem_serial.read().await.get_data().await 
            });
        
        // we have pending request
        if !self.client.can_queue_request() {
            match self.client.poll_response() {
                Ok(response) => self.handle_response(response),
                Err(e) => {
                    match e {
                        ProxyTransactionError::NoPendingRequest => unreachable!(),
                        ProxyTransactionError::Pending => {},
                        ProxyTransactionError::ChannelFull => todo!(),
                        ProxyTransactionError::ChannelClosed => todo!(),
                        ProxyTransactionError::TagMismatch => todo!(),
                        ProxyTransactionError::Response(response_error) => todo!(),
                    }
                },
            }
        }

        match self.entry 
        {
            Some(entry) => {

            }
            None => {
                // Has no entry, check for a new one
                if self.client.can_queue_request() {
                    _ = self.client
                        .queue_request(Request::GetNextEntry)
                        .expect("Should be able to enqueue");
                }

                match self.client.poll_response() {
                    Ok(response) => {

                    },
                    Err(_) => {

                    },
                }
            }
        }

        self.calculate_weight_and_counter(xtrem_zebra_data.clone());
    }

    fn handle_response(&mut self, response: Response)
    {
        use Response::*;

        match response {
            GetNextEntry(entry) => self.entry = entry,
            GetScrapQuantity(quantity) => {
                let Some(entry) = &mut self.entry else {
                    tracing::error!("Entry is none but should be Some");
                    return;
                };

                entry.scrap_quantity = quantity.unwrap_or(0.0);
            },
            Finalize => {
                self.entry = None;
                self.plate_counter = 0;
            },
        }
    }
}

#[derive(Debug)]
struct SignalLight {
    green_light: DigitalOutput,
    green_light_on_since: Option<Instant>,
    yellow_light: DigitalOutput,
    _yellow_light_on_since: Option<Instant>,
    red_light: DigitalOutput,
    red_light_on_since: Option<Instant>,
    _beeper: DigitalOutput,
}
