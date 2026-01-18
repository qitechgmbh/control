use crate::bottle_sorter::api::{BottleSorterEvents, LiveValuesEvent, StateEvent};
use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use smol::channel::{Receiver, Sender};
use std::time::Instant;

pub mod act;
pub mod api;
pub mod new;

use crate::bottle_sorter::api::BottleSorterNamespace;
use crate::{BOTTLE_SORTER, VENDOR_QITECH};

#[derive(Debug)]
pub struct BottleSorter {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: BottleSorterNamespace,
    pub last_state_emit: Instant,
    pub last_live_values_emit: Instant,
    
    // State
    pub outputs: [bool; 8],
    pub stepper_speed_mm_s: f64,
    pub stepper_enabled: bool,
    
    // Hardware
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub douts: [DigitalOutput; 8],
    pub stepper: StepperVelocityEL70x1,
    
    // Config
    pub steps_per_mm: f64,
    
    // Pulse control
    pub pulse_outputs: [Option<Instant>; 8],
    pub pulse_duration_ms: u64,
}

impl Machine for BottleSorter {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl BottleSorter {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: BOTTLE_SORTER,
    };
}

impl BottleSorter {
    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            outputs: self.outputs,
            stepper_speed_mm_s: self.stepper_speed_mm_s,
            stepper_enabled: self.stepper_enabled,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(BottleSorterEvents::State(event));
    }

    pub fn get_live_values(&self) -> LiveValuesEvent {
        LiveValuesEvent {
            stepper_position: self.stepper.get_position() as f64 / self.steps_per_mm,
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace
            .emit(BottleSorterEvents::LiveValues(event));
    }

    /// Set the stepper speed in mm/s
    pub fn set_stepper_speed(&mut self, speed_mm_s: f64) {
        self.stepper_speed_mm_s = speed_mm_s;
        let steps_per_s = speed_mm_s * self.steps_per_mm;
        let _ = self.stepper.set_speed(steps_per_s);
        self.emit_state();
    }

    /// Set the stepper enabled state
    pub fn set_stepper_enabled(&mut self, enabled: bool) {
        self.stepper_enabled = enabled;
        self.stepper.set_enabled(enabled);
        self.emit_state();
    }

    /// Pulse a specific output (turn on for a few milliseconds)
    pub fn pulse_output(&mut self, index: usize) {
        if index < self.outputs.len() {
            self.outputs[index] = true;
            self.douts[index].set(true);
            self.pulse_outputs[index] = Some(Instant::now());
            self.emit_state();
        }
    }

    /// Check and turn off pulsed outputs
    pub fn update_pulse_outputs(&mut self) {
        let now = Instant::now();
        for i in 0..8 {
            if let Some(start_time) = self.pulse_outputs[i] {
                if now.duration_since(start_time).as_millis() >= self.pulse_duration_ms as u128 {
                    self.outputs[i] = false;
                    self.douts[i].set(false);
                    self.pulse_outputs[i] = None;
                }
            }
        }
    }
}
