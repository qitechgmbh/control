use super::api::Winder1Room;
use super::tension_arm::TensionArm;
use super::{WinderV1, WinderV1Mode};
use anyhow::Error;
use control_core::actors::analog_input_getter::{AnalogInputDevice, AnalogInputGetter};
use control_core::actors::digital_output_setter::DigitalOutputSetter;
use control_core::actors::stepper_driver_pulse_train::StepperDriverPulseTrain;
use control_core::identification::MachineDeviceIdentification;
use control_core::machines::new::{
    get_device_by_index, get_mdi_by_role, get_subdevice_by_index, validate_no_role_dublicates,
    validate_same_machine_identification, MachineNewTrait,
};
use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::el2002::{EL2002Port, EL2002};
use ethercat_hal::devices::el2521::{EL2521Configuration, EL2521Port, EL2521};
use ethercat_hal::devices::el2522::{
    EL2522ChannelConfiguration, EL2522Configuration, EL2522Port, EL2522,
};
use ethercat_hal::devices::el3001::{EL3001PdoPreset, EL3001Port, EL3001};
use ethercat_hal::devices::{downcast_device, subdevice_identity_to_tuple, Device};
use ethercat_hal::devices::{
    ek1100::EK1100_IDENTITY_A,
    el2002::EL2002_IDENTITY_A,
    el2521::{EL2521_IDENTITY_0000_A, EL2521_IDENTITY_0000_B, EL2521_IDENTITY_0024_A},
    el2522::EL2522_IDENTITY_A,
    el3001::EL3001_IDENTITY_A,
};
use ethercat_hal::io::analog_input::AnalogInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::pulse_train_output::PulseTrainOutput;
use ethercat_hal::shared_config::el30xx::EL30XXConfiguration;
use ethercat_hal::types::EthercrabSubDevicePreoperational;
use futures::executor::block_on;
use smol::lock::RwLock;
use std::sync::Arc;

impl MachineNewTrait for WinderV1 {
    fn new<'maindevice>(
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &Vec<EthercrabSubDevicePreoperational<'maindevice>>,
        devices: &Vec<Arc<RwLock<dyn Device>>>,
    ) -> Result<Self, Error> {
        // get machine identification unique
        let machine_identification_unique = identified_device_group
            .first()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "[{}::MachineNewTrait/WinderV1::new] No machine identification",
                    module_path!()
                )
            })?
            .machine_identification_unique
            .clone();

        // validate general stuff
        validate_same_machine_identification(identified_device_group)?;
        validate_no_role_dublicates(identified_device_group)?;

        // using block_on because making this funciton async creates a lifetime issue
        // if its async the compiler thinks &subdevices is persisted in the future which might never execute
        // so we can't drop subdevices unless this machine is dropped, which is bad
        block_on(async {
            // Role 0
            // Buscoupler
            // EK1100
            let mdi = get_mdi_by_role(identified_device_group, 0).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/WinderV1::new] No device with role 0",
                module_path!()
            )))?;
            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            match subdevice_identity_to_tuple(&subdevice_identity) {
                EK1100_IDENTITY_A => (),
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WinderV1::new] Device with role 0 is not an EK1100",
                        module_path!()
                    ))
                }
            };

            // Role 1
            // 2x Digitalausgang
            // EL2002
            let mdi = get_mdi_by_role(identified_device_group, 1).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/WinderV1::new] No device with role 1",
                module_path!()
            )))?;
            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let device = get_device_by_index(devices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let el2002 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL2002_IDENTITY_A => downcast_device::<EL2002>(device.clone()).await?,
                _ => Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/WinderV1::new] Device with role 1 is not an EL2002",
                    module_path!()
                ))?,
            };

            // Role 2
            // 1x Analogeingang Lastarm
            let mdi = get_mdi_by_role(identified_device_group, 2).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/WinderV1::new] No device with role 2",
                module_path!()
            )))?;
            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let device = get_device_by_index(devices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let el3001 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL3001_IDENTITY_A => downcast_device::<EL3001>(device.clone()).await?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WinderV1::new] Device with role 2 is not an EL3001",
                        module_path!()
                    ))
                }
            };
            el3001
                .write()
                .await
                .write_config(
                    &subdevice,
                    &EL30XXConfiguration {
                        pdo_assignment: EL3001PdoPreset::Compact,
                        ..Default::default()
                    },
                )
                .await?;

            // Role 3
            // 1x Pulszug Traverse
            let mdi = get_mdi_by_role(identified_device_group, 3).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/WinderV1::new] No device with role 3",
                module_path!()
            )))?;
            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let device = get_device_by_index(devices, mdi.subdevice_index)?;
            let el2521 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL2521_IDENTITY_0000_A | EL2521_IDENTITY_0000_B | EL2521_IDENTITY_0024_A => {
                    downcast_device::<EL2521>(device.clone()).await?
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WinderV1::new] Device with role 3 is not an EL2521",
                        module_path!()
                    ))
                }
            };
            el2521
                .write()
                .await
                .write_config(
                    &subdevice,
                    &EL2521Configuration {
                        direct_input_mode: true,
                        ..EL2521Configuration::default()
                    },
                )
                .await?;

            // Role 4
            // 2x Pulszuf Puller & Winder
            let mdi = get_mdi_by_role(identified_device_group, 4).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/WinderV1::new] No device with role 4",
                module_path!()
            )))?;
            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let device = get_device_by_index(devices, mdi.subdevice_index)?;
            let el2522 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL2522_IDENTITY_A => downcast_device::<EL2522>(device.clone()).await?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WinderV1::new] Device with role 4 is not an EL2522",
                        module_path!()
                    ))
                }
            };
            el2522
                .write()
                .await
                .write_config(
                    &subdevice,
                    &EL2522Configuration {
                        channel1_configuration: EL2522ChannelConfiguration {
                            direct_input_mode: true,
                            ..EL2522ChannelConfiguration::default()
                        },
                        channel2_configuration: EL2522ChannelConfiguration {
                            direct_input_mode: true,
                            ..EL2522ChannelConfiguration::default()
                        },
                        ..EL2522Configuration::default()
                    },
                )
                .await?;

            let mut new = Self {
                winder_driver: StepperDriverPulseTrain::new(PulseTrainOutput::new(
                    el2522.clone(),
                    EL2522Port::PTO1,
                )),
                traverse_driver: StepperDriverPulseTrain::new(PulseTrainOutput::new(
                    el2521,
                    EL2521Port::PTO1,
                )),
                puller_driver: StepperDriverPulseTrain::new(PulseTrainOutput::new(
                    el2522,
                    EL2522Port::PTO2,
                )),
                tension_arm: TensionArm::new(AnalogInputGetter::new(
                    AnalogInput::new(el3001, EL3001Port::AI1),
                    AnalogInputDevice::EL300x.into(),
                )),
                laser_driver: DigitalOutputSetter::new(DigitalOutput::new(el2002, EL2002Port::DO1)),
                room: Winder1Room::new(machine_identification_unique),
                last_measurement_emit: chrono::Utc::now(),
                mode: WinderV1Mode::Standby,
            };

            // initalize events
            new.emit_traverse_state();
            new.emit_mode_state();

            Ok(new)
        })
    }
}
