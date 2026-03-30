use std::time::Instant;

use self::api::{StateEvent, WagoDioSeparateEvents, WagoDioSeparateNamespace};
use crate::{
    AsyncThreadMessage, MACHINE_WAGO_DIO_SEPARATE_V1, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::{digital_input::DigitalInput, digital_output::DigitalOutput};
use smol::channel::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct WagoDioSeparate {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: WagoDioSeparateNamespace,
    pub last_state_emit: Instant,
    pub inputs: [bool; 8],
    pub led_on: [bool; 8],
    pub digital_input: [DigitalInput; 8],
    pub digital_output: [DigitalOutput; 8],
}

impl Machine for WagoDioSeparate {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl WagoDioSeparate {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_WAGO_DIO_SEPARATE_V1,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            inputs: self.inputs,
            led_on: self.led_on,
        }
    }

    pub fn emit_state(&mut self) {
        for (i, di) in self.digital_input.iter().enumerate() {
            self.inputs[i] = match di.get_value() {
                Ok(v) => v,
                Err(_) => false,
            };
        }
        let event = self.get_state().build();
        self.namespace.emit(WagoDioSeparateEvents::State(event));
    }

    pub fn set_led(&mut self, index: usize, on: bool) {
        if index < self.led_on.len() {
            self.led_on[index] = on;
            self.digital_output[index].set(on);
            self.emit_state();
        }
    }

    pub fn set_all_leds(&mut self, on: bool) {
        self.led_on = [on; 8];
        for dout in self.digital_output.iter() {
            dout.set(on);
        }
        self.emit_state();
    }
}
