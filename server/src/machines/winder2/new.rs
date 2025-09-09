use std::time::Instant;

use super::api::Winder2Namespace;
use super::tension_arm::TensionArm;
use super::{Winder2, Winder2Mode};
use crate::machines::winder2::puller_speed_controller::PullerSpeedController;
use crate::machines::winder2::spool_speed_controller::SpoolSpeedController;
use crate::machines::winder2::traverse_controller::TraverseController;
use anyhow::Error;
use control_core::converters::angular_step_converter::AngularStepConverter;
use control_core::converters::linear_step_converter::LinearStepConverter;
use control_core::machines::identification::DeviceHardwareIdentification;
use control_core::machines::new::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_device_identification_by_role,
    get_ethercat_device_by_index, get_subdevice_by_index, validate_no_role_dublicates,
    validate_same_machine_identification_unique,
};
use control_core::uom_extensions::velocity::meter_per_minute;
use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::ek1100::EK1100;
use ethercat_hal::devices::el2002::{EL2002, EL2002_IDENTITY_B, EL2002Port};
use ethercat_hal::devices::el7031::coe::EL7031Configuration;
use ethercat_hal::devices::el7031::pdo::EL7031PredefinedPdoAssignment;
use ethercat_hal::devices::el7031::{
    EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B, EL7031DigitalInputPort, EL7031StepperPort,
};
use ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
use ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
use ethercat_hal::devices::el7031_0030::{
    self, EL7031_0030, EL7031_0030_IDENTITY_A, EL7031_0030AnalogInputPort, EL7031_0030StepperPort,
};
use ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
use ethercat_hal::devices::el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port};
use ethercat_hal::devices::{EthercatDeviceUsed, downcast_device, subdevice_identity_to_tuple};
use ethercat_hal::devices::{ek1100::EK1100_IDENTITY_A, el2002::EL2002_IDENTITY_A};
use ethercat_hal::io::analog_input::AnalogInput;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::shared_config;
use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
use uom::ConstZero;
use uom::si::f64::{Length, Velocity};
use uom::si::length::{centimeter, meter, millimeter};

impl MachineNewTrait for Winder2 {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();

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
            // Role 0
            // Buscoupler
            // EK1100
            {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 0)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        _ => Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not Ethercat",
                            module_path!()
                        ))?, //uncommented
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EK1100_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EK1100>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
            }

            // Role 1
            // 2x Digitalausgang
            // EL2002
            let el2002 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 1)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        _ => Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 1 is not Ethercat",
                            module_path!()
                        ))?, //uncommented
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL2002_IDENTITY_A | EL2002_IDENTITY_B => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL2002>(ethercat_device).await?
                    }
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Winder2::new] Device with role 1 is not an EL2002",
                        module_path!()
                    ))?,
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                device
            };

            // Role 2
            // 1x Stepper Spool
            // EL7041-0052
            let (el7041, _el7041_config) = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 2)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        _ => Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 2 is not Ethercat",
                            module_path!()
                        ))?,
                    };
                let subdevice = get_subdevice_by_index(
                    hardware.subdevices,
                    device_hardware_identification_ethercat.subdevice_index,
                )?;
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL7041_0052_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL7041_0052>(ethercat_device).await?
                    }
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Winder2::new] Device with role 3 is not an EL7041-0052",
                        module_path!()
                    ))?,
                };
                let config = EL7041_0052Configuration {
                    stm_features: shared_config::el70x1::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 6000,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                device
                    .write()
                    .await
                    .write_config(&subdevice, &config)
                    .await?;
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                (device, config)
            };

            // Role 3
            // 1x Stepper Traverse
            // EL7031
            let (el7031, _el7031_config) = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 3)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        _ => Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 3 is not Ethercat",
                            module_path!()
                        ))?,
                    };
                let subdevice = get_subdevice_by_index(
                    hardware.subdevices,
                    device_hardware_identification_ethercat.subdevice_index,
                )?;
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL7031_IDENTITY_A | EL7031_IDENTITY_B => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL7031>(ethercat_device).await?
                    }
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Winder2::new] Device with role 4 is not an EL7031",
                        module_path!()
                    ))?,
                };
                let config = EL7031Configuration {
                    stm_features: shared_config::el70x1::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        // Max Speed of 1000 steps/s
                        // Max @ 9cm diameter = approx 85 m/min
                        // Max @ 20cm diameter = approx 185 m/min
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
                    .write()
                    .await
                    .write_config(&subdevice, &config)
                    .await?;
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                (device, config)
            };

            // Role 4
            // 1x Stepper Puller
            // EL7031
            let (el7031_0030, _el7031_0030_config) = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 4)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        _ => Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 4 is not Ethercat",
                            module_path!()
                        ))?,
                    };
                let subdevice = get_subdevice_by_index(
                    hardware.subdevices,
                    device_hardware_identification_ethercat.subdevice_index,
                )?;
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL7031_0030_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL7031_0030>(ethercat_device).await?
                    }
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Winder2::new] Device with role 5 is not an EL7031-0030",
                        module_path!()
                    ))?,
                };
                let config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        // Max Speed of 1000 steps/s
                        // Max @ 8cm diameter = approx 75 m/min
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
                    .write()
                    .await
                    .write_config(&subdevice, &config)
                    .await?;
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                (device, config)
            };

            let mode = Winder2Mode::Standby;

            let machine_id = params
                .device_group
                .first()
                .expect("device group must have at least one device")
                .device_machine_identification
                .machine_identification_unique
                .clone();

            let mut new = Self {
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
                namespace: Winder2Namespace::new(params.socket_queue_tx.clone()),
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
                machine_manager: params.machine_manager.clone(),
                machine_identification_unique: machine_id,
                connected_buffer: None,
                last_state_event: None,
            };

            // initalize events
            new.emit_state();
            Ok(new)
        })
    }
}
