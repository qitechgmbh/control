use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::channel::{Receiver, Sender};
use socketioxide::extract::SocketRef;

use crate::{
    AsyncThreadMessage, MACHINE_XTREM_ZEBRA, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
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
    //laser: Arc<RwLock<Laser>>,

    // socketio
    namespace: XtremZebraNamespace,
    last_measurement_emit: Instant,

    // scale values
    weight: f64,

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

    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let weight = self.weight;

        let live_values = LiveValuesEvent { weight };

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
        //     let laser_data = smol::block_on(async { self.laser.read().await.get_data().await });
        //     self.diameter = Length::new::<millimeter>(
        //         laser_data
        //             .as_ref()
        //             .map(|data| data.diameter.get::<millimeter>())
        //             .unwrap_or(0.0),
        //     );
        //
        //     self.x_diameter = laser_data
        //         .as_ref()
        //         .and_then(|data| data.x_axis.as_ref())
        //         .cloned();
        //
        //     self.y_diameter = laser_data
        //         .as_ref()
        //         .and_then(|data| data.y_axis.as_ref())
        //         .cloned();
        //
        //     self.roundness = self.calculate_roundness();
    }
}
