use std::time::Duration;

use control_core::transmission::fixed::FixedTransmission;
use qitech_lib::{ethercat_hal::{devices::el3204::EL3204Port, io::{analog_input::AnalogInput, digital_output::DigitalOutput, serial_interface::SerialInterface, temperature_input::TemperatureInput}}, units::{AngularVelocity, Pressure, ThermodynamicTemperature, angular_velocity::revolution_per_minute, pressure::bar, thermodynamic_temperature::degree_celsius}};

use crate::{MachineHardware, MachineMessage, MachineNew, extruder1::{Heating, mitsubishi_cs80::MitsubishiCS80, screw_speed_controller::ScrewSpeedController}};
use super::{ExtruderV3, temperature_controller::TemperatureController};




impl MachineNew for ExtruderV3 {
    fn new(hw : MachineHardware) -> Result<Self, anyhow::Error> {
            
        let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(2);

        let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1);
        let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2);
        let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3);
        let t4 = TemperatureInput::new(el3204, EL3204Port::T4);

        
        let digital_out_1 = DigitalOutput::new(el2004.clone(), EL2004Port::DO1);
        let digital_out_2 = DigitalOutput::new(el2004.clone(), EL2004Port::DO2);
        let digital_out_3 = DigitalOutput::new(el2004.clone(), EL2004Port::DO3);
        let digital_out_4 = DigitalOutput::new(el2004.clone(), EL2004Port::DO4);

        let pressure_sensor = AnalogInput::new(el3021, EL3021Port::AI1);
        // The Extruders temparature Controllers should disable the relais when the max_temperature is reached
        let extruder_max_temperature = ThermodynamicTemperature::new::<degree_celsius>(300.0);
        // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
        let temperature_controller_front = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            t1,
            digital_out_1,
            Heating::default(),
            Duration::from_millis(500),
            900.0,
            1.0,
        );

        // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
        let temperature_controller_middle = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            t2,
            digital_out_2,
            Heating::default(),
            Duration::from_millis(500),
            900.0,
            1.0,
        );

        // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
        let temperature_controller_back = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            t3,
            digital_out_3,
            Heating::default(),
            Duration::from_millis(500),
            900.0,
            1.0,
        );

        // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
        let temperature_controller_nozzle = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            t4,
            digital_out_4,
            Heating::default(),
            Duration::from_millis(500),
            150.0,
            0.95,
        );

        let inverter = MitsubishiCS80::new(SerialInterface::new(el6021, EL6021Port::SI1));

        let target_pressure = Pressure::new::<bar>(0.0);
        let target_rpm = AngularVelocity::new::<revolution_per_minute>(0.0);
        let motor_poles = 2;

        let screw_speed_controller = ScrewSpeedController::new(
            inverter,
            target_pressure,
            target_rpm,
            pressure_sensor,
            FixedTransmission::new(1.0 / 30.0),
            motor_poles,
        );

        
        Ok(())
    }
}
