use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildError;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine_build;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::ek1100::EK1100;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el2008::EL2008;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el3024::EL3024;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el4002::EL4002;
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::analog_output::AnalogOutputDevice;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::angular_velocity::revolution_per_minute;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;

use super::AquaPathV1;
use crate::machines::aquapath::AquaPathV1Mode;
use crate::machines::aquapath::controller::Controller;
use crate::machines::aquapath::controller::ControllerConfig;
use crate::machines::aquapath::controller::ControllerHardware;
use crate::machines::aquapath::controller::PidGains;
use crate::machines::aquapath::reservoir::Left;
use crate::machines::aquapath::reservoir::Reservoir;
use crate::machines::aquapath::reservoir::Right;

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

fn init_el2008(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL2008>>> {
    let el2008 = ctx.find_ethercat_device_and_addr::<EL2008>(1)?;
    let res = interface.enable_dc_sync0(el2008.1);
    match res {
        Ok(_) => (),
        Err(_) => {
            return Err(BuildError::EtherCATConfigureError(
                "Failed to set sync0 for el2008".to_owned(),
            ));
        }
    }
    Ok(el2008.0)
}

fn init_el4002(ctx: &BuildContext) -> BuildResult<Rc<RefCell<EL4002>>> {
    let el4002 = ctx.find_ethercat_device_and_addr::<EL4002>(2)?;
    Ok(el4002.0)
}

fn init_el3024(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL3024>>> {
    let el3024 = ctx.find_ethercat_device_and_addr::<EL3024>(3)?;
    let res = interface.enable_dc_sync0(el3024.1);
    match res {
        Ok(_) => (),
        Err(_) => {
            return Err(BuildError::EtherCATConfigureError(
                "Failed to set sync0 for el3024".to_owned(),
            ));
        }
    }
    Ok(el3024.0)
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

        Self::new(ctx, relais_controller, as006_sensor, fan_speed_control)
    }
}

impl AquaPathV1 {
    fn init_commands(ctx: &mut BuildContext) -> BuildResult<()> {
        ctx.command("state.set_standby")
            .execute(Self::switch_to_standby)
            .build()?;
        ctx.command("state.set_auto")
            .execute(Self::switch_to_auto)
            .build()?;
        ctx.command("pump.start_left_pump")
            .execute(Self::cmd_start_pump::<Left>)
            .build()?;
        ctx.command("pump.stop_left_pump")
            .execute(Self::cmd_stop_pump::<Left>)
            .build()?;
        ctx.command("pump.start_right_pump")
            .execute(Self::cmd_start_pump::<Right>)
            .build()?;
        ctx.command("pump.stop_right_pump")
            .execute(Self::cmd_stop_pump::<Right>)
            .build()?;
        Ok(())
    }

    #[machine_build(AquaPathV1)]
    fn new(
        ctx: &mut BuildContext,
        relais_controller: Rc<RefCell<dyn DigitalOutputDevice>>,
        as006_sensor: Rc<RefCell<dyn AnalogInputDevice>>,
        fan_speed_control: Rc<RefCell<dyn AnalogOutputDevice>>,
    ) -> BuildResult<Self> {
        let controller = |hardware| {
            Controller::new(
                hardware,
                ControllerConfig::default(),
                PidGains {
                    kp: Self::DEFAULT_PID_KP,
                    ki: Self::DEFAULT_PID_KI,
                    kd: Self::DEFAULT_PID_KD,
                },
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                AngularVelocity::new::<revolution_per_minute>(100.0),
            )
        };

        let left_controller = controller(ControllerHardware {
            relays: relais_controller.clone(),
            fan: fan_speed_control.clone(),
            sensor: as006_sensor.clone(),
            pump_relay_port: LEFT_PUMP_PORT,
            heating_relay_port: LEFT_HEATING_RELAY_PORT,
            cooling_relay_port: LEFT_COOLING_RELAY_PORT,
            fan_port: LEFT_FAN_SPEED_PORT,
            flow_sensor_port: LEFT_FLOW_SENSOR_PORT,
            temperature_sensor_port: LEFT_TEMP_SENSOR_PORT,
        });

        let right_controller = controller(ControllerHardware {
            relays: relais_controller,
            fan: fan_speed_control,
            sensor: as006_sensor,
            pump_relay_port: RIGHT_PUMP_PORT,
            heating_relay_port: RIGHT_HEATING_RELAY_PORT,
            cooling_relay_port: RIGHT_COOLING_RELAY_PORT,
            fan_port: RIGHT_FAN_SPEED_PORT,
            flow_sensor_port: RIGHT_FLOW_SENSOR_PORT,
            temperature_sensor_port: RIGHT_TEMP_SENSOR_PORT,
        });

        Self::init_commands(ctx)?;

        Ok(Self {
            mode: AquaPathV1Mode::Standby,
            mode_state: ctx.state::<AquaPathV1Mode>("mode_state.mode").build()?,
            ambient_temperature_calibration: ThermodynamicTemperature::new::<degree_celsius>(22.0),
            ambient_temperature_calibration_config: ctx
                .config::<f64>("ambient_temperature_calibration")
                .on_external_changed(Self::on_ambient_temperature_calibration_changed)
                .default(22.0)
                .build()?,
            notice_event_emitter: ctx.event("notice_event").build()?,
            left: Reservoir::build_left(ctx, left_controller)?,
            right: Reservoir::build_right(ctx, right_controller)?,
        })
    }
}
