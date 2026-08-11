use crate::machines::aquapath::{AquaPathV1Mode, Flow, Temperature, controller::{Controller, ControllerConfig}};

use super::{
    AquaPathV1
};
use qitech_framework::{machine::{BuildContext, BuildResult, MachineBuild}, machine_build};
use qitech_lib::{ethercat_hal::{
    EtherCATThreadChannel, devices::beckhoff_modules::{ek1100::EK1100, el2008::EL2008, el3024::EL3024, el4002::EL4002}, io::{
        analog_input::AnalogInputDevice, analog_output::AnalogOutputDevice,
        digital_output::DigitalOutputDevice,
    },
}, units::{AngularVelocity, ThermodynamicTemperature, angular_velocity::revolution_per_minute, thermodynamic_temperature::degree_celsius}};
use std::{cell::RefCell, rc::Rc};

// --- Analog Input Ports (EL3024) ---
const LEFT_FLOW_SENSOR_PORT: usize = 0; // AI1
const LEFT_TEMP_SENSOR_PORT: usize = 1; // AI2
const RIGHT_FLOW_SENSOR_PORT: usize = 2; // AI3
const RIGHT_TEMP_SENSOR_PORT: usize = 3; // AI4

// --- Digital Output Ports (EL2008) ---
const LEFT_PUMP_PORT: usize = 0; // DO1
const LEFT_HEATING_RELAY_PORT: usize = 1; // DO2
const LEFT_COOLING_RELAY_PORT: usize = 3; // DO4
const RIGHT_PUMP_PORT: usize = 4; // DO5
const RIGHT_HEATING_RELAY_PORT: usize = 5; // DO6
const RIGHT_COOLING_RELAY_PORT: usize = 7; // DO8

// --- Analog Output Ports (el4002) ---
const LEFT_FAN_SPEED_PORT: usize = 0; // AO1
const RIGHT_FAN_SPEED_PORT: usize = 1; // AO2


fn init_ek1100(ctx: &BuildContext) -> BuildResult<()> {
    ctx.find_ethercat_device_and_addr::<EK1100>(0)?;
    Ok(())
}


fn init_el2008(ctx: &BuildContext,interface : EtherCATThreadChannel) -> BuildResult<Rc<RefCell<EL2008>>> {
    let el2008 = ctx.find_ethercat_device_and_addr::<EL2008>(0)?;
    interface.enable_dc_sync0(el2008.1)?;
    Ok( el2008.0 )
}

fn init_el4002(ctx: &BuildContext) -> BuildResult<Rc<RefCell<EL4002>>> {
    let el4002 = ctx.find_ethercat_device_and_addr::<EL4002>(0)?;
    Ok( el4002.0 )
}

fn init_el3024(ctx: &BuildContext,interface : EtherCATThreadChannel) -> BuildResult<Rc<RefCell<EL3024>>> {
    let el3024 = ctx.find_ethercat_device_and_addr::<EL3024>(0)?;
    interface.enable_dc_sync0(el3024.1)?;
    Ok( el3024.0 )
}

impl MachineBuild for AquaPathV1 {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let interface = ctx.get_ethercat_interface()?;
        let _ = init_ek1100(ctx)?;
        let el2008 = init_el2008(ctx, interface.clone())?;
        let el4002 = init_el4002(ctx)?;
        let el3024 = init_el3024(ctx, interface.clone())?;
        let relais_controller: Rc<RefCell<dyn DigitalOutputDevice>> = el2008;
        let as006_sensor: Rc<RefCell<dyn AnalogInputDevice>> = el3024;
        let fan_speed_control: Rc<RefCell<dyn AnalogOutputDevice>> = el4002;
        
        Self::new(ctx,relais_controller, as006_sensor,fan_speed_control)
    }
}

impl AquaPathV1 {
    #[machine_build(AquaPathV1)]
    fn new(ctx: &mut BuildContext, relais_controller : Rc<RefCell<dyn DigitalOutputDevice>>,as006_sensor:Rc<RefCell<dyn AnalogInputDevice>>,fan_speed_control : Rc<RefCell<dyn AnalogOutputDevice>>) -> BuildResult<Self> {

        let controller_config = ControllerConfig::default();

        let left_controller = Controller::new(
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
            LEFT_PUMP_PORT,
            LEFT_FLOW_SENSOR_PORT,
            LEFT_FAN_SPEED_PORT,
            LEFT_COOLING_RELAY_PORT,
            LEFT_HEATING_RELAY_PORT,
            LEFT_TEMP_SENSOR_PORT,
        );

        let right_controller = Controller::new(
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
            RIGHT_PUMP_PORT,
            RIGHT_FLOW_SENSOR_PORT,
            RIGHT_FAN_SPEED_PORT,
            RIGHT_COOLING_RELAY_PORT,
            RIGHT_HEATING_RELAY_PORT,
            RIGHT_TEMP_SENSOR_PORT,
        );

        let emitter = ctx.event("notice_event").build()?;

        let machine = Self {            
            mode: AquaPathV1Mode::Standby,
            ambient_temperature_calibration: ThermodynamicTemperature::new::<degree_celsius>(22.0),
            left_controller,
            right_controller,
            notice_event_emitter : emitter
        };
        
        Ok(machine)
    }
}
