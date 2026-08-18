use std::time::{Duration, Instant};

use qitech_framework::machine::ActResult;
use qitech_framework::machine::Machine;

use crate::machines::extruder1::{Extruder, Mode};

impl<const VARIANT: usize> Machine for Extruder<VARIANT> {
    fn act(&mut self, _dt: Duration) -> ActResult {
        // The controllers time themselves off `Instant` (PWM windows, PID dt, Modbus frame
        // timing), so the wall clock is what they need rather than the tick delta.
        let now = Instant::now();

        {
            let relais = self.relais_output.clone();
            let mut relais = relais.borrow_mut();
            let relais_ref = &mut *relais;

            let temp_sensor = self.temperature_input.clone();
            let temp_sensor = temp_sensor.borrow();
            let temp_sensor_ref = &*temp_sensor;

            let serial_interface = self.serial_interface.clone();
            let mut serial_interface = serial_interface.borrow_mut();
            let serial_interface_ref = &mut *serial_interface;

            let pressure_sensor = self.pressure_sensor.clone();
            let pressure_sensor = pressure_sensor.borrow();
            let pressure_sensor_ref = &*pressure_sensor;

            self.temperature_controller_back
                .update(now, relais_ref, temp_sensor_ref);
            self.temperature_controller_nozzle
                .update(now, relais_ref, temp_sensor_ref);
            self.temperature_controller_front
                .update(now, relais_ref, temp_sensor_ref);
            self.temperature_controller_middle
                .update(now, relais_ref, temp_sensor_ref);

            let is_extruding = self.mode() == Mode::Extrude;

            self.screw_speed_controller.update(
                now,
                is_extruding,
                serial_interface_ref,
                pressure_sensor_ref,
            );

            if self.mode() == Mode::Standby {
                self.temperature_controller_back.disable(relais_ref);
                self.temperature_controller_front.disable(relais_ref);
                self.temperature_controller_middle.disable(relais_ref);
                self.temperature_controller_nozzle.disable(relais_ref);
            }

            // The screw controller drops out of Extrude on its own (pressure limit, or a stop
            // request), so fall back to Heat when it reports the motor is off.
            if self.mode() == Mode::Extrude && !self.screw_speed_controller.get_motor_enabled() {
                self.screw_speed_controller.turn_motor_off();
                self.screw_speed_controller.reset_pid();
                self.set_mode_internal(Mode::Heat);
            }
        }

        self.update_energy(now);

        Ok(())
    }
}
