use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::stepper_velocity_wago_750_671::StepperVelocityWago750671;
use smol::channel::{Receiver, Sender};

use self::api::{
    AxisStateEvent, StateEvent, Wago671Slot12TestMachineEvents, Wago671Slot12TestMachineNamespace,
};
use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_671_SLOT12_TEST_MACHINE,
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago671Slot12TestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago671Slot12TestMachineNamespace,
    pub last_state_emit: Instant,
    pub last_debug_emit: Instant,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub slot1: StepperVelocityWago750671,
    pub slot2: StepperVelocityWago750671,
}

impl Machine for Wago671Slot12TestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Wago671Slot12TestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_671_SLOT12_TEST_MACHINE,
    };

    fn axis_state(stepper: &StepperVelocityWago750671) -> AxisStateEvent {
        AxisStateEvent {
            enabled: stepper.enabled,
            target_speed: stepper.target_velocity_register,
            target_velocity: stepper.target_velocity_register,
            target_speed_steps_per_second: stepper.get_speed(),
            actual_velocity: stepper.get_actual_velocity_register(),
            actual_speed_steps_per_second: stepper.get_actual_speed_steps_per_second(),
            acceleration: stepper.get_target_acceleration(),
            freq: stepper.freq_range_sel,
            acc_freq: stepper.acc_range_sel,
            raw_position: stepper.get_raw_position() as i64,
            control_byte1: stepper.get_control_byte1(),
            control_byte2: stepper.get_control_byte2(),
            control_byte3: stepper.get_control_byte3(),
            status_byte1: stepper.get_status_byte1(),
            status_byte2: stepper.get_status_byte2(),
            status_byte3: stepper.get_status_byte3(),
            speed_mode_ack: stepper.get_s1_bit3_speed_mode_ack(),
            start_ack: stepper.get_s1_bit2_start_ack(),
            di1: stepper.get_s3_bit0(),
            di2: stepper.get_s3_bit1(),
        }
    }

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            slot1: Self::axis_state(&self.slot1),
            slot2: Self::axis_state(&self.slot2),
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(Wago671Slot12TestMachineEvents::State(event));
    }

    fn axis_mut(&mut self, axis: u8) -> Option<&mut StepperVelocityWago750671> {
        match axis {
            1 => Some(&mut self.slot1),
            2 => Some(&mut self.slot2),
            _ => None,
        }
    }

    pub fn set_target_speed(&mut self, axis: u8, speed: i16) {
        if let Some(stepper) = self.axis_mut(axis) {
            stepper.set_velocity(speed);
            self.emit_state();
        }
    }

    pub fn set_enabled(&mut self, axis: u8, enabled: bool) {
        if let Some(stepper) = self.axis_mut(axis) {
            stepper.set_enabled(enabled);
            self.emit_state();
        }
    }

    pub fn set_freq(&mut self, axis: u8, factor: u8) {
        if let Some(stepper) = self.axis_mut(axis) {
            stepper.set_freq_range_sel(factor);
            self.emit_state();
        }
    }

    pub fn set_acc_freq(&mut self, axis: u8, factor: u8) {
        if let Some(stepper) = self.axis_mut(axis) {
            stepper.set_acc_range_sel(factor);
            self.emit_state();
        }
    }

    pub fn set_acceleration(&mut self, axis: u8, acceleration: u16) {
        if let Some(stepper) = self.axis_mut(axis) {
            stepper.set_acceleration(acceleration);
            self.emit_state();
        }
    }

    pub fn emit_debug_if_due(&mut self, now: Instant) {
        if now.duration_since(self.last_debug_emit).as_millis() < 500 {
            return;
        }
        self.last_debug_emit = now;
        let state = self.get_state();
        tracing::info!(
            "Wago671Slot12TestMachine | slot1 enabled={} target_vel={} actual_vel={} acc={} raw_pos={} C1=0x{:02X} C2=0x{:02X} C3=0x{:02X} S1=0x{:02X} S2=0x{:02X} S3=0x{:02X} speed_ack={} start_ack={} | slot2 enabled={} target_vel={} actual_vel={} acc={} raw_pos={} C1=0x{:02X} C2=0x{:02X} C3=0x{:02X} S1=0x{:02X} S2=0x{:02X} S3=0x{:02X} speed_ack={} start_ack={}",
            state.slot1.enabled,
            state.slot1.target_velocity,
            state.slot1.actual_velocity,
            state.slot1.acceleration,
            state.slot1.raw_position,
            state.slot1.control_byte1,
            state.slot1.control_byte2,
            state.slot1.control_byte3,
            state.slot1.status_byte1,
            state.slot1.status_byte2,
            state.slot1.status_byte3,
            state.slot1.speed_mode_ack,
            state.slot1.start_ack,
            state.slot2.enabled,
            state.slot2.target_velocity,
            state.slot2.actual_velocity,
            state.slot2.acceleration,
            state.slot2.raw_position,
            state.slot2.control_byte1,
            state.slot2.control_byte2,
            state.slot2.control_byte3,
            state.slot2.status_byte1,
            state.slot2.status_byte2,
            state.slot2.status_byte3,
            state.slot2.speed_mode_ack,
            state.slot2.start_ack,
        );
    }
}
