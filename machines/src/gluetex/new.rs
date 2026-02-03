mod gluetex_imports {
    pub use super::super::api::GluetexNamespace;
    pub use super::super::controllers::puller_speed_controller::PullerSpeedController;
    pub use super::super::controllers::slave_puller_speed_controller::SlavePullerSpeedController;
    pub use super::super::controllers::spool_speed_controller::SpoolSpeedController;
    pub use super::super::controllers::tension_arm::TensionArm;
    pub use super::super::controllers::traverse_controller::TraverseController;
    pub use super::super::features::filament_tension::FilamentTensionCalculator;
    pub use super::super::{Gluetex, GluetexMode, PullerMode};
    pub use crate::{
        MachineNewHardware, MachineNewParams, MachineNewTrait, validate_no_role_dublicates,
        validate_same_machine_identification_unique,
    };
    pub use anyhow::Error;
    pub use control_core::converters::angular_step_converter::AngularStepConverter;
    pub use control_core::converters::linear_step_converter::LinearStepConverter;

    pub use ethercat_hal::coe::ConfigurableDevice;
    pub use ethercat_hal::devices::ek1100::EK1100;
    pub use ethercat_hal::devices::el2002::{EL2002, EL2002_IDENTITY_B, EL2002Port};
    pub use ethercat_hal::devices::el2008::{
        EL2008, EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008Port,
    };
    pub use ethercat_hal::devices::el3204::{
        EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port,
    };
    pub use ethercat_hal::devices::el7031::coe::EL7031Configuration;
    pub use ethercat_hal::devices::el7031::pdo::EL7031PredefinedPdoAssignment;
    pub use ethercat_hal::devices::el7031::{
        EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B, EL7031DigitalInputPort, EL7031StepperPort,
    };
    pub use ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
    pub use ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
    pub use ethercat_hal::devices::el7031_0030::{
        self, EL7031_0030, EL7031_0030_IDENTITY_A, EL7031_0030AnalogInputPort,
        EL7031_0030DigitalInputPort, EL7031_0030StepperPort,
    };
    pub use ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
    pub use ethercat_hal::devices::el7041_0052::{
        EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port,
    };
    pub use ethercat_hal::devices::{ek1100::EK1100_IDENTITY_A, el2002::EL2002_IDENTITY_A};
    pub use ethercat_hal::io::analog_input::AnalogInput;
    pub use ethercat_hal::io::digital_input::DigitalInput;
    pub use ethercat_hal::io::digital_output::DigitalOutput;
    pub use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
    pub use ethercat_hal::io::temperature_input::TemperatureInput;
    pub use ethercat_hal::shared_config;
    pub use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
    pub use std::time::{Duration, Instant};
    pub use units::ConstZero;
    pub use units::angle::degree;
    pub use units::f64::*;
    pub use units::length::{centimeter, meter, millimeter};
    pub use units::thermodynamic_temperature::degree_celsius;
    pub use units::velocity::meter_per_minute;
}

pub use gluetex_imports::*;

use crate::get_ethercat_device;
use crate::gluetex::controllers::temperature_controller::TemperatureController;

impl MachineNewTrait for Gluetex {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff

        let device_identification = params.device_group.to_vec();

        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Gluetex::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        // using block_on because making this funciton async creates a lifetime issue
        // if its async the compiler thinks &subdevices is persisted in the future which might never execute
        // so we can't drop subdevices unless this machine is dropped, which is bad
        smol::block_on(async {
            // Role 0: Buscoupler EK1100
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, vec![EK1100_IDENTITY_A]).await?;

            // Role 1: 2x Digital outputs EL2002
            let el2002 = get_ethercat_device::<EL2002>(
                hardware,
                params,
                1,
                vec![EL2002_IDENTITY_A, EL2002_IDENTITY_B],
            )
            .await?
            .0;

            // Role 2: Stepper Spool EL7041-0052
            let el7041 = {
                let device = get_ethercat_device::<EL7041_0052>(
                    hardware,
                    params,
                    2,
                    vec![EL7041_0052_IDENTITY_A],
                )
                .await?;

                let el7041_config = EL7041_0052Configuration {
                    stm_features: shared_config::el70x1::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 2800,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7041_config)
                    .await?;
                device.0
            };

            // Role 3: Stepper Traverse EL7031
            let el7031 = {
                let device = get_ethercat_device::<EL7031>(
                    hardware,
                    params,
                    3,
                    vec![EL7031_IDENTITY_A, EL7031_IDENTITY_B],
                )
                .await?;

                let el7031_config = EL7031Configuration {
                    stm_features: shared_config::el70x1::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_config)
                    .await?;

                device.0
            };

            // Role 4: Stepper Puller EL7031-0030
            let el7031_0030 = {
                let device = get_ethercat_device::<EL7031_0030>(
                    hardware,
                    params,
                    4,
                    vec![EL7031_0030_IDENTITY_A],
                )
                .await?;

                let el7031_0030_config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 2700,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };
                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_0030_config)
                    .await?;
                device.0
            };

            // Role 5: Temperature Sensors 1-4 EL3204
            let el3204_1 = get_ethercat_device::<EL3204>(
                hardware,
                params,
                5,
                vec![EL3204_IDENTITY_A, EL3204_IDENTITY_B],
            )
            .await?
            .0;

            // Role 6: Temperature Sensors 5-6 (and 2 spare ports) EL3204
            let el3204_2 = get_ethercat_device::<EL3204>(
                hardware,
                params,
                6,
                vec![EL3204_IDENTITY_A, EL3204_IDENTITY_B],
            )
            .await?
            .0;

            // Temperature inputs
            let temperature_1 = TemperatureInput::new(el3204_1.clone(), EL3204Port::T1);
            let temperature_2 = TemperatureInput::new(el3204_1.clone(), EL3204Port::T2);
            let temperature_3 = TemperatureInput::new(el3204_1.clone(), EL3204Port::T3);
            let temperature_4 = TemperatureInput::new(el3204_1, EL3204Port::T4);
            let temperature_5 = TemperatureInput::new(el3204_2.clone(), EL3204Port::T1);
            let temperature_6 = TemperatureInput::new(el3204_2, EL3204Port::T2);

            // Role 7: Digital outputs for heater SSRs EL2008
            let el2008 = get_ethercat_device::<EL2008>(
                hardware,
                params,
                7,
                vec![EL2008_IDENTITY_A, EL2008_IDENTITY_B],
            )
            .await?
            .0;

            // Role 8: Addon Motor 3 EL7031-0030 (with endstop for konturrad functionality)
            let el7031_addon3_shared = {
                let device = get_ethercat_device::<EL7031_0030>(
                    hardware,
                    params,
                    8,
                    vec![EL7031_0030_IDENTITY_A, EL7031_IDENTITY_A, EL7031_IDENTITY_B],
                )
                .await?;

                let el7031_0030_config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_0030_config)
                    .await?;

                device.0
            };

            // Role 9: Slave Puller EL7031-0030 (with analog input for tension arm)
            let el7031_0030_slave = {
                let device = get_ethercat_device::<EL7031_0030>(
                    hardware,
                    params,
                    9,
                    vec![EL7031_0030_IDENTITY_A, EL7031_IDENTITY_A, EL7031_IDENTITY_B],
                )
                .await?;

                let el7031_0030_config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 2700,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };
                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_0030_config)
                    .await?;
                device.0
            };

            // Role 10: Addon Motor 4 EL7031-0030
            let el7031_addon4 = {
                let device = get_ethercat_device::<EL7031_0030>(
                    hardware,
                    params,
                    10,
                    vec![EL7031_0030_IDENTITY_A, EL7031_IDENTITY_A, EL7031_IDENTITY_B],
                )
                .await?;

                let el7031_0030_config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_0030_config)
                    .await?;

                device.0
            };

            // Role 11: Addon Motor 5 EL7031-0030
            let el7031_addon5 = {
                let device = get_ethercat_device::<EL7031_0030>(
                    hardware,
                    params,
                    11,
                    vec![EL7031_0030_IDENTITY_A, EL7031_IDENTITY_A, EL7031_IDENTITY_B],
                )
                .await?;

                let el7031_0030_config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_0030_config)
                    .await?;

                device.0
            };

            // Digital outputs for SSR control (24V to external SSRs for 60W heaters)
            let heater_ssr_1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1);
            let heater_ssr_2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2);
            let heater_ssr_3 = DigitalOutput::new(el2008.clone(), EL2008Port::DO3);
            let heater_ssr_4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4);
            let heater_ssr_5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5);
            let heater_ssr_6 = DigitalOutput::new(el2008, EL2008Port::DO6);

            // Maximum temperature for all heating zones
            let max_temperature = ThermodynamicTemperature::new::<degree_celsius>(300.0);

            // PID-controlled temperature controllers (60W heaters via SSR)
            let temperature_controller_1 = TemperatureController::new(
                0.079,
                0.0006,
                2.568,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                max_temperature,
                temperature_1,
                heater_ssr_1,
                super::Heating::default(),
                Duration::from_millis(500),
                60.0, // 60W heater
                1.0,
            );

            let temperature_controller_2 = TemperatureController::new(
                0.066,
                0.0004,
                2.566,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                max_temperature,
                temperature_2,
                heater_ssr_2,
                super::Heating::default(),
                Duration::from_millis(500),
                60.0,
                1.0,
            );

            let temperature_controller_3 = TemperatureController::new(
                0.076,
                0.0005,
                2.911,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                max_temperature,
                temperature_3,
                heater_ssr_3,
                super::Heating::default(),
                Duration::from_millis(500),
                60.0,
                1.0,
            );

            let temperature_controller_4 = TemperatureController::new(
                0.073,
                0.0005,
                2.62,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                max_temperature,
                temperature_4,
                heater_ssr_4,
                super::Heating::default(),
                Duration::from_millis(500),
                60.0,
                1.0,
            );

            let temperature_controller_5 = TemperatureController::new(
                0.078,
                0.0005,
                2.920,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                max_temperature,
                temperature_5,
                heater_ssr_5,
                super::Heating::default(),
                Duration::from_millis(500),
                60.0,
                1.0,
            );

            let temperature_controller_6 = TemperatureController::new(
                0.066,
                0.0004,
                2.593,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                max_temperature,
                temperature_6,
                heater_ssr_6,
                super::Heating::default(),
                Duration::from_millis(500),
                60.0,
                1.0,
            );

            let mode = GluetexMode::Standby;

            let machine_id = params
                .device_group
                .first()
                .expect("device group must have at least one device")
                .device_machine_identification
                .machine_identification_unique
                .clone();
            let (sender, receiver) = smol::channel::unbounded();
            let mut new = Self {
                main_sender: params.main_thread_channel.clone(),
                max_connected_machines: 2,
                api_receiver: receiver,
                api_sender: sender,
                traverse: StepperVelocityEL70x1::new(el7031.clone(), EL7031StepperPort::STM1),
                traverse_end_stop: DigitalInput::new(el7031, EL7031DigitalInputPort::DI1),
                temperature_controller_1,
                temperature_controller_2,
                temperature_controller_3,
                temperature_controller_4,
                temperature_controller_5,
                temperature_controller_6,
                heating_enabled: false,
                puller: StepperVelocityEL70x1::new(
                    el7031_0030.clone(),
                    EL7031_0030StepperPort::STM1,
                ),
                spool: StepperVelocityEL70x1::new(el7041, EL7041_0052Port::STM1),
                addon_motor_3: StepperVelocityEL70x1::new(
                    el7031_addon3_shared.clone(),
                    EL7031_0030StepperPort::STM1,
                ),
                addon_motor_3_endstop: DigitalInput::new(
                    el7031_addon3_shared.clone(),
                    EL7031_0030DigitalInputPort::DI1,
                ),
                addon_motor_3_analog_input: AnalogInput::new(
                    el7031_addon3_shared,
                    EL7031_0030AnalogInputPort::AI1,
                ),
                addon_motor_4: StepperVelocityEL70x1::new(
                    el7031_addon4.clone(),
                    EL7031_0030StepperPort::STM1,
                ),
                addon_motor_5: StepperVelocityEL70x1::new(
                    el7031_addon5,
                    EL7031_0030StepperPort::STM1,
                ),
                addon_tension_arm: TensionArm::new(AnalogInput::new(
                    el7031_addon4,
                    EL7031_0030AnalogInputPort::AI1,
                )),
                addon_motor_3_controller:
                    super::controllers::addon_motor_controller::AddonMotorController::new(200),
                addon_motor_4_controller:
                    super::controllers::addon_motor_controller::AddonMotorController::new(200),
                addon_motor_5_controller:
                    super::controllers::addon_motor_controller::AddonMotorController::new(200),
                addon_motor_3_last_sync: Instant::now(),
                tension_arm: TensionArm::new(AnalogInput::new(
                    el7031_0030,
                    EL7031_0030AnalogInputPort::AI1,
                )),
                laser: DigitalOutput::new(el2002, EL2002Port::DO1),
                namespace: GluetexNamespace {
                    namespace: params.namespace.clone(),
                },
                mode: mode.clone(),
                spool_step_converter: AngularStepConverter::new(200),
                spool_speed_controller: SpoolSpeedController::new(),
                last_measurement_emit: Instant::now(),
                last_state_emit: Instant::now(),
                spool_mode: mode.clone().into(),
                traverse_mode: mode.clone().into(),
                puller_mode: mode.clone().into(),
                puller_speed_controller: PullerSpeedController::new(
                    Velocity::new::<meter_per_minute>(1.0),
                    Length::new::<millimeter>(1.75),
                    LinearStepConverter::from_diameter(
                        200,                            // Assuming 200 steps per revolution for the puller stepper,
                        Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                    ),
                ),
                slave_puller: StepperVelocityEL70x1::new(
                    el7031_0030_slave.clone(),
                    EL7031_0030StepperPort::STM1,
                ),
                slave_tension_arm: TensionArm::new(AnalogInput::new(
                    el7031_0030_slave,
                    EL7031_0030AnalogInputPort::AI1,
                )),
                slave_puller_speed_controller: SlavePullerSpeedController::new(
                    Angle::new::<degree>(20.0), // Min angle (low tension, high speed)
                    Angle::new::<degree>(90.0), // Max angle (high tension, low speed)
                    LinearStepConverter::from_diameter(
                        200,                            // 200 steps per revolution
                        Length::new::<centimeter>(8.0), // 8cm diameter
                    ),
                    FilamentTensionCalculator::new(
                        Angle::new::<degree>(20.0), // Min angle for tension calc
                        Angle::new::<degree>(90.0), // Max angle for tension calc
                    ),
                ),
                slave_puller_mode: mode.clone().into(),
                slave_puller_user_enabled: false, // Default to disabled
                traverse_controller: TraverseController::new(
                    Length::new::<millimeter>(22.0), // Default inner limit
                    Length::new::<millimeter>(92.0), // Default outer limit
                    64,                              // Microsteps
                ),
                emitted_default_state: false,
                spool_automatic_action: super::SpoolAutomaticAction {
                    progress: Length::ZERO,
                    progress_last_check: Instant::now(),
                    target_length: Length::new::<meter>(250.0),
                    mode: super::api::SpoolAutomaticActionMode::NoAction,
                },
                machine_identification_unique: machine_id,
                connected_machines: vec![],
                tension_arm_monitor_config: super::TensionArmMonitorConfig::default(),
                tension_arm_monitor_triggered: false,
                sleep_timer_config: super::SleepTimerConfig::default(),
                last_activity_time: Instant::now(),
                last_emitted_sleep_timer_remaining: 0,
                sleep_mode_triggered: false,
            };

            // initalize events
            new.emit_state();
            Ok(new)
        })
    }
}
