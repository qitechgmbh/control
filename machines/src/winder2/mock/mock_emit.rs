use std::time::Instant;

use super::Winder2;
use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::winder2::Winder2Mode;
use crate::winder2::api::LiveValuesEvent;
use crate::winder2::api::{ModeState, SpoolAutomaticActionMode, StateEvent, Winder2Events};
use crate::winder2::puller_speed_controller::{GearRatio, PullerRegulationMode};
use crate::winder2::spool_speed_controller::SpoolSpeedControllerType;
use crate::{MACHINE_WINDER_V1, VENDOR_QITECH};
use control_core::socketio::event::BuildEvent;
use control_core::socketio::namespace::NamespaceCacheingLogic;

impl Winder2 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_WINDER_V1,
    };

    pub fn stop_or_pull_spool_reset(&mut self, _now: Instant) {}
    /// Implement Mode
    pub fn set_mode(&mut self, mode: &Winder2Mode) {
        self.mode_state = ModeState {
            mode: mode.clone().into(),
            can_wind: true,
        };
        self.emit_state();
    }

    /// Implement Traverse
    pub fn set_laser(&mut self, value: bool) {
        self.traverse_state.laserpointer = value;
        self.emit_state();
    }

    pub fn traverse_set_limit_inner(&mut self, limit: f64) {
        self.traverse_state.limit_inner = limit;
        self.emit_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        self.traverse_state.limit_outer = limit;
        self.emit_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        self.traverse_state.step_size = step_size;
        self.emit_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
        self.traverse_state.padding = padding;
        self.emit_state();
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        self.traverse_state.position_in = self.traverse_state.limit_inner;
        self.emit_state();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        self.traverse_state.position_in = self.traverse_state.limit_outer;
        self.emit_state();
    }

    pub fn traverse_goto_home(&mut self) {
        self.traverse_state.position_in = 0.0;
        self.emit_state();
    }

    pub fn emit_live_values(&mut self) {
        let event = LiveValuesEvent {
            traverse_position: Some(0.0),
            puller_speed: 0.0,
            spool_rpm: 0.0,
            tension_arm_angle: 0.0,
            spool_progress: 0.0,
        };

        let event = event.build();

        self.namespace.emit(Winder2Events::LiveValues(event));
    }

    pub fn build_state_event(&mut self) -> StateEvent {
        use crate::MachineCrossConnectionState;

        let connected_machine = self.connected_machines.get(0);

        let ident = match connected_machine {
            Some(machine) => Some(machine.ident.clone()),
            None => None,
        };

        let cross_conn = MachineCrossConnectionState {
            machine_identification_unique: ident,
            is_available: connected_machine.is_some(),
        };

        StateEvent {
            is_default_state: self.is_default_state,
            traverse_state: self.traverse_state.clone(),
            puller_state: self.puller_state.clone(),
            spool_automatic_action_state: self.spool_automatic_action_state.clone(),
            mode_state: self.mode_state.clone(),
            tension_arm_state: self.tension_arm_state.clone(),
            spool_speed_controller_state: self.spool_speed_controller_state.clone(),
            connected_machine_state: cross_conn,
        }
    }

    pub fn emit_state(&mut self) {
        let state = self.build_state_event();
        let event = state.build();
        self.namespace.emit(Winder2Events::State(event));
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, _mode: &Winder2Mode) {
        self.mode_state = ModeState {
            mode: crate::winder2::api::Mode::Standby,
            can_wind: true,
        };
        self.emit_state();
    }

    /// Implement Tension Arm
    pub fn tension_arm_zero(&mut self) {
        self.tension_arm_state.zeroed = true;
        self.emit_state();
    }

    pub fn set_spool_automatic_required_meters(&mut self, meters: f64) {
        self.spool_automatic_action_state.spool_required_meters = meters;
        self.emit_state();
    }

    pub fn set_spool_automatic_mode(&mut self, mode: SpoolAutomaticActionMode) {
        self.spool_automatic_action_state
            .spool_automatic_action_mode = mode;
        self.emit_state();
    }

    pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
        self.puller_state.regulation = puller_regulation_mode;
        self.emit_state();
    }

    /// Set target speed in m/min
    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        self.puller_state.target_speed = target_speed;
        self.emit_state();
    }

    /// Set target diameter in mm
    pub fn puller_set_target_diameter(&mut self, target_diameter: f64) {
        self.puller_state.target_diameter = target_diameter;
        self.emit_state();
    }

    /// Set forward direction
    pub fn puller_set_forward(&mut self, forward: bool) {
        self.puller_state.forward = forward;
        self.emit_state();
    }

    /// Set gear ratio for winding speed
    pub fn puller_set_gear_ratio(&mut self, gear_ratio: GearRatio) {
        self.puller_state.gear_ratio = gear_ratio;
        self.emit_state();
    }

    // Spool Speed Controller API methods
    pub fn spool_set_regulation_mode(&mut self, regulation_mode: SpoolSpeedControllerType) {
        self.spool_speed_controller_state.regulation_mode = regulation_mode;
        self.emit_state();
    }

    /// Set minimum speed for minmax mode in RPM
    pub fn spool_set_minmax_min_speed(&mut self, min_speed_rpm: f64) {
        self.spool_speed_controller_state.minmax_min_speed = min_speed_rpm;
        self.emit_state();
    }

    /// Set maximum speed for minmax mode in RPM
    pub fn spool_set_minmax_max_speed(&mut self, max_speed_rpm: f64) {
        self.spool_speed_controller_state.minmax_max_speed = max_speed_rpm;
        self.emit_state();
    }

    /// Set tension target for adaptive mode (0.0-1.0)
    pub fn spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.spool_speed_controller_state.adaptive_tension_target = tension_target;
        self.emit_state();
    }

    /// Set radius learning rate for adaptive mode
    pub fn spool_set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.spool_speed_controller_state
            .adaptive_radius_learning_rate = radius_learning_rate;
        self.emit_state();
    }

    /// Set max speed multiplier for adaptive mode
    pub fn spool_set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.spool_speed_controller_state
            .adaptive_max_speed_multiplier = max_speed_multiplier;
        self.emit_state();
    }

    /// Set acceleration factor for adaptive mode
    pub fn spool_set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.spool_speed_controller_state
            .adaptive_acceleration_factor = acceleration_factor;
        self.emit_state();
    }

    /// Set deacceleration urgency multiplier for adaptive mode
    pub fn spool_set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.spool_speed_controller_state
            .adaptive_deacceleration_urgency_multiplier = deacceleration_urgency_multiplier;
        self.emit_state();
    }

    /// Set forward rotation direction
    pub fn spool_set_forward(&mut self, forward: bool) {
        self.spool_speed_controller_state.forward = forward;
        self.emit_state();
    }

    /// implement machine connection
    /// set connected buffer
    pub fn set_connected_buffer(
        &mut self,
        _machine_identification_unique: MachineIdentificationUnique,
    ) {
        self.emit_state();
    }

    /// disconnect buffer
    pub fn disconnect_buffer(
        &mut self,
        _machine_identification_unique: MachineIdentificationUnique,
    ) {
        self.emit_state();
    }
}
