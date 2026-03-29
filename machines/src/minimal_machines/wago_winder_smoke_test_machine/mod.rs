use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::{
    io::{
        digital_output::DigitalOutput,
        stepper_velocity_wago_750_671::{StepperVelocityWago750671, Wago750671Mode},
    },
};
use smol::channel::{Receiver, Sender};

use self::api::{
    AxisState, StateEvent, WagoWinderSmokeTestMachineEvents, WagoWinderSmokeTestMachineNamespace,
};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_WINDER_SMOKE_TEST_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct WagoWinderSmokeTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: WagoWinderSmokeTestMachineNamespace,
    pub last_state_emit: Instant,
    pub steppers: [StepperVelocityWago750671; 1],
    pub digital_output1: DigitalOutput,
    pub digital_output2: DigitalOutput,
}

impl Machine for WagoWinderSmokeTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl WagoWinderSmokeTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_WINDER_SMOKE_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let axes = std::array::from_fn(|i| AxisState {
            enabled: self.steppers[i].enabled,
            target_velocity: self.steppers[i].target_velocity,
            target_acceleration: self.steppers[i].target_acceleration,
            freq_range_sel: self.steppers[i].freq_range_sel,
            acc_range_sel: self.steppers[i].acc_range_sel,
            mode: self.steppers[i].get_mode().map(|mode| match mode {
                Wago750671Mode::PrimaryApplication => "PrimaryApplication",
                Wago750671Mode::Program => "Program",
                Wago750671Mode::Reference => "Reference",
                Wago750671Mode::Jog => "Jog",
                Wago750671Mode::Mailbox => "Mailbox",
            }.to_string()),
            speed_mode_ack: self.steppers[i].get_s1_bit3_speed_mode_ack(),
            di1: self.steppers[i].get_s3_bit0(),
            di2: false,
            status_byte1: self.steppers[i].get_status_byte1(),
            status_byte2: self.steppers[i].get_status_byte2(),
            status_byte3: self.steppers[i].get_status_byte3(),
        });

        StateEvent {
            axes,
            digital_output1: self.digital_output1.get(),
            digital_output2: self.digital_output2.get(),
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(WagoWinderSmokeTestMachineEvents::State(event));
    }

    pub fn set_stepper_enabled(&mut self, axis: usize, enabled: bool) {
        if let Some(stepper) = self.steppers.get_mut(axis) {
            stepper.set_enabled(enabled);
        }
        self.emit_state();
    }

    pub fn set_stepper_velocity(&mut self, axis: usize, velocity: i16) {
        if let Some(stepper) = self.steppers.get_mut(axis) {
            stepper.set_velocity(velocity);
        }
        self.emit_state();
    }

    pub fn set_stepper_freq_range(&mut self, axis: usize, factor: u8) {
        if let Some(stepper) = self.steppers.get_mut(axis) {
            stepper.set_freq_range_sel(factor);
        }
        self.emit_state();
    }

    pub fn set_stepper_acc_range(&mut self, axis: usize, factor: u8) {
        if let Some(stepper) = self.steppers.get_mut(axis) {
            stepper.set_acc_range_sel(factor);
        }
        self.emit_state();
    }

    pub fn set_digital_output(&mut self, port: usize, value: bool) {
        match port {
            1 => self.digital_output1.set(value.into()),
            2 => self.digital_output2.set(value.into()),
            _ => {}
        }
        self.emit_state();
    }
}
