use crate::extruder1::ExtruderV2Mode;
use crate::extruder1::heating_decoupling::{IDX_BACK, IDX_FRONT, IDX_MIDDLE, IDX_NOZZLE};
use crate::{MachineApi, extruder1::ExtruderV2};
use qitech_lib::machines::{
    Machine, MachineDataRegistry, MachineError, MachineIdentificationUnique,
};
use std::time::{Duration, Instant};

impl Machine for ExtruderV2 {
    fn act(&mut self, _registry: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
        let now = Instant::now();
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };
        {
            let mut relais = self.relais_output.borrow_mut();
            let relais_ref = &mut *relais;

            let temp_sensor = self.temperature_input.borrow();
            let temp_sensor_ref = &*temp_sensor;

            let mut serial_interface = self.serial_interface.borrow_mut();
            let serial_interface_ref = &mut *serial_interface;

            let mut pressure_sensor = self.pressure_sensor.borrow_mut();
            let pressure_sensor_ref = &mut *pressure_sensor;

            // Run all four zone PIDs, mix their demands through the decoupling matrix, then
            // drive the relays. The array order is the one place the matrix's zone indices are
            // bound to the controllers, so keep the IDX_* constants rather than bare literals.
            let u_pid = [
                self.temperature_controller_nozzle
                    .compute_control(now, temp_sensor_ref),
                self.temperature_controller_front
                    .compute_control(now, temp_sensor_ref),
                self.temperature_controller_middle
                    .compute_control(now, temp_sensor_ref),
                self.temperature_controller_back
                    .compute_control(now, temp_sensor_ref),
            ];

            let u_actual = self.heating_decoupler.apply(u_pid);

            self.temperature_controller_nozzle
                .apply_output(now, relais_ref, u_actual[IDX_NOZZLE]);
            self.temperature_controller_front
                .apply_output(now, relais_ref, u_actual[IDX_FRONT]);
            self.temperature_controller_middle
                .apply_output(now, relais_ref, u_actual[IDX_MIDDLE]);
            self.temperature_controller_back
                .apply_output(now, relais_ref, u_actual[IDX_BACK]);

            if self.mode == super::ExtruderV2Mode::Extrude {
                self.screw_speed_controller.update(
                    now,
                    true,
                    serial_interface_ref,
                    pressure_sensor_ref,
                );
            } else {
                self.screw_speed_controller.update(
                    now,
                    false,
                    serial_interface_ref,
                    pressure_sensor_ref,
                );
            }

            if self.mode == super::ExtruderV2Mode::Standby {
                self.temperature_controller_back.disable(relais_ref);
                self.temperature_controller_front.disable(relais_ref);
                self.temperature_controller_middle.disable(relais_ref);
                self.temperature_controller_nozzle.disable(relais_ref);
            }

            if self.mode == super::ExtruderV2Mode::Extrude
                && !self.screw_speed_controller.get_motor_enabled()
            {
                match self.mode {
                    ExtruderV2Mode::Standby => {
                        self.temperature_controller_back.allow_heating();
                        self.temperature_controller_front.allow_heating();
                        self.temperature_controller_middle.allow_heating();
                        self.temperature_controller_nozzle.allow_heating();
                    }
                    ExtruderV2Mode::Heat => (),
                    ExtruderV2Mode::Extrude => {
                        self.screw_speed_controller.turn_motor_off();
                        self.screw_speed_controller.reset_pid();
                    }
                }
                self.mode = ExtruderV2Mode::Heat;
            }
        }

        let now = Instant::now();
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.update_total_energy(now);
            self.maybe_emit_state_event();
            // Emit live values at 30 FPS
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
        Ok(())
    }

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}
}
