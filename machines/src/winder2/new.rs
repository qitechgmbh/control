#[cfg(not(feature = "mock-machine"))]
mod winder2_imports {
    pub use super::super::api::Winder2Namespace;
    pub use super::super::tension_arm::TensionArm;
    pub use super::super::{Winder2, Winder2Mode};
    pub use crate::winder2::puller_speed_controller::PullerSpeedController;
    pub use crate::winder2::spool_speed_controller::SpoolSpeedController;
    pub use crate::winder2::traverse_controller::TraverseController;
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
    pub use ethercat_hal::devices::el7031::coe::EL7031Configuration;
    pub use ethercat_hal::devices::el7031::pdo::EL7031PredefinedPdoAssignment;
    pub use ethercat_hal::devices::el7031::{
        EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B, EL7031DigitalInputPort, EL7031StepperPort,
    };
    pub use ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
    pub use ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
    pub use ethercat_hal::devices::el7031_0030::{
        self, EL7031_0030, EL7031_0030_IDENTITY_A, EL7031_0030AnalogInputPort,
        EL7031_0030StepperPort,
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
    pub use ethercat_hal::shared_config;
    pub use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
    pub use std::time::Instant;
    pub use units::ConstZero;
    pub use units::f64::*;
    pub use units::length::{centimeter, meter, millimeter};
    pub use units::velocity::meter_per_minute;
}

#[cfg(not(feature = "mock-machine"))]
pub use winder2_imports::*;

#[cfg(not(feature = "mock-machine"))]
use crate::get_ethercat_device;

#[cfg(not(feature = "mock-machine"))]
impl MachineNewTrait for Winder2 {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff

        let device_identification = params.device_group.to_vec();

        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Winder2::new] MachineNewHardware is not Ethercat",
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

            let mode = Winder2Mode::Standby;

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
                puller: StepperVelocityEL70x1::new(
                    el7031_0030.clone(),
                    EL7031_0030StepperPort::STM1,
                ),
                spool: StepperVelocityEL70x1::new(el7041, EL7041_0052Port::STM1),
                tension_arm: TensionArm::new(AnalogInput::new(
                    el7031_0030,
                    EL7031_0030AnalogInputPort::AI1,
                )),
                laser: DigitalOutput::new(el2002, EL2002Port::DO1),
                namespace: Winder2Namespace {
                    namespace: params.namespace.clone(),
                },
                mode: mode.clone(),
                spool_step_converter: AngularStepConverter::new(200),
                spool_speed_controller: SpoolSpeedController::new(),
                last_measurement_emit: Instant::now(),
                spool_mode: mode.clone().into(),
                traverse_mode: mode.clone().into(),
                puller_mode: mode.into(),
                puller_speed_controller: PullerSpeedController::new(
                    Velocity::new::<meter_per_minute>(1.0),
                    Length::new::<millimeter>(1.75),
                    LinearStepConverter::from_diameter(
                        200,                            // Assuming 200 steps per revolution for the puller stepper,
                        Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                    ),
                ),
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
            };

            // initalize events
            new.emit_state();
            Ok(new)
        })
    }
}
