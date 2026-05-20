// ============================================================================
// mod.rs — Core struct definition + Machine trait + business logic
// ============================================================================
// FIND & REPLACE to adapt this template:
//   MyMachine      → YourMachineName  (e.g. Wago750_460Machine)
//   MY_MACHINE_ID  → your constant    (e.g. WAGO_750_460_MACHINE)
//   my_machine     → your_machine     (used for namespace types — match dir name)
//
// Files in this directory:
//   mod.rs — this file: struct, MACHINE_IDENTIFICATION, helper methods
//   new.rs — MachineNew trait: hardware initialization, called once at startup
//   act.rs — Machine trait: act(), react(), get_identification()
//   api.rs — MachineApi trait + StateEvent + Mutation + Namespace
// ============================================================================

use std::time::Instant;

use self::api::{MyMachineEvents, MyMachineNamespace, StateEvent};
use crate::{MY_MACHINE_ID, MachineMessage, QiTechMachine, VENDOR_QITECH};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

// ----------------------------------------------------------------------------
// Step 1 — Define your machine struct.
//
// The first 5 fields are mandatory plumbing — every minimal machine has them.
// Add your hardware handles + any per-machine state below.
//
// Hardware handle conventions:
//   - Beckhoff terminal:        Rc<RefCell<EL2004>>           or
//                               Rc<RefCell<dyn DigitalOutputDevice>>  (trait obj)
//   - WAGO module (subdevice):  Box<Wago750_530>              (owned by machine)
// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct MyMachine {
    // --- mandatory plumbing -------------------------------------------------
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: MyMachineNamespace,
    pub last_state_emit: Instant,
    // --- TODO: add hardware handles + state fields here ---------------------
    // pub digital_output_device: Rc<RefCell<dyn DigitalOutputDevice>>,
    // pub last_output_state: [bool; 4],
}

impl QiTechMachine for MyMachine {}

// ----------------------------------------------------------------------------
// Step 2 — Bind MACHINE_IDENTIFICATION.
//
// MY_MACHINE_ID must be declared in machine_implementations/src/lib.rs and
// must be a unique u16. See lib.rs and pick the next free hex value.
// ----------------------------------------------------------------------------
impl MyMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MY_MACHINE_ID,
    };

    // ------------------------------------------------------------------------
    // Step 3 — Build the state snapshot used for emit + HTTP polling.
    // Called from act.rs (periodic) and api.rs (RequestValues).
    // ------------------------------------------------------------------------
    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            // TODO: fill from your hardware + struct state
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(MyMachineEvents::State(event));
    }

    // ------------------------------------------------------------------------
    // Step 4 — Add domain-specific helpers here.
    //
    // Helpers usually mutate a hardware handle then call `self.emit_state()`
    // so subscribers see the change immediately.
    //
    // pub fn set_output(&mut self, index: usize, value: bool) {
    //     self.digital_output_device.borrow_mut().set_output(index, value);
    //     self.last_output_state[index] = value;
    //     self.emit_state();
    // }
    // ------------------------------------------------------------------------
}
