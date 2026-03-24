// ============================================================================
// mod.rs — Core struct definition
// ============================================================================
// This file defines the machine struct and implements the `Machine` trait.
//
// FIND & REPLACE to adapt this template:
//   MyMachine      → YourMachineName  (e.g. WagoDiMachine)
//   MY_MACHINE_ID  → your constant    (e.g. WAGO_DI_MACHINE)
// ============================================================================

use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, Machine, MachineMessage, MY_MACHINE_ID, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use self::api::{MyMachineEvents, MyMachineNamespace, StateEvent};

pub mod act;
pub mod api;
pub mod new;

// ----------------------------------------------------------------------------
// Step 1 — Define your machine struct.
//
// Add the hardware handles your machine needs (DigitalOutput, DigitalInput,
// AnalogInput, etc.) as fields. The first five fields are mandatory plumbing
// for every machine — do not remove them.
// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct MyMachine {
    // --- mandatory plumbing (keep as-is) ------------------------------------
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: MyMachineNamespace,
    pub last_state_emit: Instant,

    // --- TODO: add your hardware handles here --------------------------------
    // Examples:
    //   pub douts: [DigitalOutput; 4],
    //   pub dins:  [DigitalInput; 4],
    //   pub ain:   AnalogInput,
}

// ----------------------------------------------------------------------------
// Step 2 — Implement the `Machine` trait (mandatory, do not change).
// ----------------------------------------------------------------------------
impl Machine for MyMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

// ----------------------------------------------------------------------------
// Step 3 — Set the unique machine ID constant.
//
// The constant MY_MACHINE_ID must be declared in machines/src/lib.rs and must
// be a unique u16 value not used by any other machine. See lib.rs for the
// existing values and pick the next available one.
// ----------------------------------------------------------------------------
impl MyMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MY_MACHINE_ID,
    };
}

// ----------------------------------------------------------------------------
// Step 4 — Add your machine's business logic methods here.
//
// `get_state()` and `emit_state()` are called by act.rs and api.rs — keep
// their signatures stable.  Add any other helpers your machine needs.
// ----------------------------------------------------------------------------
impl MyMachine {
    /// Build the current state snapshot (used for serialization and events).
    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            // TODO: fill with your actual state fields
        }
    }

    /// Serialize and broadcast the current state to all subscribers.
    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(MyMachineEvents::State(event));
    }

    // TODO: add domain-specific helpers, e.g.:
    //
    // pub fn set_output(&mut self, index: usize, on: bool) {
    //     self.douts[index].set(on);
    //     self.emit_state();
    // }
}
