use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::{digital_input::DigitalInput, digital_output::DigitalOutput, stepper_velocity_wago_750_672::StepperVelocityWago750672};
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, Machine, MachineMessage, TEST_MACHINE_BOTTLECAPS, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use self::api::{BottlecapsTestMachineEvents, BottlecapsTestMachineNamespace, StateEvent};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct BottlecapsTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: BottlecapsTestMachineNamespace,
    pub last_state_emit: Instant,

    pub stepper: StepperVelocityWago750672,
    pub douts: [DigitalOutput; 8],
    pub dins: [DigitalInput; 8],
}

impl Machine for BottlecapsTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl BottlecapsTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: TEST_MACHINE_BOTTLECAPS,
    };
}

// ----------------------------------------------------------------------------
// Step 4 — Add your machine's business logic methods here.
//
// `get_state()` and `emit_state()` are called by act.rs and api.rs — keep
// their signatures stable.  Add any other helpers your machine needs.
// ----------------------------------------------------------------------------
impl BottlecapsTestMachine {
    /// Build the current state snapshot (used for serialization and events).
    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            // TODO: fill with your actual state fields
        }
    }

    /// Serialize and broadcast the current state to all subscribers.
    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(BottlecapsTestMachineEvents::State(event));
    }

    // TODO: add domain-specific helpers, e.g.:
    //
    // pub fn set_output(&mut self, index: usize, on: bool) {
    //     self.douts[index].set(on);
    //     self.emit_state();
    // }
}
