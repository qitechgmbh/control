use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::stepper_velocity_wago_750_671::{
    S1Flag, S2Flag, S3Flag, StatusByteS1, StatusByteS2, StatusByteS3, StepperVelocityWago750671,
    Wago750671Mode,
};
use smol::channel::{Receiver, Sender};

use self::api::{
    StateEvent, WagoWinderSmokeTestMachineEvents, WagoWinderSmokeTestMachineNamespace,
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
    pub stepper: StepperVelocityWago750671,
    pub last_debug_snapshot: Option<String>,
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
    const COARSE_SEEK_REGISTER: i16 = 5000;
    const STEP_ACCELERATION: u16 = 1000;

    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_WINDER_SMOKE_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let status_byte1 = self.stepper.get_status_byte1();
        let status_byte2 = self.stepper.get_status_byte2();
        let status_byte3 = self.stepper.get_status_byte3();
        let s1 = StatusByteS1::from_bits(status_byte1);
        let s2 = StatusByteS2::from_bits(status_byte2);
        let s3 = StatusByteS3::from_bits(status_byte3);

        StateEvent {
            enabled: self.stepper.enabled,
            target_velocity: self.stepper.target_velocity_register,
            actual_velocity: self.stepper.get_actual_velocity_register(),
            target_acceleration: self.stepper.target_acceleration,
            freq_range_sel: self.stepper.freq_range_sel,
            acc_range_sel: self.stepper.acc_range_sel,
            mode: self.stepper.get_mode().map(|mode| {
                match mode {
                    Wago750671Mode::PrimaryApplication => "PrimaryApplication",
                    Wago750671Mode::Positioning => "Positioning",
                    Wago750671Mode::Program => "Program",
                    Wago750671Mode::Reference => "Reference",
                    Wago750671Mode::Jog => "Jog",
                    Wago750671Mode::Mailbox => "Mailbox",
                }
                .to_string()
            }),
            ready: s1.has_flag(S1Flag::Ready),
            stop2n_ack: s1.has_flag(S1Flag::Stop2NAck),
            start_ack: s1.has_flag(S1Flag::StartAck),
            speed_mode_ack: self.stepper.get_s1_bit3_speed_mode_ack(),
            standstill: s2.has_flag(S2Flag::StandStill),
            on_speed: s2.has_flag(S2Flag::OnSpeed),
            direction_positive: s2.has_flag(S2Flag::Direction),
            error: s2.has_flag(S2Flag::Error),
            reset: s3.has_flag(S3Flag::Reset),
            position: self.stepper.get_position() as i64,
            raw_position: self.stepper.get_raw_position() as i64,
            di1: self.stepper.get_s3_bit0(),
            di2: self.stepper.get_s3_bit1(),
            status_byte1,
            status_byte2,
            status_byte3,
            control_byte1: self.stepper.get_control_byte1(),
            control_byte2: self.stepper.get_control_byte2(),
            control_byte3: self.stepper.get_control_byte3(),
        }
    }

    pub fn emit_state(&mut self) {
        let state = self.get_state();
        let debug_snapshot = format!(
            "enabled={} target_vel={} actual_vel={} acc={} mode={:?} ready={} stop2n_ack={} start_ack={} speed_ack={} standstill={} on_speed={} dir_pos={} error={} reset={} di1={} di2={} pos={} raw_pos={} c1=0x{:02X} c2=0x{:02X} c3=0x{:02X} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X}",
            state.enabled,
            state.target_velocity,
            state.actual_velocity,
            state.target_acceleration,
            state.mode,
            state.ready,
            state.stop2n_ack,
            state.start_ack,
            state.speed_mode_ack,
            state.standstill,
            state.on_speed,
            state.direction_positive,
            state.error,
            state.reset,
            state.di1,
            state.di2,
            state.position,
            state.raw_position,
            state.control_byte1,
            state.control_byte2,
            state.control_byte3,
            state.status_byte1,
            state.status_byte2,
            state.status_byte3,
        );
        if self.last_debug_snapshot.as_ref() != Some(&debug_snapshot) {
            tracing::info!("WagoWinderSmokeTest | {}", debug_snapshot);
            self.last_debug_snapshot = Some(debug_snapshot);
        }
        let event = state.build();
        self.namespace
            .emit(WagoWinderSmokeTestMachineEvents::State(event));
    }

    pub fn set_stepper_enabled(&mut self, enabled: bool) {
        self.stepper.set_enabled(enabled);
        self.emit_state();
    }

    pub fn set_stepper_velocity(&mut self, velocity: i16) {
        self.stepper.set_velocity_register(velocity);
        self.emit_state();
    }

    pub fn set_stepper_position(&mut self, position: i64) {
        self.stepper.set_position(position as i128);
        self.emit_state();
    }

    pub fn set_stepper_freq_range(&mut self, factor: u8) {
        self.stepper.set_freq_range_sel(factor);
        self.emit_state();
    }

    pub fn set_stepper_acc_range(&mut self, factor: u8) {
        self.stepper.set_acc_range_sel(factor);
        self.emit_state();
    }

    pub fn start_coarse_seek(&mut self) {
        self.stepper.set_enabled(true);
        self.stepper.clear_fast_stop();
        self.stepper.request_speed_mode();
        self.stepper.set_acceleration(Self::STEP_ACCELERATION);
        self.stepper
            .set_velocity_register(Self::COARSE_SEEK_REGISTER);
        self.emit_state();
    }

    pub fn stop_by_zero_velocity(&mut self) {
        self.stepper.clear_fast_stop();
        self.stepper.request_speed_mode();
        self.stepper.set_velocity_register(0);
        self.emit_state();
    }

    pub fn stop_by_stop2n(&mut self) {
        self.stepper.request_fast_stop();
        self.emit_state();
    }

    pub fn release_stop2n(&mut self) {
        self.stepper.clear_fast_stop();
        self.stepper.request_speed_mode();
        self.emit_state();
    }
}
