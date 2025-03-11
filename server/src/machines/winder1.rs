use super::{
    get_device_by_index, get_device_by_role, get_subdevice_by_index, validate_no_role_dublicates,
    validate_same_machine_identification,
};
use crate::ethercat::device_identification::MachineDeviceIdentification;
use anyhow::Error;
use ethercat_hal::actors::analog_input_logger::AnalogInputLogger;
use ethercat_hal::actors::digital_output_setter::DigitalOutputSetter;
use ethercat_hal::actors::stepper_driver_pulse_train::StepperDriverPulseTrain;
use ethercat_hal::actors::Actor;
use ethercat_hal::coe::Configuration;
use ethercat_hal::devices::el2002::{EL2002Port, EL2002};
use ethercat_hal::devices::el2521::{EL2521Configuration, EL2521Port, EL2521};
use ethercat_hal::devices::el2522::{
    EL2522ChannelConfiguration, EL2522Configuration, EL2522Port, EL2522,
};
use ethercat_hal::devices::el3001::{EL3001Port, EL3001};
use ethercat_hal::devices::{
    downcast_device, specifc_device_from_subdevice, subdevice_identity_to_tuple, Device,
};
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
use ethercat_hal::types::EthercrabSubDevicePreoperational;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct WinderV1 {
    // drivers
    traverse_driver: StepperDriverPulseTrain,
    puller_driver: StepperDriverPulseTrain,
    winder_driver: StepperDriverPulseTrain,
    tension_arm_driver: AnalogInputLogger,
    laser_driver: DigitalOutputSetter,
}

impl WinderV1 {
    pub fn new<'maindevice>(
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &Vec<EthercrabSubDevicePreoperational<'maindevice>>,
        devices: &Vec<Option<Arc<RwLock<dyn Device>>>>,
    ) -> Result<Self, Error> {
        // validate general stuff
        validate_same_machine_identification(identified_device_group)?;
        validate_no_role_dublicates(identified_device_group)?;

        let rt = tokio::runtime::Runtime::new()?;
        // using block_on because making this funciton async creates a lifetime issue
        // if its async the compiler thinks &subdevices is persisted in the future which might never execute
        // so we can't drop subdevices unless this machine is dropped, which is bad
        rt.block_on(async {
            // Role 0
            // Buscoupler
            // EK1100
            let (subdevice_index, _) = get_device_by_role(identified_device_group, 0)?;
            let subdevice = get_subdevice_by_index(subdevices, subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            match subdevice_identity_to_tuple(&subdevice_identity) {
                EK1100_IDENTITY_A => (),
                _ => return Err(anyhow::anyhow!("Device with role 0 is not an EK1100")),
            };

            // Role 1
            // 2x Digitalausgang
            // EL2002
            let (subdevice_index, _) = get_device_by_role(identified_device_group, 1)?;
            let subdevice = get_subdevice_by_index(subdevices, subdevice_index)?;
            let device = get_device_by_index(devices, subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let el2002 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL2002_IDENTITY_A => downcast_device::<EL2002>(device.clone()).await?,
                _ => Err(anyhow::anyhow!("Device with role 1 is not an EL2002"))?,
            };

            // Role 2
            // 1x Analogeingang Lastarm
            let (subdevice_index, _) = get_device_by_role(identified_device_group, 2)?;
            let subdevice = get_subdevice_by_index(subdevices, subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let el3001 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL3001_IDENTITY_A => specifc_device_from_subdevice::<EL3001>(subdevice).await?,
                _ => return Err(anyhow::anyhow!("Device with role 2 is not an EL3001")),
            };

            // Role 3
            // 1x Pulszug Traverse
            let (subdevice_index, _) = get_device_by_role(identified_device_group, 3)?;
            let subdevice = get_subdevice_by_index(subdevices, subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let device = get_device_by_index(devices, subdevice_index)?;
            let el2521 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL2521_IDENTITY_0000_A | EL2521_IDENTITY_0000_B | EL2521_IDENTITY_0024_A => {
                    downcast_device::<EL2521>(device.clone()).await?
                }
                _ => return Err(anyhow::anyhow!("Device with role 3 is not an EL2521")),
            };
            let el2521_configuration = EL2521Configuration {
                direct_input_mode: true,
                ..EL2521Configuration::default()
            };
            el2521_configuration.write_config(&subdevice).await?;
            el2521
                .write()
                .await
                .set_configuration(&el2521_configuration);

            // Role 4
            // 2x Pulszuf Puller & Winder
            let (subdevice_index, _) = get_device_by_role(identified_device_group, 4)?;
            let subdevice = get_subdevice_by_index(subdevices, subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let device = get_device_by_index(devices, subdevice_index)?;
            let el2522 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL2522_IDENTITY_A => downcast_device::<EL2522>(device.clone()).await?,
                _ => return Err(anyhow::anyhow!("Device with role 4 is not an EL2522")),
            };
            let el2522_configuration = EL2522Configuration {
                channel1_configuration: EL2522ChannelConfiguration {
                    direct_input_mode: true,
                    ..EL2522ChannelConfiguration::default()
                },
                channel2_configuration: EL2522ChannelConfiguration {
                    direct_input_mode: true,
                    ..EL2522ChannelConfiguration::default()
                },
                ..EL2522Configuration::default()
            };
            el2522_configuration.write_config(&subdevice).await?;
            el2522
                .write()
                .await
                .set_configuration(&el2522_configuration);

            Ok(Self {
                traverse_driver: StepperDriverPulseTrain::new(PulseTrainOutput::new(
                    el2521,
                    EL2521Port::PTO1,
                )),
                puller_driver: StepperDriverPulseTrain::new(PulseTrainOutput::new(
                    el2522.clone(),
                    EL2522Port::PTO1,
                )),
                winder_driver: StepperDriverPulseTrain::new(PulseTrainOutput::new(
                    el2522,
                    EL2522Port::PTO2,
                )),
                tension_arm_driver: AnalogInputLogger::new(AnalogInput::new(
                    el3001,
                    EL3001Port::AI1,
                )),
                laser_driver: DigitalOutputSetter::new(DigitalOutput::new(el2002, EL2002Port::DO1)),
            })
        })
    }
}

impl Actor for WinderV1 {
    fn act(
        &mut self,
        now_ts: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.traverse_driver.act(now_ts).await;
            self.puller_driver.act(now_ts).await;
            self.winder_driver.act(now_ts).await;
            self.tension_arm_driver.act(now_ts).await;
            self.laser_driver.act(now_ts).await;
        })
    }
}
