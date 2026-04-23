use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::stepper_velocity_wago_750_671::StepperVelocityWago750671;
use smol::channel::{Receiver, Sender};

use self::api::{StateEvent, Wago671Slot1TestMachineEvents, Wago671Slot1TestMachineNamespace};
use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_671_SLOT1_TEST_MACHINE,
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago671Slot1TestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago671Slot1TestMachineNamespace,
    pub last_state_emit: Instant,
    pub last_debug_emit: Instant,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub stepper: StepperVelocityWago750671,
}

impl Machine for Wago671Slot1TestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Wago671Slot1TestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_671_SLOT1_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            enabled: self.stepper.enabled,
            target_speed: self.stepper.target_velocity_register,
            target_velocity: self.stepper.target_velocity_register,
            target_speed_steps_per_second: self.stepper.get_speed(),
            actual_velocity: self.stepper.get_actual_velocity_register(),
            actual_speed_steps_per_second: self.stepper.get_actual_speed_steps_per_second(),
            acceleration: self.stepper.get_target_acceleration(),
            freq: self.stepper.freq_range_sel,
            acc_freq: self.stepper.acc_range_sel,
            raw_position: self.stepper.get_raw_position() as i64,
            control_byte1: self.stepper.get_control_byte1(),
            control_byte2: self.stepper.get_control_byte2(),
            control_byte3: self.stepper.get_control_byte3(),
            status_byte1: self.stepper.get_status_byte1(),
            status_byte2: self.stepper.get_status_byte2(),
            status_byte3: self.stepper.get_status_byte3(),
            speed_mode_ack: self.stepper.get_s1_bit3_speed_mode_ack(),
            start_ack: self.stepper.get_s1_bit2_start_ack(),
            di1: self.stepper.get_s3_bit0(),
            di2: self.stepper.get_s3_bit1(),
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(Wago671Slot1TestMachineEvents::State(event));
    }

    pub fn set_target_speed(&mut self, speed: i16) {
        self.stepper.set_velocity(speed);
        self.emit_state();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.stepper.set_enabled(enabled);
        self.emit_state();
    }

    pub fn set_freq(&mut self, factor: u8) {
        self.stepper.set_freq_range_sel(factor);
        self.emit_state();
    }

    pub fn set_acc_freq(&mut self, factor: u8) {
        self.stepper.set_acc_range_sel(factor);
        self.emit_state();
    }

    pub fn set_acceleration(&mut self, acceleration: u16) {
        self.stepper.set_acceleration(acceleration);
        self.emit_state();
    }

    pub fn emit_debug_if_due(&mut self, now: Instant) {
        if now.duration_since(self.last_debug_emit).as_millis() < 500 {
            return;
        }
        self.last_debug_emit = now;
        let state = self.get_state();
        tracing::info!(
            "Wago671Slot1TestMachine | enabled={} target_vel={} target_speed={} actual_vel={} actual_speed={:.2} acc={} freq={} acc_freq={} raw_pos={} C1=0x{:02X} C2=0x{:02X} C3=0x{:02X} S1=0x{:02X} S2=0x{:02X} S3=0x{:02X} speed_ack={} start_ack={} di1={} di2={}",
            state.enabled,
            state.target_velocity,
            state.target_speed_steps_per_second,
            state.actual_velocity,
            state.actual_speed_steps_per_second,
            state.acceleration,
            state.freq,
            state.acc_freq,
            state.raw_position,
            state.control_byte1,
            state.control_byte2,
            state.control_byte3,
            state.status_byte1,
            state.status_byte2,
            state.status_byte3,
            state.speed_mode_ack,
            state.start_ack,
            state.di1,
            state.di2,
        );
    }
}
