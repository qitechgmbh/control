use std::{sync::Arc, time::Instant};

use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::{
    channel::{Receiver, Sender},
    lock::RwLock,
};
use socketioxide::extract::SocketRef;

use crate::{
    AsyncThreadMessage, MACHINE_XTREM_ZEBRA, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
    serial::devices::xtrem_zebra::XtremSerial,
    xtrem_zebra::api::{
        LiveValuesEvent, StateEvent, XtremZebraEvents, XtremZebraNamespace, XtremZebraState,
    },
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct XtremZebra {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    // drivers
    xtrem_zebra: Arc<RwLock<XtremSerial>>,

    // socketio
    namespace: XtremZebraNamespace,
    last_measurement_emit: Instant,

    // scale values
    total_weight: f64,
    current_weight: f64,
    last_weight: f64,
    cycle_max_weight: f64,
    in_accumulation: bool,

    plate_counter: u32,

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
        let sockets = self.namespace.clone().unwrap().sockets.clone();
        sockets
    }
}

impl Drop for XtremZebra {
    fn drop(&mut self) {
        tracing::info!(
            "[LaserMachine::{:?}] Dropping machine and disconnecting clients...",
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
        };

        self.namespace
            .emit(XtremZebraEvents::LiveValues(live_values.build()));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let xtrem_zebra = XtremZebraState {};

        StateEvent {
            is_default_state: false,
            xtrem_zebra_state: xtrem_zebra,
        }
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            xtrem_zebra_state: XtremZebraState {},
        };

        self.namespace.emit(XtremZebraEvents::State(state.build()));
    }

    pub fn update(&mut self) {
        let xtrem_zebra_data =
            smol::block_on(async { self.xtrem_zebra.read().await.get_data().await });

        let new_weight = xtrem_zebra_data
            .as_ref()
            .map(|data| data.current_weight)
            .unwrap_or(0.0);

        // Detect accumulation start
        if !self.in_accumulation && new_weight > 0.0 {
            self.in_accumulation = true;
            self.cycle_max_weight = 0.0;
        }

        // Track maximum and display only while > 0
        if self.in_accumulation {
            if new_weight > self.cycle_max_weight {
                self.cycle_max_weight = new_weight;
            }

            // While accumulating, show current max as total
            self.total_weight = self.cycle_max_weight;

            // Detect return to 0 (end of accumulation)
            if new_weight == 0.0 && self.last_weight > 0.0 {
                self.in_accumulation = false;
                // Hide total weight when scale resets
                self.total_weight = 0.0;
            }
        }

        // Update tracking values
        self.last_weight = new_weight;
        self.current_weight = new_weight;
    }
}
