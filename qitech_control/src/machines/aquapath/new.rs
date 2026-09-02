use crate::machines::aquapath::{
    AquaPathV1Mode, Flow, Temperature,
    api::{
        ConfigProperties, Measurements, ModeState, PidState, StateProperties, ThermalSafetyState,
        ToleranceState,
    },
    controller::{Controller, ControllerConfig, CoolingMode},
};

use super::AquaPathV1;
use qitech_framework::{
    machine::{BuildContext, BuildError, BuildResult, MachineBuild}, machine_build,
};
use qitech_lib::{
    ethercat_hal::{
        EtherCATThreadChannel,
        devices::beckhoff_modules::{
            ek1100::EK1100, el2008::EL2008, el3024::EL3024, el4002::EL4002,
        },
        io::{
            analog_input::AnalogInputDevice, analog_output::AnalogOutputDevice,
            digital_output::DigitalOutputDevice,
        },
    },
    units::{
        AngularVelocity, ThermodynamicTemperature, angular_velocity::revolution_per_minute,
        thermodynamic_temperature::degree_celsius,
    },
};
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

fn init_el2008(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL2008>>> {
    let el2008 = ctx.find_ethercat_device_and_addr::<EL2008>(1)?;
    let res = interface.enable_dc_sync0(el2008.1);
    match res {
        Ok(_) => (),
        Err(_) => {
            return Err(BuildError::EtherCATConfigureError("Failed to set sync0 for el2008".to_owned()));
        },
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
            return Err(BuildError::EtherCATConfigureError("Failed to set sync0 for el3024".to_owned()));
        },
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

fn init_measurements(ctx: &mut BuildContext) -> BuildResult<Measurements> {
    Ok(Measurements {
        left_flow: ctx.measurement::<f64>("left_flow").build()?,
        right_flow: ctx.measurement::<f64>("right_flow").build()?,
        left_temperature: ctx.measurement::<f64>("left_temperature").build()?,
        right_temperature: ctx.measurement::<f64>("right_temperature").build()?,
        left_revolutions: ctx.measurement::<f64>("left_revolutions").build()?,
        right_revolutions: ctx.measurement::<f64>("right_revolutions").build()?,
        left_power: ctx.measurement::<f64>("left_power").build()?,
        right_power: ctx.measurement::<f64>("right_power").build()?,
        left_total_energy: ctx.measurement::<f64>("left_total_energy").build()?,
        right_total_energy: ctx.measurement::<f64>("right_total_energy").build()?,
    })
}

#[machine_build(AquaPathV1)]
fn init_state(ctx: &mut BuildContext) -> BuildResult<StateProperties> {
    let mode_state = ModeState {
        mode: ctx.state::<AquaPathV1Mode>("mode_state.mode").build()?,
    };

    let left_thermal_safety_state = ThermalSafetyState {
        thermal_delay: ctx
            .state::<f64>("left_thermal_safety_state.thermal_delay")
            .build()?,
        cooldown_min_temperature: ctx
            .state::<f64>("left_thermal_safety_state.cooldown_min_temperature")
            .build()?,
    };

    let right_thermal_safety_state = ThermalSafetyState {
        thermal_delay: ctx
            .state::<f64>("right_thermal_safety_state.thermal_delay")
            .build()?,
        cooldown_min_temperature: ctx
            .state::<f64>("right_thermal_safety_state.cooldown_min_temperature")
            .build()?,
    };

    Ok(StateProperties {
        mode_state,
        left_heating_startup_wait_active: ctx
            .state::<bool>("left_heating_startup_wait_active")
            .build()?,
        right_heating_startup_wait_active: ctx
            .state::<bool>("right_heating_startup_wait_active")
            .build()?,
        left_pump_cooldown_active: ctx.state::<bool>("left_pump_cooldown_active").build()?,
        right_pump_cooldown_active: ctx.state::<bool>("right_pump_cooldown_active").build()?,
        left_should_flow: ctx.state::<bool>("left_should_flow").build()?,
        right_should_flow: ctx.state::<bool>("right_should_flow").build()?,
        left_heating: ctx.state::<bool>("left_heating").build()?,
        right_heating: ctx.state::<bool>("right_heating").build()?,
        left_pump_cooldown_remaining: ctx.state::<f64>("left_pump_cooldown_remaining").build()?,
        right_pump_cooldown_remaining: ctx.state::<f64>("right_pump_cooldown_remaining").build()?,
        left_heating_startup_wait_remaining: ctx
            .state::<f64>("left_heating_startup_wait_remaining")
            .build()?,
        right_heating_startup_wait_remaining: ctx
            .state::<f64>("right_heating_startup_wait_remaining")
            .build()?,
        left_cooling_mode: ctx
            .state::<Option<CoolingMode>>("left_cooling_mode")
            .build()?,
        right_cooling_mode: ctx
            .state::<Option<CoolingMode>>("right_cooling_mode")
            .build()?,
        left_thermal_safety_state,
        right_thermal_safety_state,
    })
}

impl AquaPathV1 {
    #[machine_build(AquaPathV1)]
    fn init_config(ctx: &mut BuildContext) -> BuildResult<ConfigProperties> {
        let left_tolerance_state = ToleranceState {
            heating: ctx
                .config::<f64>("left_tolerance_config.heating")
                .on_external_changed(Self::on_set_left_heating_tolerance)
                .default(0.4)
                .build()?,
            cooling: ctx
                .config::<f64>("left_tolerance_config.cooling")
                .on_external_changed(Self::on_set_left_cooling_tolerance)
                .default(0.8)
                .build()?,
        };

        let right_tolerance_state = ToleranceState {
            heating: ctx
                .config::<f64>("right_tolerance_config.heating")
                .on_external_changed(Self::on_set_right_heating_tolerance)
                .default(0.4)
                .build()?,
            cooling: ctx
                .config::<f64>("right_tolerance_config.cooling")
                .on_external_changed(Self::on_set_right_cooling_tolerance)
                .default(0.8)
                .build()?,
        };

        let left_pid_config = PidState {
            kp: ctx
                .config::<f64>("left_pid_config.kp")
                .default(AquaPathV1::DEFAULT_PID_KP)
                .on_external_changed(Self::on_set_left_pid)
                .build()?,
            ki: ctx
                .config::<f64>("left_pid_config.ki")
                .default(AquaPathV1::DEFAULT_PID_KI)
                .on_external_changed(Self::on_set_left_pid)
                .build()?,
            kd: ctx
                .config::<f64>("left_pid_config.kd")
                .default(AquaPathV1::DEFAULT_PID_KD)
                .on_external_changed(Self::on_set_left_pid)
                .build()?,
        };

        let right_pid_config = PidState {
            kp: ctx
                .config::<f64>("right_pid_config.kp")
                .default(AquaPathV1::DEFAULT_PID_KP)
                .on_external_changed(Self::on_set_right_pid)
                .build()?,
            ki: ctx
                .config::<f64>("right_pid_config.ki")
                .default(AquaPathV1::DEFAULT_PID_KI)
                .on_external_changed(Self::on_set_right_pid)
                .build()?,
            kd: ctx
                .config::<f64>("right_pid_config.kd")
                .default(AquaPathV1::DEFAULT_PID_KD)
                .on_external_changed(Self::on_set_right_pid)
                .build()?,
        };

        let props = ConfigProperties {
            left_target_temperature: ctx
                .config::<f64>("left_target_temperature")
                .on_external_changed(Self::on_left_target_temparature_changed)
                .build()?,
            right_target_temperature: ctx
                .config::<f64>("right_target_temperature")
                .on_external_changed(Self::on_right_target_temparature_changed)
                .build()?,
            ambient_temperature_calibration: ctx
                .config::<f64>("ambient_temperature_calibration")
                .on_external_changed(Self::on_set_ambient_temperature_calibration)
                .default(22.0)
                .build()?,
            left_fan_max_revolutions: ctx
                .config::<f64>("left_fan_max_revolutions")
                .on_external_changed(Self::on_set_left_revolutions)
                .default(100.0)
                .build()?,
            right_fan_max_revolutions: ctx
                .config::<f64>("right_fan_max_revolutions")
                .on_external_changed(Self::on_set_right_revolutions)
                .default(100.0)
                .minimum(0.0)
                .maximum(100.0)
                .build()?,
            left_tolerance_config: left_tolerance_state,
            right_tolerance_config: right_tolerance_state,
            left_pid_config,
            right_pid_config,
            left_thermal_flow_settle_duration: ctx
                .config::<f64>("left_thermal_flow_settle_duration")
                .on_external_changed(Self::on_set_left_thermal_flow_settle_duration)
                .default(0.0)
                .minimum(0.0)
                .maximum(30.0)
                .build()?,
            right_thermal_flow_settle_duration: ctx
                .config::<f64>("right_thermal_flow_settle_duration")
                .on_external_changed(Self::on_set_right_thermal_flow_settle_duration)
                .default(0.0)
                .minimum(0.0)
                .maximum(30.0)
                .build()?,
            left_pump_cooldown_min_temperature: ctx
                .config::<f64>("left_pump_cooldown_min_temperature")
                .on_external_changed(Self::on_set_left_cooldown_min_temp)
                .default(32.0)
                .minimum(10.0)
                .maximum(80.0)
                .build()?,
            right_pump_cooldown_min_temperature: ctx
                .config::<f64>("right_pump_cooldown_min_temperature")
                .on_external_changed(Self::on_set_right_cooldown_min_temp)
                .default(22.0)
                .minimum(10.0)
                .maximum(80.0)
                .build()?,
        };
        Ok(props)
    }

    fn init_commands(ctx: &mut BuildContext) -> BuildResult<()> {
        ctx.command("state.set_standby")
            .execute(Self::switch_to_standby)
            .build()?;
        ctx.command("state.set_auto")
            .execute(Self::switch_to_auto)
            .build()?;
        ctx.command("pump.start_right_pump")
            .execute(Self::cmd_start_right_pump)
            .build()?;
        ctx.command("pump.stop_right_pump")
            .execute(Self::cmd_stop_right_pump)
            .build()?;
        ctx.command("pump.start_left_pump")
            .execute(Self::cmd_start_left_pump)
            .build()?;
        ctx.command("pump.stop_left_pump")
            .execute(Self::cmd_stop_left_pump)
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
        Self::init_commands(ctx)?;
        let emitter = ctx.event("notice_event").build()?;
        let machine = Self {
            mode: AquaPathV1Mode::Standby,
            ambient_temperature_calibration: ThermodynamicTemperature::new::<degree_celsius>(22.0),
            left_controller,
            right_controller,
            notice_event_emitter: emitter,
            measurements: init_measurements(ctx)?,
            state_props: init_state(ctx)?,
            config_props: Self::init_config(ctx)?,
        };
        Ok(machine)
    }
}
