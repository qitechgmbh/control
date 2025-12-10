use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use api::Configuration;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use serde::Serialize;
use smol::{
    channel::{Receiver, Sender},
    lock::RwLock,
};
use socketioxide::extract::SocketRef;

use crate::{
    AsyncThreadMessage, MACHINE_XTREM_ZEBRA, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
    serial::devices::xtrem_zebra::{XtremData, XtremSerial},
    xtrem_zebra::api::{
        LiveValuesEvent, StateEvent, XtremZebraEvents, XtremZebraNamespace, XtremZebraState,
    },
};

pub mod act;
pub mod api;
pub mod new;

use beas_bsl::{ApiConfig, WeightedItem};

#[derive(Debug)]
pub struct XtremZebra {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,

    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    // drivers
    xtrem_serial: Arc<RwLock<XtremSerial>>,

    namespace: XtremZebraNamespace,
    last_measurement_emit: Instant,

    // scale values
    total_weight: f64,
    current_weight: f64,
    last_weight: f64,
    cycle_max_weight: f64,
    in_accumulation: bool,

    plate1_target: f64,
    plate2_target: f64,
    plate3_target: f64,

    tolerance: f64,

    plate1_counter: u32,
    plate2_counter: u32,
    plate3_counter: u32,

    tare_weight: f64,
    last_raw_weight: f64,

    signal_light: SignalLight,

    weighted_item: WeightedItem,
    configuration: Configuration,

    request_tx: Sender<()>,
    item_rx: Receiver<WeightedItem>,
    config_tx: Sender<ApiConfig>,
    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

impl Machine for XtremZebra {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl XtremZebraNamespace {
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

impl Drop for XtremZebra {
    fn drop(&mut self) {
        tracing::info!(
            "[XtremZebra::{:?}] Dropping machine and disconnecting clients...",
            self.machine_identification_unique
        );
        smol::block_on(self.namespace.disconnect_all());
    }
}

impl XtremZebra {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_XTREM_ZEBRA,
    };

    pub fn emit_live_values(&mut self) {
        let live_values = LiveValuesEvent {
            total_weight: self.total_weight,
            current_weight: self.current_weight,
            plate1_counter: self.plate1_counter,
            plate2_counter: self.plate2_counter,
            plate3_counter: self.plate3_counter,
        };

        self.namespace
            .emit(XtremZebraEvents::LiveValues(live_values.build()));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let xtrem_zebra = XtremZebraState {
            plate1_target: self.plate1_target,
            plate2_target: self.plate2_target,
            plate3_target: self.plate3_target,
            tolerance: self.tolerance,
        };

        StateEvent {
            is_default_state: false,
            xtrem_zebra_state: xtrem_zebra,
            weighted_item: self.weighted_item.clone(),
            configuration: self.configuration.clone(),
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

                if (w >= self.plate1_target - self.tolerance)
                    && (w <= self.plate1_target + self.tolerance)
                {
                    self.signal_light.green_light.set(true);
                    self.signal_light.green_light_on_since = Some(Instant::now());
                    self.plate1_counter += 1;
                } else if (w >= self.plate2_target - self.tolerance)
                    && (w <= self.plate2_target + self.tolerance)
                {
                    self.signal_light.green_light.set(true);
                    self.signal_light.green_light_on_since = Some(Instant::now());
                    self.plate2_counter += 1;
                } else if (w >= self.plate3_target - self.tolerance)
                    && (w <= self.plate3_target + self.tolerance)
                {
                    self.signal_light.green_light.set(true);
                    self.signal_light.green_light_on_since = Some(Instant::now());
                    self.plate3_counter += 1;
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
        let light_duration = Duration::from_secs(10);

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

    pub fn set_plate1_target_weight(&mut self, target: f64) {
        self.plate1_target = target;
        self.emit_state();
    }
    pub fn set_plate2_target_weight(&mut self, target: f64) {
        self.plate2_target = target;
        self.emit_state();
    }
    pub fn set_plate3_target_weight(&mut self, target: f64) {
        self.plate3_target = target;
        self.emit_state();
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
    pub fn zero_counters(&mut self) {
        self.plate1_counter = 0;
        self.plate2_counter = 0;
        self.plate3_counter = 0;
        self.emit_state();
    }
    pub fn clear_lights(&mut self) {
        self.signal_light.green_light.set(false);
        self.signal_light.yellow_light.set(false);
        self.signal_light.red_light.set(false);
    }
    pub fn start(&mut self) {
        let _res = self.config_tx.try_send(ApiConfig {
            server_root: self.configuration.config_string.clone().unwrap(),
            password: self.configuration.password.clone().unwrap(),
            session_id: None,
        });
        let _res = self.request_tx.try_send(());
    }

    pub fn update(&mut self) {
        let xtrem_zebra_data =
            smol::block_on(async { self.xtrem_serial.read().await.get_data().await });
        self.calculate_weight_and_counter(xtrem_zebra_data.clone());
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
