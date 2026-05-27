use crate::{MachineHardware, MachineNew};
use super::{
    AquaPathV1, AquaPathV1Mode,
    api::AquaPathV1Namespace,
    controller::{Controller, ControllerConfig},
};
use super::{Flow, Temperature};
use anyhow::Error;

use qitech_lib::ethercat_hal::{
    EtherCATThreadChannel, devices::{
        ek1100::EK1100, el2008::EL2008, el3024::EL3024, el4002::EL4002
    }, io::{analog_input::AnalogInputDevice, analog_output::AnalogOutputDevice, digital_output::DigitalOutputDevice}
};

use std::{cell::RefCell, rc::Rc, time::Instant};
use qitech_lib::units::{
    AngularVelocity,
    angular_velocity::revolution_per_minute,
    thermodynamic_temperature::{ThermodynamicTemperature, degree_celsius},
};

// --- Analog Input Ports (EL3024) ---
const FRONT_FLOW_SENSOR_PORT: usize = 0; // AI1
const FRONT_TEMP_SENSOR_PORT: usize = 1;        // AI2
const BACK_FLOW_SENSOR_PORT: usize = 2;  // AI3
const BACK_TEMP_SENSOR_PORT: usize = 3;  // AI4

// --- Digital Output Ports (EL2008) ---
const FRONT_PUMP_PORT: usize = 0;           // DO1
const FRONT_HEATING_RELAY_PORT: usize = 1;  // DO2
const FRONT_COOLING_RELAY_PORT: usize = 3;  // DO4
const BACK_PUMP_PORT: usize = 4;            // DO5
const BACK_HEATING_RELAY_PORT: usize = 5;   // DO6
const BACK_COOLING_RELAY_PORT: usize = 7;   // DO8

// --- Analog Output Ports (el4002) ---
const FRONT_FAN_SPEED_PORT: usize = 0; // AO1
const BACK_FAN_SPEED_PORT: usize = 1;        // AO2

impl MachineNew for AquaPathV1 {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let _ek1100 = hw.try_get_ethercat_device_and_addr_by_role::<EK1100>(0)?;
        let el2008 = hw.try_get_ethercat_device_and_addr_by_role::<EL2008>(1)?;
        let el4002 = hw.try_get_ethercat_device_and_addr_by_role::<EL4002>(2)?;
        let el3024 = hw.try_get_ethercat_device_and_addr_by_role::<EL3024>(3)?;
 
        let interface: EtherCATThreadChannel = match &hw.ethercat_interface {
            Some(ecat_interface) => ecat_interface.clone(),
            None => {
                return Err(anyhow::anyhow!(
                    "AquaPathV1: No EtherCat Interface was supplied!"
                ));
            }
        };
        
        interface.enable_dc_sync0(el2008.1)?;
        interface.enable_dc_sync0(el4002.1)?;
        interface.enable_dc_sync0(el3024.1)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(2);

        let relais_controller : Rc<RefCell<dyn DigitalOutputDevice>> = el2008.0.clone();
        let as006_sensor : Rc<RefCell<dyn AnalogInputDevice>> = el3024.0.clone();
        let fan_speed_control : Rc<RefCell<dyn AnalogOutputDevice>> =  el4002.0.clone();                
        let controller_config = ControllerConfig::default();

        let back_controller = Controller::new(
            AquaPathV1::DEFAULT_PID_KP,
            AquaPathV1::DEFAULT_PID_KI,
            AquaPathV1::DEFAULT_PID_KD,
            Temperature::default(),
            ThermodynamicTemperature::new::<degree_celsius>(25.0),
            AngularVelocity::new::<revolution_per_minute>(100.0),
            Flow::default(),
            controller_config,
            fan_speed_control.clone(),
            relais_controller.clone(),
            as006_sensor.clone(),
            FRONT_PUMP_PORT,
            FRONT_FLOW_SENSOR_PORT,
            FRONT_FAN_SPEED_PORT,
            FRONT_COOLING_RELAY_PORT,
            FRONT_HEATING_RELAY_PORT,
            FRONT_TEMP_SENSOR_PORT,
        );

        let front_controller = Controller::new(
            AquaPathV1::DEFAULT_PID_KP,
            AquaPathV1::DEFAULT_PID_KI,
            AquaPathV1::DEFAULT_PID_KD,
            Temperature::default(),
            ThermodynamicTemperature::new::<degree_celsius>(25.0),
            AngularVelocity::new::<revolution_per_minute>(100.0),
            Flow::default(),
            controller_config,
            fan_speed_control.clone(),
            relais_controller.clone(),
            as006_sensor.clone(),
            BACK_PUMP_PORT,
            BACK_FLOW_SENSOR_PORT,
            BACK_FAN_SPEED_PORT,
            BACK_COOLING_RELAY_PORT,
            BACK_HEATING_RELAY_PORT,
            BACK_TEMP_SENSOR_PORT,
        );

            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique:  hw.identification,
                namespace: AquaPathV1Namespace { namespace: None },
                mode: AquaPathV1Mode::Standby,
                ambient_temperature_calibration: ThermodynamicTemperature::new::<degree_celsius>(
                    22.0,
                ),
                last_measurement_emit: Instant::now(),
                front_controller,
                back_controller,
            };
            machine.emit_state();
            Ok(machine)
    }
}
