use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::minimal_bottle_sorter::api::{LiveValuesEvent, MinimalBottleSorterEvents, StateEvent};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use smol::channel::{Receiver, Sender};
use std::time::Instant;
pub mod act;
pub mod api;
pub mod new;
use crate::minimal_bottle_sorter::api::MinimalBottleSorterNamespace;
use crate::{MINIMAL_BOTTLE_SORTER, VENDOR_QITECH};

#[derive(Debug)]
pub struct MinimalBottleSorter {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: MinimalBottleSorterNamespace,
    pub last_state_emit: Instant,
    pub last_live_values_emit: Instant,
    pub stepper_enabled: bool,
    pub stepper_speed: f64,      // mm/s
    pub stepper_direction: bool, // true = forward, false = backward
    pub outputs: [bool; 8],
    pub pulse_remaining: [u32; 8], // remaining milliseconds for pulse
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub stepper: StepperVelocityEL70x1,
    pub douts: [DigitalOutput; 8],
}

impl Machine for MinimalBottleSorter {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl MinimalBottleSorter {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MINIMAL_BOTTLE_SORTER,
    };
}

impl MinimalBottleSorter {
    pub fn emit_state(&mut self) {
        let event = StateEvent {
            stepper_enabled: self.stepper_enabled,
            stepper_speed: self.stepper_speed,
            stepper_direction: self.stepper_direction,
            outputs: self.outputs,
        }
        .build();

        self.namespace.emit(MinimalBottleSorterEvents::State(event));
    }

    pub fn emit_live_values(&mut self) {
        let event = LiveValuesEvent {
            stepper_actual_speed: self.stepper.get_speed() as f64,
            stepper_position: self.stepper.get_position(),
        }
        .build();

        self.namespace
            .emit(MinimalBottleSorterEvents::LiveValues(event));
    }

    /// Set stepper speed in mm/s
    pub fn set_stepper_speed(&mut self, speed_mm_s: f64) {
        self.stepper_speed = speed_mm_s.abs().max(0.0).min(100.0); // Limit to 0-100 mm/s
        self.update_stepper();
        self.emit_state();
    }

    /// Set stepper direction
    pub fn set_stepper_direction(&mut self, forward: bool) {
        self.stepper_direction = forward;
        self.update_stepper();
        self.emit_state();
    }

    /// Enable/disable stepper motor
    pub fn set_stepper_enabled(&mut self, enabled: bool) {
        self.stepper_enabled = enabled;
        self.stepper.set_enabled(enabled);
        self.emit_state();
    }

    /// Update stepper motor speed based on current settings
    fn update_stepper(&mut self) {
        if self.stepper_enabled {
            // Convert mm/s to steps/s (assuming 200 steps per revolution and some mechanical conversion)
            // This is a placeholder - adjust based on your actual mechanical setup
            let steps_per_mm = 100.0; // Adjust this value based on your setup
            let speed_steps_s = self.stepper_speed * steps_per_mm;
            let signed_speed = if self.stepper_direction {
                speed_steps_s
            } else {
                -speed_steps_s
            };
            let _ = self.stepper.set_speed(signed_speed);
        }
    }

    /// Trigger a pulse on a digital output (turn on briefly then off)
    pub fn pulse_output(&mut self, index: usize, duration_ms: u32) {
        if index < 8 {
            self.outputs[index] = true;
            self.douts[index].set(true);
            self.pulse_remaining[index] = duration_ms;
            self.emit_state();
        }
    }

    /// Update pulse timers and turn off outputs when time expires
    pub fn update_pulses(&mut self, delta_ms: u32) {
        for i in 0..8 {
            if self.pulse_remaining[i] > 0 {
                if self.pulse_remaining[i] <= delta_ms {
                    self.pulse_remaining[i] = 0;
                    self.outputs[i] = false;
                    self.douts[i].set(false);
                } else {
                    self.pulse_remaining[i] -= delta_ms;
                }
            }
        }
    }
}
