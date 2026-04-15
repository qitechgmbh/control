use std::time::Instant;
use anyhow::Error;
use qitech_lib::{ethercat_hal::devices::{ek1100::EK1100, el2008::EL2008, el3204::EL3204, el4002::EL4002, el5152::EL5152}, units::{AngularVelocity, ThermodynamicTemperature, angular_velocity::revolution_per_minute, thermodynamic_temperature::degree_celsius}};
use crate::{MachineHardware, MachineNew};
use super::{AquaPathV1, Flow, Temperature, api::AquaPathV1Namespace, controller::Controller};

impl MachineNew for AquaPathV1 {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let _ek1100 = hw.try_get_ethercat_device_by_role::<EK1100>(0)?;
        let el2008 = hw.try_get_ethercat_device_by_role::<EL2008>(1)?;
        let el4002 = hw.try_get_ethercat_device_by_role::<EL4002>(2)?;
        let el3204 = hw.try_get_ethercat_device_by_role::<EL3204>(3)?;
        let el5152 = hw.try_get_ethercat_device_by_role::<EL5152>(4)?;
        let (sender,receiver) = tokio::sync::mpsc::channel(2);

        const FRONT_CONTROLLER_COOLING_PORT : usize = 0;
        const FRONT_CONTROLLER_COOLING_RELAIS_PORT : usize = 3; 
        const FRONT_CONTROLLER_HEATING_RELAIS_PORT : usize = 1; 
        const FRONT_CONTROLLER_HEATING_IN_PORT : usize = 0;
        const FRONT_CONTROLLER_HEATING_OUT_PORT : usize = 1;
        const FRONT_CONTROLLER_PUMP_RELAIS_PORT : usize = 0; 
        const FRONT_CONTROLLER_FLOW_SENSOR_PORT : usize = 0;

        const BACK_CONTROLLER_COOLING_PORT : usize = 1;
        const BACK_CONTROLLER_COOLING_RELAIS_PORT : usize = 7; 
        const BACK_CONTROLLER_HEATING_RELAIS_PORT : usize = 5; 
        const BACK_CONTROLLER_HEATING_IN_PORT : usize = 2;
        const BACK_CONTROLLER_HEATING_OUT_PORT : usize = 3;
        const BACK_CONTROLLER_PUMP_RELAIS_PORT : usize = 4; 
        const BACK_CONTROLLER_FLOW_SENSOR_PORT : usize = 1;

        let front_controller = Controller::new(
                Self::DEFAULT_PID_KP,
                Self::DEFAULT_PID_KI,
                Self::DEFAULT_PID_KD,
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                el4002.clone(),
                el2008.clone(),                
                el3204.clone(),
                AngularVelocity::new::<revolution_per_minute>(100.0),
                Flow::default(),
                el5152.clone(),
                FRONT_CONTROLLER_COOLING_PORT, //ao1 cooling controller
                FRONT_CONTROLLER_COOLING_RELAIS_PORT, // do4
                FRONT_CONTROLLER_HEATING_RELAIS_PORT, // do2                
                FRONT_CONTROLLER_HEATING_IN_PORT, // t1
                FRONT_CONTROLLER_HEATING_OUT_PORT, // t2                
                FRONT_CONTROLLER_PUMP_RELAIS_PORT, // do1 pump relais
                FRONT_CONTROLLER_FLOW_SENSOR_PORT, //enc1 
        );

        let back_controller = Controller::new(
                Self::DEFAULT_PID_KP,
                Self::DEFAULT_PID_KI,
                Self::DEFAULT_PID_KD,
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                el4002.clone(),
                el2008.clone(),                
                el3204.clone(),
                AngularVelocity::new::<revolution_per_minute>(100.0),
                Flow::default(),
                el5152.clone(),
                BACK_CONTROLLER_COOLING_PORT, //ao1 cooling controller
                BACK_CONTROLLER_COOLING_RELAIS_PORT, // do4
                BACK_CONTROLLER_HEATING_RELAIS_PORT, // do2                
                BACK_CONTROLLER_HEATING_IN_PORT, // t1
                BACK_CONTROLLER_HEATING_OUT_PORT, // t2                
                BACK_CONTROLLER_PUMP_RELAIS_PORT, // do1 pump relais
                BACK_CONTROLLER_FLOW_SENSOR_PORT, //enc1 
        );


        let water_cooling = Self {
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: hw.identification,
            namespace: AquaPathV1Namespace{ namespace:None },
            mode: super::AquaPathV1Mode::Standby,
            last_measurement_emit: Instant::now(),
            front_controller,
            back_controller,
        };

        Ok(water_cooling)
    }    
}
