use std::{cell::RefCell, rc::Rc, time::{Duration, Instant}};
use control_core::transmission::fixed::FixedTransmission;
use qitech_lib::{ethercat_hal::{devices::{ek1100::EK1100, el2004::{EL2004},el3021::EL3021, el3204::EL3204, el6021::EL6021}, io::{analog_input::{ AnalogInputDevice}, digital_output::{ DigitalOutputDevice}, serial_interface::SerialInterface, temperature_input::TemperatureInputDevice}}, units::{AngularVelocity, Pressure, ThermodynamicTemperature, angular_velocity::revolution_per_minute, pressure::bar, thermodynamic_temperature::degree_celsius}};
use crate::{MachineHardware, MachineMessage, MachineNew};
use super::{ExtruderV2, Heating, api::ExtruderV2Namespace, mitsubishi_cs80::MitsubishiCS80, screw_speed_controller::ScrewSpeedController, temperature_controller::TemperatureController};

impl MachineNew for ExtruderV2 {
    fn new(hw: MachineHardware) -> Result<Self, anyhow::Error> {
        let ek1100 : Rc<RefCell<EK1100>> = hw.try_get_ethercat_device_by_role(0)?;
        
        let el6021 : Rc<RefCell<EL6021>> = hw.try_get_ethercat_device_by_role(1)?;
        let el2004 : Rc<RefCell<EL2004>> = hw.try_get_ethercat_device_by_role(2)?;
        let el3021 : Rc<RefCell<EL3021>> = hw.try_get_ethercat_device_by_role(3)?;
        let el3204 : Rc<RefCell<EL3204>> = hw.try_get_ethercat_device_by_role(4)?;
        let el3204 : Rc<RefCell<EL3204>> = hw.try_get_ethercat_device_by_role(5)?;
        
        let temperature_device : Rc<RefCell<dyn TemperatureInputDevice>> = el3204;
        let pressure_sensor : Rc<RefCell<dyn AnalogInputDevice>> = el3021;
        let digital_out_device : Rc<RefCell<dyn DigitalOutputDevice>> = el2004;

        // The Extruders temparature Controllers should disable the relais when the max_temperature is reached
        let extruder_max_temperature = ThermodynamicTemperature::new::<degree_celsius>(300.0);
        let temperature_controller_front = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature, 
            Heating::default(),
            Duration::from_millis(500),
            700.0,
            1.0,
            0,
            0
        );

        let temperature_controller_middle = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            Heating::default(),
            Duration::from_millis(500),
            700.0,
            1.0,
            1,
            1
        );

        let temperature_controller_back = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            Heating::default(),
            Duration::from_millis(500),
            700.0,
            1.0,
            2,
            2,
        );

        // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
        let temperature_controller_nozzle = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            Heating::default(),
            Duration::from_millis(500),
            200.0,
            0.95,
            2,
            2,
        );

        let inverter = MitsubishiCS80::new(SerialInterface::new(el6021, EL6021Port::SI1));
        let target_pressure = Pressure::new::<bar>(0.0);
            let target_rpm = AngularVelocity::new::<revolution_per_minute>(0.0);

            let motor_poles = 4;

            let screw_speed_controller = ScrewSpeedController::new(
                inverter,
                target_pressure,
                target_rpm,
                pressure_sensor,
                FixedTransmission::new(1.0 / 34.0),
                motor_poles,
            );            
            let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(2);

            let mut extruder: ExtruderV2 = Self {
                api_receiver: rx,
                api_sender: tx,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: ExtruderV2Namespace {
                    namespace: params.namespace.clone(),
                },
                last_measurement_emit: Instant::now(),
                mode:  crate::extruder1::ExtruderV2Mode::Standby,
                total_energy_kwh: 0.0,
                last_energy_calculation_time: None,
                temperature_controller_front,
                temperature_controller_middle,
                temperature_controller_back,
                temperature_controller_nozzle,
                screw_speed_controller,
                emitted_default_state: false,
                last_status_hash: None,
            };
            extruder.emit_state();
            Ok(extruder)        
    }
}
