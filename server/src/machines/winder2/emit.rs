use crate::machines::buffer1::BufferV1;
use crate::machines::winder2::api::{
    ConnectedMachineState, LiveValuesEvent, ModeState, PullerState, SpoolAutomaticActionMode,
    SpoolAutomaticActionState, SpoolSpeedControllerState, StateEvent, TensionArmState,
    TraverseState, Winder2Events,
};
use crate::machines::winder2::puller_speed_controller::PullerRegulationMode;
use crate::machines::winder2::{Winder2, Winder2Mode, spool_speed_controller};
use control_core::machines::identification::MachineIdentificationUnique;
use control_core::machines::{ConnectedMachine, ConnectedMachineData, downcast_machine};
use control_core::socketio::event::BuildEvent;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use control_core::uom_extensions::velocity::meter_per_minute;
use smol::block_on;
use smol::lock::Mutex;
use std::sync::Arc;
use uom::si::angle::degree;
use uom::si::angular_velocity::revolution_per_minute;
use uom::si::f64::{Length, Velocity};
use uom::si::length::{meter, millimeter};

impl Winder2 {
    // --- Traverse functions ---
    pub fn traverse_set_limit_inner(&mut self, limit: f64) {
        let new_inner = Length::new::<meter>(limit);
        let current_outer = self.traverse_controller.get_limit_outer();
        if !Self::validate_traverse_limits(new_inner, current_outer) {
            return;
        }
        self.traverse_controller.set_limit_inner(new_inner);
        self.emit_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        let new_outer = Length::new::<meter>(limit);
        let current_inner = self.traverse_controller.get_limit_inner();
        if !Self::validate_traverse_limits(current_inner, new_outer) {
            return;
        }
        self.traverse_controller.set_limit_outer(new_outer);
        self.emit_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        self.traverse_controller
            .set_step_size(Length::new::<meter>(step_size));
        self.emit_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
        self.traverse_controller
            .set_padding(Length::new::<meter>(padding));
        self.emit_state();
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        if self.can_go_in() {
            self.traverse_controller.goto_limit_inner();
        }
        self.emit_state();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        if self.can_go_out() {
            self.traverse_controller.goto_limit_outer();
        }
        self.emit_state();
    }

    pub fn traverse_goto_home(&mut self) {
        if self.can_go_home() {
            self.traverse_controller.goto_home();
        }
        self.emit_state();
    }

    // --- Mode / Puller / Spool functions ---
    pub fn set_mode(&mut self, mode: &Winder2Mode) {
        let should_update = *mode != Winder2Mode::Wind || self.can_wind();
        if should_update {
            self.mode = mode.clone();
            self.set_spool_mode(mode);
            self.set_puller_mode(mode);
            self.set_traverse_mode(mode);
        }
        self.emit_state();
    }

    pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
        self.puller_speed_controller
            .set_regulation_mode(puller_regulation_mode);
        self.emit_state();
    }

    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        let target_speed = Velocity::new::<uom::si::velocity::meter_per_second>(target_speed);
        self.puller_speed_controller.set_target_speed(target_speed);
        self.emit_state();
    }

    pub fn puller_set_target_diameter(&mut self, target_diameter: f64) {
        let target_diameter = Length::new::<meter>(target_diameter);
        self.puller_speed_controller
            .set_target_diameter(target_diameter);
        self.emit_state();
    }

    pub fn puller_set_forward(&mut self, forward: bool) {
        self.puller_speed_controller.set_forward(forward);
        self.emit_state();
    }

    pub fn spool_set_regulation_mode(
        &mut self,
        regulation_mode: spool_speed_controller::SpoolSpeedControllerType,
    ) {
        self.spool_speed_controller.set_type(regulation_mode);
        self.emit_state();
    }

    pub fn spool_set_minmax_min_speed(&mut self, min_speed_rpm: f64) {
        let min_speed = uom::si::f64::AngularVelocity::new::<
            uom::si::angular_velocity::revolution_per_minute,
        >(min_speed_rpm);
        if let Err(e) = self.spool_speed_controller.set_minmax_min_speed(min_speed) {
            tracing::error!("Failed to set spool min speed: {:?}", e);
        }
        self.emit_state();
    }

    pub fn spool_set_minmax_max_speed(&mut self, max_speed_rpm: f64) {
        let max_speed = uom::si::f64::AngularVelocity::new::<
            uom::si::angular_velocity::revolution_per_minute,
        >(max_speed_rpm);
        if let Err(e) = self.spool_speed_controller.set_minmax_max_speed(max_speed) {
            tracing::error!("Failed to set spool max speed: {:?}", e);
        }
        self.emit_state();
    }

    pub fn spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.spool_speed_controller
            .set_adaptive_tension_target(tension_target);
        self.emit_state();
    }

    pub fn spool_set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.spool_speed_controller
            .set_adaptive_radius_learning_rate(radius_learning_rate);
        self.emit_state();
    }

    pub fn spool_set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.spool_speed_controller
            .set_adaptive_max_speed_multiplier(max_speed_multiplier);
        self.emit_state();
    }

    pub fn spool_set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.spool_speed_controller
            .set_adaptive_acceleration_factor(acceleration_factor);
        self.emit_state();
    }

    pub fn spool_set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.spool_speed_controller
            .set_adaptive_deacceleration_urgency_multiplier(deacceleration_urgency_multiplier);
        self.emit_state();
    }

    pub fn set_spool_automatic_required_meters(&mut self, meters: f64) {
        self.spool_automatic_action.target_length = Length::new::<meter>(meters);
        self.emit_state();
    }

    pub fn set_spool_automatic_mode(&mut self, mode: SpoolAutomaticActionMode) {
        self.spool_automatic_action.mode = mode;
        self.emit_state();
    }

    pub fn set_connected_buffer(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            BufferV1::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => return,
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let buffer_weak = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let buffer_weak = match buffer_weak {
            Some(buffer_weak) => buffer_weak,
            None => return,
        };
        let buffer_strong = match buffer_weak.upgrade() {
            Some(buffer_strong) => buffer_strong,
            None => return,
        };

        let buffer: Arc<Mutex<BufferV1>> = block_on(downcast_machine::<BufferV1>(buffer_strong))
            .expect("failed downcasting machine");

        let machine = Arc::downgrade(&buffer);

        self.connected_buffer = Some(ConnectedMachine {
            machine_identification_unique,
            machine,
        });
        self.emit_state();

        self.reverse_connect();
    }

    pub fn disconnect_buffer(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            BufferV1::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        if let Some(connected) = &self.connected_buffer {
            if let Some(buffer_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut buffer = buffer_arc.lock().await;
                    if buffer.connected_winder.is_some() {
                        buffer.connected_winder = None;
                        buffer.emit_state();
                    }
                };
                smol::spawn(future).detach();
            }
        }
        self.connected_buffer = None;

        self.emit_state();
    }

    pub fn tension_arm_zero(&mut self) {
        self.tension_arm.zero();
        self.emit_live_values();
        self.emit_state();
    }

    pub fn emit_live_values(&mut self) {
        let angle_deg = self.tension_arm.get_angle().get::<degree>();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let angle_deg = if angle_deg >= 270.0 {
            angle_deg - 360.0
        } else {
            angle_deg
        };

        // Calculate puller speed from current motor steps
        let steps_per_second = self.puller.get_speed();
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(steps_per_second as f64);
        let puller_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);

        // Calculate spool RPM from current motor steps
        let spool_rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(self.spool.get_speed() as f64)
            .get::<revolution_per_minute>();

        let live_values = LiveValuesEvent {
            traverse_position: self
                .traverse_controller
                .get_current_position()
                .map(|x| x.get::<millimeter>()),
            puller_speed: puller_speed.get::<meter_per_minute>(),
            spool_rpm,
            tension_arm_angle: angle_deg,
            spool_progress: self.spool_automatic_action.progress.get::<meter>(),
        };

        let event = live_values.build();
        self.namespace.emit(Winder2Events::LiveValues(event));
    }

    pub fn build_state_event(&mut self) -> StateEvent {
        StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            traverse_state: TraverseState {
                limit_inner: self
                    .traverse_controller
                    .get_limit_inner()
                    .get::<millimeter>(),
                limit_outer: self
                    .traverse_controller
                    .get_limit_outer()
                    .get::<millimeter>(),
                position_in: self
                    .traverse_controller
                    .get_limit_inner()
                    .get::<millimeter>(),
                position_out: self
                    .traverse_controller
                    .get_limit_outer()
                    .get::<millimeter>(),
                is_going_in: self.traverse_controller.is_going_in(),
                is_going_out: self.traverse_controller.is_going_out(),
                is_homed: self.traverse_controller.is_homed(),
                is_going_home: self.traverse_controller.is_going_home(),
                is_traversing: self.traverse_controller.is_traversing(),
                laserpointer: self.laser.get(),
                step_size: self.traverse_controller.get_step_size().get::<millimeter>(),
                padding: self.traverse_controller.get_padding().get::<millimeter>(),
                can_go_in: self.can_go_in(),
                can_go_out: self.can_go_out(),
                can_go_home: self.can_go_home(),
            },
            puller_state: PullerState {
                regulation: self.puller_speed_controller.regulation_mode.clone(),
                target_speed: self
                    .puller_speed_controller
                    .target_speed
                    .get::<meter_per_minute>(),
                target_diameter: self
                    .puller_speed_controller
                    .target_diameter
                    .get::<millimeter>(),
                forward: self.puller_speed_controller.forward,
            },
            mode_state: ModeState {
                mode: self.mode.clone().into(),
                can_wind: self.can_wind(),
            },
            tension_arm_state: TensionArmState {
                zeroed: self.tension_arm.zeroed,
            },
            spool_speed_controller_state: SpoolSpeedControllerState {
                regulation_mode: self.spool_speed_controller.get_type().clone(),
                minmax_min_speed: self
                    .spool_speed_controller
                    .get_minmax_min_speed()
                    .get::<revolution_per_minute>(),
                minmax_max_speed: self
                    .spool_speed_controller
                    .get_minmax_max_speed()
                    .get::<revolution_per_minute>(),
                adaptive_tension_target: self.spool_speed_controller.get_adaptive_tension_target(),
                adaptive_radius_learning_rate: self
                    .spool_speed_controller
                    .get_adaptive_radius_learning_rate(),
                adaptive_max_speed_multiplier: self
                    .spool_speed_controller
                    .get_adaptive_max_speed_multiplier(),
                adaptive_acceleration_factor: self
                    .spool_speed_controller
                    .get_adaptive_acceleration_factor(),
                adaptive_deacceleration_urgency_multiplier: self
                    .spool_speed_controller
                    .get_adaptive_deacceleration_urgency_multiplier(),
            },
            spool_automatic_action_state: SpoolAutomaticActionState {
                spool_required_meters: self.spool_automatic_action.target_length.get::<meter>(),
                spool_automatic_action_mode: self.spool_automatic_action.mode.clone(),
            },
            connected_machine_state: ConnectedMachineState {
                machine_identification_unique: self.connected_buffer.as_ref().map(
                    |connected_machine| {
                        ConnectedMachineData::from(connected_machine).machine_identification_unique
                    },
                ),
                is_available: self
                    .connected_buffer
                    .as_ref()
                    .map(|connected_machine| {
                        ConnectedMachineData::from(connected_machine).is_available
                    })
                    .unwrap_or(false),
            },
        }
    }
    pub fn emit_state(&mut self) {
        let state_event = self.build_state_event();
        let event = state_event.build();
        self.namespace.emit(Winder2Events::State(event));
    }
}
