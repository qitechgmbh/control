use std::time::Instant;

use crate::machines::winder2::puller_speed_controller::PullerSpeedController;

use super::api::Winder1Namespace;
use super::spool_speed_controller::SpoolSpeedController;
use super::tension_arm::TensionArm;
use super::{Winder2, Winder2Mode};
use anyhow::Error;
use control_core::actors::analog_input_getter::AnalogInputGetter;
use control_core::actors::digital_output_setter::DigitalOutputSetter;
use control_core::actors::stepper_driver_el70x1::StepperDriverEL70x1;
use control_core::converters::step_converter::StepConverter;
use control_core::machines::identification::DeviceHardwareIdentification;
use control_core::machines::new::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_device_identification_by_role,
    get_ethercat_device_by_index, get_subdevice_by_index, validate_no_role_dublicates,
    validate_same_machine_identification_unique,
};
use control_core::uom_extensions::acceleration::meter_per_minute_per_second;
use control_core::uom_extensions::angular_acceleration::revolution_per_minute_per_second;
use control_core::uom_extensions::velocity::meter_per_minute;
use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::el2002::{EL2002, EL2002Port};
use ethercat_hal::devices::el7031::coe::EL7031Configuration;
use ethercat_hal::devices::el7031::pdo::EL7031PredefinedPdoAssignment;
use ethercat_hal::devices::el7031::{
    EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B, EL7031StepperPort,
};
use ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
use ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
use ethercat_hal::devices::el7031_0030::{
    self, EL7031_0030, EL7031_0030_IDENTITY_A, EL7031_0030AnalogInputPort, EL7031_0030StepperPort,
};
use ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
use ethercat_hal::devices::el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port};
use ethercat_hal::devices::{downcast_device, subdevice_identity_to_tuple};
use ethercat_hal::devices::{ek1100::EK1100_IDENTITY_A, el2002::EL2002_IDENTITY_A};
use ethercat_hal::io::analog_input::AnalogInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::shared_config;
use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
use uom::si::acceleration::meter_per_second_squared;
use uom::si::angular_velocity::revolution_per_minute;
use uom::si::f64::{Acceleration, AngularAcceleration, AngularVelocity, Length, Velocity};
use uom::si::length::millimeter;
use uom::si::velocity::kilometer_per_hour;

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
            // _ => {
            //     return Err(anyhow::anyhow!(
            //         "[{}::MachineNewTrait/Winder2::new] MachineNewHardware is not Ethercat",
            //         module_path!()
            //     ));
            // }
        };

        log::info!(
            "[{}::MachineNewTrait/Winder2::new] Hardware: Ethercat Devices {:?}, Subdevices {:?}",
            module_path!(),
            hardware.ethercat_devices.len(),
            hardware.subdevices.len()
        );

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
                        // _ => Err(anyhow::anyhow!(
                        //     "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not Ethercat",
                        //     module_path!()
                        // ))?,
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                match subdevice_identity_to_tuple(&subdevice_identity) {
                    EK1100_IDENTITY_A => (),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                };
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
                        // _ => Err(anyhow::anyhow!(
                        //     "[{}::MachineNewTrait/Winder2::new] Device with role 1 is not Ethercat",
                        //     module_path!()
                        // ))?,
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL2002_IDENTITY_A => {
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
                }
            };

            // Role 2
            // 1x Stepper Spool
            // EL7041-0052
            let (el7041, el7041_config) = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 2)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        // _ => Err(anyhow::anyhow!(
                        //     "[{}::MachineNewTrait/Winder2::new] Device with role 3 is not Ethercat",
                        //     module_path!()
                        // ))?,
                    };
                let subdevice = get_subdevice_by_index(
                    hardware.subdevices,
                    device_hardware_identification_ethercat.subdevice_index,
                )?;
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice_identity = subdevice.identity();
                let el7041 = match subdevice_identity_to_tuple(&subdevice_identity) {
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
                let el7041_config = EL7041_0052Configuration {
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
                el7041
                    .write()
                    .await
                    .write_config(&subdevice, &el7041_config)
                    .await?;
                (el7041, el7041_config)
            };

            // Role 3
            // 1x Stepper Traverse
            // EL7031
            let (el7031, el7031_config) = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 3)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        // _ => Err(anyhow::anyhow!(
                        //     "[{}::MachineNewTrait/Winder2::new] Device with role 4 is not Ethercat",
                        //     module_path!()
                        // ))?,
                    };
                let subdevice = get_subdevice_by_index(
                    hardware.subdevices,
                    device_hardware_identification_ethercat.subdevice_index,
                )?;
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice_identity = subdevice.identity();
                let el7031 = match subdevice_identity_to_tuple(&subdevice_identity) {
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
                let el7031_config = EL7031Configuration {
                    stm_features: shared_config::el70x1::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };
                el7031
                    .write()
                    .await
                    .write_config(&subdevice, &el7031_config)
                    .await?;
                (el7031, el7031_config)
            };

            // Role 4
            // 1x Stepper Puller
            // EL7031
            let (el7031_0030, el7031_0030_config) = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 4)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        // _ => Err(anyhow::anyhow!(
                        //     "[{}::MachineNewTrait/Winder2::new] Device with role 4 is not Ethercat",
                        //     module_path!()
                        // ))?,
                    };
                let subdevice = get_subdevice_by_index(
                    hardware.subdevices,
                    device_hardware_identification_ethercat.subdevice_index,
                )?;
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice_identity = subdevice.identity();
                let el7031_0030 = match subdevice_identity_to_tuple(&subdevice_identity) {
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
                let el7031_0030_config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };
                el7031_0030
                    .write()
                    .await
                    .write_config(&subdevice, &el7031_0030_config)
                    .await?;
                (el7031_0030, el7031_0030_config)
            };

            let mode = Winder2Mode::Standby;

            let mut new = Self {
                traverse: StepperDriverEL70x1::new(
                    StepperVelocityEL70x1::new(el7031, EL7031StepperPort::STM1),
                    &el7031_config.stm_features.speed_range,
                ),
                puller: StepperDriverEL70x1::new(
                    StepperVelocityEL70x1::new(el7031_0030.clone(), EL7031_0030StepperPort::STM1),
                    &el7031_0030_config.stm_features.speed_range,
                ),
                spool: StepperDriverEL70x1::new(
                    StepperVelocityEL70x1::new(el7041, EL7041_0052Port::STM1),
                    &el7041_config.stm_features.speed_range,
                ),
                tension_arm: TensionArm::new(AnalogInputGetter::new(AnalogInput::new(
                    el7031_0030,
                    EL7031_0030AnalogInputPort::AI1,
                ))),
                laser: DigitalOutputSetter::new(DigitalOutput::new(el2002, EL2002Port::DO1)),
                namespace: Winder1Namespace::new(),
                mode: mode.clone(),
                spool_step_converter: StepConverter::new(600),
                spool_speed_controller: SpoolSpeedController::new(
                    AngularVelocity::new::<revolution_per_minute>(0.0),
                    AngularVelocity::new::<revolution_per_minute>(600.0),
                    AngularAcceleration::new::<revolution_per_minute_per_second>(200.0),
                    AngularAcceleration::new::<revolution_per_minute_per_second>(-200.0),
                ),
                last_measurement_emit: Instant::now(),
                spool_mode: mode.clone().into(),
                puller_mode: mode.into(),
                puller_speed_controller: PullerSpeedController::new(
                    Acceleration::new::<meter_per_minute_per_second>(10.0),
                    Velocity::new::<meter_per_minute>(1.0),
                    Length::new::<millimeter>(1.75),
                ),
                puller_step_converter: StepConverter::new(600),
            };

            // initalize events
            new.emit_traverse_state();
            new.emit_mode_state();
            new.emit_spool_state();
            new.emit_tension_arm_state();
            new.emit_puller_state();

            Ok(new)
        })
    }
}
