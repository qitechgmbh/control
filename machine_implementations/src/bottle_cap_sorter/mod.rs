pub mod act;
pub mod api;
pub mod conveyer_belt_controller;
pub mod new;
pub mod valve_controller;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct ColorBoundsState {
    pub h_min: f32,
    pub h_max: f32,
    pub s_min: f32,
    pub s_max: f32,
    pub l_min: f32,
    pub l_max: f32,
    pub valve_index: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ColorHsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ColorBoundsHsl {
    pub h_min: f32,
    pub h_max: f32,
    pub s_min: f32,
    pub s_max: f32,
    pub l_min: f32,
    pub l_max: f32,
}

impl ColorBoundsHsl {
    pub fn new(
        h_min: f32,
        h_max: f32,
        s_min: f32,
        s_max: f32,
        l_min: f32,
        l_max: f32,
    ) -> Result<Self,anyhow::Error> {
        // Check hue bounds (0-360 range, allowing wrap-around)
        if h_min < 0.0 || h_min > 360.0 || h_max < 0.0 || h_max > 360.0 {
            return Err(anyhow::anyhow!("Hue values must be between 0.0 and 360.0"));
        }

        // Check saturation bounds (0-1 range)
        if s_min < 0.0 || s_min > 1.0 || s_max < 0.0 || s_max > 1.0 || s_min > s_max {
            return Err(anyhow::anyhow!(
                "Saturation values must be between 0.0 and 1.0, with s_min <= s_max"
            ));
        }

        // Check lightness bounds (0-1 range)
        if l_min < 0.0 || l_min > 1.0 || l_max < 0.0 || l_max > 1.0 || l_min > l_max {
            return Err(anyhow::anyhow!(
                "Lightness values must be between 0.0 and 1.0, with l_min <= l_max"
            ));
        }

        Ok(ColorBoundsHsl {
            h_min,
            h_max,
            s_min,
            s_max,
            l_min,
            l_max,
        })
    }

    pub fn contains(&self, color: &ColorHsl) -> bool {
        let h_in_range = if self.h_min <= self.h_max {
            // Normal range: h_min <= h <= h_max
            color.h >= self.h_min && color.h <= self.h_max
        } else {
            // Wrap around case: h >= h_min OR h <= h_max
            color.h >= self.h_min || color.h <= self.h_max
        };

        let s_in_range = color.s >= self.s_min && color.s <= self.s_max;
        let l_in_range = color.l >= self.l_min && color.l <= self.l_max;

        h_in_range && s_in_range && l_in_range
    }

    pub fn overlaps(&self, other: &ColorBoundsHsl) -> bool {
        let h_overlap = self.hue_ranges_overlap(other);
        let s_overlap = !(self.s_max < other.s_min || other.s_max < self.s_min);
        let l_overlap = !(self.l_max < other.l_min || other.l_max < self.l_min);

        h_overlap && s_overlap && l_overlap
    }

    fn hue_ranges_overlap(&self, other: &ColorBoundsHsl) -> bool {
        let self_wraps = self.h_min > self.h_max;
        let other_wraps = other.h_min > other.h_max;

        match (self_wraps, other_wraps) {
            (false, false) => {
                // Neither range wraps - standard overlap check
                !(self.h_max < other.h_min || other.h_max < self.h_min)
            }
            (true, true) => {
                // Both ranges wrap - they always overlap
                true
            }
            (true, false) => {
                // Self wraps, other doesn't
                // Overlap if other intersects with either [h_min, 360] or [0, h_max]
                other.h_max >= self.h_min || other.h_min <= self.h_max
            }
            (false, true) => {
                // Other wraps, self doesn't
                // Overlap if self intersects with either [other.h_min, 360] or [0, other.h_max]
                self.h_max >= other.h_min || self.h_min <= other.h_max
            }
        }
    }
}


use api::{ConveyorBeltState, LiveValuesEvent, ModeState, Sorter1Events, Sorter1Namespace, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use conveyer_belt_controller::ConveyorBeltController;
use qitech_lib::{ethercat_hal::io::{digital_output::DigitalOutputDevice, stepper_velocity_el70x1::StepperVelocityEL70x1Device}, machines::{MachineIdentification, MachineIdentificationUnique}, units::{Length, Velocity, length::centimeter, time::second, velocity::meter_per_second}};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};
use std::{
    cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc, time::Instant
};
use valve_controller::ValveController;

use crate::{MACHINE_SORTER_V1, MachineMessage, QiTechMachine, VENDOR_QITECH};
// Valve positions from detection point in centimeters
// halved due to motor having different translation
// index 0 was 26.0 for example
const VALVE_DISTANCES_CM: [f64; 8] = [13.0, 20.5, 27.5, 35.0, 42.5, 50.0, 57.5, 65.0];

#[derive(Debug, Clone)]
pub struct ScheduledEjection {
    pub id: u32,
    pub crossing_time: Instant,
    pub valve_index: usize,
    pub trigger_time: Instant,
    pub color: ColorRgb,
}

impl ScheduledEjection {
    /// Create a new scheduled ejection with calculated trigger time
    pub fn new(
        id: u32,
        crossing_time: Instant,
        valve_index: usize,
        conveyor_speed: Velocity,
        color: ColorRgb,
    ) -> Self {
        let valve_distance = Length::new::<centimeter>(VALVE_DISTANCES_CM[valve_index]);
        let travel_time = valve_distance / conveyor_speed;
        tracing::info!("{}", travel_time.get::<second>());
        let trigger_time =
            crossing_time + std::time::Duration::from_secs_f64(travel_time.get::<second>());

        Self {
            id,
            crossing_time,
            valve_index,
            trigger_time,
            color,
        }
    }

    /// Check if the ejection is ready to be triggered
    pub fn is_ready_to_trigger(&self, current_time: Instant) -> bool {
        current_time >= self.trigger_time
    }

    /// Get time until trigger in milliseconds (negative if overdue)
    pub fn time_until_trigger_ms(&self, current_time: Instant) -> i64 {
        let duration = self.trigger_time.duration_since(current_time);
        duration.as_millis() as i64
    }

    /// Get elapsed time since crossing in milliseconds
    pub fn elapsed_since_crossing_ms(&self, current_time: Instant) -> u128 {
        current_time.duration_since(self.crossing_time).as_millis()
    }
}

#[derive(Debug)]
pub struct Sorter1 {
	api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    // drivers
    pub conveyor_belt: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    
    pub air_valve_outputs: Rc<RefCell<dyn DigitalOutputDevice>>,
    pub air_valve_states: [bool; 8],

    pub conveyor_belt_controller: ConveyorBeltController,
    pub valve_controllers: [ValveController; 8],
    
    namespace: Sorter1Namespace,
    last_measurement_emit: Instant,
    
    pub machine_identification_unique: MachineIdentificationUnique,
    pub mode: Sorter1Mode,
    pub conveyor_belt_mode: ConveyorBeltMode,
    emitted_default_state: bool,
    pub scheduled_ejections: HashMap<u32, ScheduledEjection>,
}

impl Sorter1 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_SORTER_V1,
    };
}

/// Implement Core Functionality
impl Sorter1 {
	pub fn get_live_values(&mut self) -> LiveValuesEvent {
        // Get current conveyor belt speed in m/s
        let conveyor_belt_speed = self
            .conveyor_belt_controller
            .get_current_speed()
            .get::<meter_per_second>();


        LiveValuesEvent {
            conveyor_belt_speed,
            air_valve_states: self.air_valve_states,
        }
	}

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(Sorter1Events::LiveValues(event));
    }

    pub fn get_state(&mut self) -> StateEvent {
    	StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            mode_state: ModeState {
                mode: self.mode.clone().into(),
            },
            conveyor_belt_state: ConveyorBeltState {
                enabled: self.conveyor_belt_controller.is_enabled(),
                target_speed: self
                    .conveyor_belt_controller
                    .get_target_speed()
                    .get::<meter_per_second>(),
            },
            colors_state: vec![],
        }
    }

    pub fn emit_state(&mut self) {
    	let state = self.get_state();
        let event = state.build();
        self.namespace.emit(Sorter1Events::State(event));
    }
}

/// Implement Mode Management
impl Sorter1 {
    fn set_mode(&mut self, mode: &Sorter1Mode) {
        // Apply the mode changes
        self.mode = mode.clone();
        self.set_conveyor_belt_mode(mode);
        self.emit_state();
    }

    /// Apply the mode changes to the conveyor belt
    fn set_conveyor_belt_mode(&mut self, mode: &Sorter1Mode) {
        let mode: ConveyorBeltMode = mode.clone().into();

        // Transition matrix
        match self.conveyor_belt_mode {
            ConveyorBeltMode::Standby => match mode {
                ConveyorBeltMode::Standby => {}
                ConveyorBeltMode::Running => {
                    // Enable conveyor belt
                    let mut conveyor_belt = self.conveyor_belt.borrow_mut();
                    conveyor_belt.set_enabled(0,true);
                    drop(conveyor_belt);
                    self.conveyor_belt_controller.set_enabled(true);
                }
            },
            ConveyorBeltMode::Running => match mode {
                ConveyorBeltMode::Standby => {
                    // Disable conveyor belt
                    let mut conveyor_belt = self.conveyor_belt.borrow_mut();
                    conveyor_belt.set_enabled(0,false);
                    drop(conveyor_belt);
                    self.conveyor_belt_controller.set_enabled(false);
                }
                ConveyorBeltMode::Running => {}
            },
        }

        // Update the internal state
        self.conveyor_belt_mode = mode;
    }
}

fn safe_divide(divisor: f32, dividend: f32) -> f32 {
    if divisor.abs() == f32::EPSILON {
        return 0.0;
    }

    return dividend / divisor;
}

/// Implement Color Range control
impl Sorter1 {
    pub fn assign_colour_bounds_to_valve(&mut self, colour_bounds: ColorBoundsState) {
        if colour_bounds.valve_index >= 8 {
            tracing::info!(
                "assign_colour_bounds_to_valve: valve_index was out of range: {}",
                colour_bounds.valve_index
            );
            return;
        }
/*
        self.colors[colour_bounds.valve_index] = ColorBoundsHsl {
            h_min: colour_bounds.h_min,
            h_max: colour_bounds.h_max,
            s_min: safe_divide(100.0, colour_bounds.s_min),
            s_max: safe_divide(100.0, colour_bounds.s_max),
            l_min: safe_divide(100.0, colour_bounds.l_min),
            l_max: safe_divide(100.0, colour_bounds.l_max),
        };
        tracing::info!("colour bound{:?}", self.colors[colour_bounds.valve_index]);*/
        self.emit_state();
    }
}

/// Implement Conveyor Belt Control
impl Sorter1 {
    /// Set target speed in m/s
    pub fn conveyor_belt_set_target_speed(&mut self, target_speed: f64) {
        let target_speed = Velocity::new::<meter_per_second>(target_speed);
        self.conveyor_belt_controller.set_target_speed(target_speed);
        self.emit_state();
    }

    /// called by `act`
    pub fn sync_conveyor_belt_speed(&mut self, t: Instant) {
        let speed = self.conveyor_belt_controller.calc_speed(t);
        // Invert speed ebcaus emotor is mounted in reverse
        let speed = -speed * 2.0;
        let mut conveyer_belt = self.conveyor_belt.borrow_mut();
        let _ = conveyer_belt.set_speed(0,speed);
        drop(conveyer_belt);
    }
}

/// Implement Air Valve Control
impl Sorter1 {
    /// Set air valve state
    pub fn set_air_valve(&mut self, valve_index: usize, state: bool) {
        if valve_index < 8 {
        	let mut air_valv_outputs = self.air_valve_outputs.borrow_mut();
            air_valv_outputs.set_output(valve_index,state);
            drop(air_valv_outputs);
        }
        self.emit_state();
    }

    /// Set all air valves
    pub fn set_all_air_valves(&mut self, states: &[bool; 8]) {
    	let mut air_valv_outputs = self.air_valve_outputs.borrow_mut();
        for (i, &state) in states.iter().enumerate() {
            air_valv_outputs.set_output(i,state);
        }
        drop(air_valv_outputs);
        self.emit_state();
    }

    /// Activate air valve for a specific duration (in milliseconds)
    pub fn activate_air_valve_pulse(&mut self, valve_index: usize, duration_ms: u64) {
        if valve_index < 8 {
            // Activate the valve controller in pulse mode
            self.valve_controllers[valve_index].activate_pulse(duration_ms);
        	let mut air_valv_outputs = self.air_valve_outputs.borrow_mut();
            air_valv_outputs.set_output(valve_index,true);
        }
        self.emit_state();
    }
}

/// Implement Bottle Cap Scheduling
impl Sorter1 {
    fn color_to_index(&mut self, color: &ColorHsl) -> Option<usize> {
        tracing::info!(
            "[color_to_index] Checking color HSL(h={:.1}°, s={:.3}, l={:.3})",
            color.h,
            color.s,
            color.l
        );

        return None;
    }

    /// Schedule a bottle cap for processing
    /// This is called when a bottle cap crosses the middle line in the camera detection system
    pub fn schedule_bottle_cap(&mut self, id: u32, crossing_time: Instant, color: ColorRgb) {
        // convert to HSL for color-based sorting
        let color_hsl: ColorHsl = ColorHsl { h: 0.0, s: 0.0, l: 0.0 };

        let valve_index = 0;

        // Get current conveyor belt speed for calculation
        let belt_speed = self.conveyor_belt_controller.get_current_speed();

        // Create ejection with physics-based trigger time calculation
        let ejection = ScheduledEjection::new(id, crossing_time, valve_index, belt_speed, color);

        let valve_distance = VALVE_DISTANCES_CM[valve_index];
        let travel_time_ms =
            ((valve_distance / 100.0) / belt_speed.get::<meter_per_second>()) * 1000.0;

        tracing::info!(
            "[{}] Scheduling bottle cap ID {} at valve {} in {}ms. hue={:.1}°, saturation={:.3}, lightness={:.3}",
            module_path!(),
            id,
            valve_index,
            travel_time_ms,
            color_hsl.h,
            color_hsl.s,
            color_hsl.l
        );

        self.scheduled_ejections.insert(id, ejection);

        tracing::debug!(
            "[Sorter1::schedule_bottle_cap] Total scheduled ejections: {} (IDs: {:?})",
            self.scheduled_ejections.len(),
            self.scheduled_ejections.keys().collect::<Vec<_>>()
        );
    }

    /// Get valve distance in centimeters for a given valve index
    pub fn get_valve_distance_cm(valve_index: usize) -> f64 {
        if valve_index < VALVE_DISTANCES_CM.len() {
            VALVE_DISTANCES_CM[valve_index]
        } else {
            0.0 // Default fallback
        }
    }

    /// Get all scheduled ejections
    pub fn get_scheduled_ejections(&self) -> &HashMap<u32, ScheduledEjection> {
        &self.scheduled_ejections
    }

    /// Remove a scheduled ejection (e.g., after processing)
    pub fn remove_scheduled_ejection(&mut self, id: u32) -> Option<ScheduledEjection> {
        let removed = self.scheduled_ejections.remove(&id);
        if let Some(ref ejection) = removed {
            tracing::debug!(
                "[Sorter1::remove_scheduled_ejection] Removed ejection ID {} scheduled at {:?} for valve {}",
                id,
                ejection.crossing_time,
                ejection.valve_index
            );
        }
        removed
    }

    /// Clear all scheduled ejections
    pub fn clear_scheduled_ejections(&mut self) {
        let count = self.scheduled_ejections.len();
        self.scheduled_ejections.clear();
        tracing::debug!(
            "[Sorter1::clear_scheduled_ejections] Cleared {} scheduled ejections",
            count
        );
    }

    /// Trigger valves based on scheduled ejections
    /// Physics-based logic: triggers valve when bottle cap reaches calculated position
    pub fn trigger_valves(&mut self, current_time: Instant) {
        if !self.scheduled_ejections.is_empty() {
            tracing::debug!(
                "[Sorter1::trigger_valves] Checking {} scheduled ejections for triggering",
                self.scheduled_ejections.len()
            );
        }

        // Collect entries that need processing to avoid borrow checker issues
        let entries_to_process: Vec<ScheduledEjection> = self
            .scheduled_ejections
            .values()
            .filter_map(|ejection| {
                if ejection.is_ready_to_trigger(current_time) {
                    Some(ejection.clone())
                } else {
                    let time_until_trigger = ejection.time_until_trigger_ms(current_time);
                    tracing::debug!(
                        "[Sorter1::trigger_valves] Ejection ID {} waiting {}ms for valve {} (distance: {}cm)",
                        ejection.id,
                        time_until_trigger,
                        ejection.valve_index,
                        VALVE_DISTANCES_CM[ejection.valve_index]
                    );
                    None
                }
            })
            .collect();

        // Process each entry
        for ejection in entries_to_process {
            let elapsed_since_crossing = ejection.elapsed_since_crossing_ms(current_time);
            let valve_distance = VALVE_DISTANCES_CM[ejection.valve_index];

            tracing::warn!(
                "[Sorter1::trigger_valves] 🔴 VALVE TRIGGER! Activating valve {} for bottle cap ID {} at {}cm ({}ms after crossing, color: RGB({}, {}, {}))",
                ejection.valve_index,
                ejection.id,
                valve_distance,
                elapsed_since_crossing,
                ejection.color.r,
                ejection.color.g,
                ejection.color.b
            );

            // Trigger the specified valve for 50ms
            self.activate_air_valve_pulse(ejection.valve_index, 20);

            // Remove the processed ejection
            self.remove_scheduled_ejection(ejection.id);

            tracing::debug!(
                "[Sorter1::trigger_valves] Valve {} triggered and ejection ID {} removed from schedule",
                ejection.valve_index,
                ejection.id
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sorter1Mode {
    Standby,
    Running,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConveyorBeltMode {
    Standby,
    Running,
}

impl From<Sorter1Mode> for ConveyorBeltMode {
    fn from(mode: Sorter1Mode) -> Self {
        match mode {
            Sorter1Mode::Standby => ConveyorBeltMode::Standby,
            Sorter1Mode::Running => ConveyorBeltMode::Running,
        }
    }
}

impl QiTechMachine for Sorter1{}