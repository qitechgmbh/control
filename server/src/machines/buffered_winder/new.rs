use std::time::Instant;

use anyhow::Error;
use control_core::actors::stepper_driver_el70x1::StepperDriverEL70x1;
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
use ethercat_hal::devices::el2002::{EL2002, EL2002Port};
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
use uom::si::f64::{Length, Velocity};
use uom::si::length::{centimeter, millimeter};

use super::{
    api::{BufferedWinderNamespace, Mode}, BufferedWinder
};

impl MachineNewTrait for BufferedWinder {
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
                    "[{}::MachineNewTrait/BufferedWinder::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        smol::block_on(async {
            //TODO adjust

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
                            "[{}::MachineNewTrait/BufferedWinder::new] Device with role 0 is not Ethercat",
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
                            "[{}::MachineNewTrait/BufferedWinder::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
            }

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
                        _ => Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/BufferedWinder::new] Device with role 2 is not Ethercat",
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
                        "[{}::MachineNewTrait/BufferedWinder::new] Device with role 2 is not an EL7041-0052",
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
            let (el7031, el7031_config) = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 3)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                        _ => Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/BufferedWinder::new] Device with role 3 is not Ethercat",
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
                        "[{}::MachineNewTrait/BufferedWinder::new] Device with role 3 is not an EL7031",
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

            let buffered_winder: BufferedWinder = Self {
                    namespace: BufferedWinderNamespace::new(params.socket_queue_tx.clone()),
                    last_measurement_emit: Instant::now(),
                    mode: Mode::Standby,
            };

            Ok(buffered_winder)
            })

    }
}
