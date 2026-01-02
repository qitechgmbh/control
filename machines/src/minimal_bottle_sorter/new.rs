use crate::minimal_bottle_sorter::MinimalBottleSorter;
use crate::minimal_bottle_sorter::api::MinimalBottleSorterNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::ek1100::{EK1100, EK1100_IDENTITY_A};
use ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
use ethercat_hal::devices::el7041_0052::pdo::EL7041_0052PredefinedPdoAssignment;
use ethercat_hal::devices::el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port};
use ethercat_hal::devices::wago_modules::ip20_ec_di8_do8::{
    IP20_EC_DI8_DO8_IDENTITY, IP20EcDi8Do8, IP20EcDi8Do8OutputPort,
};
use ethercat_hal::devices::EthercatDeviceUsed;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};

impl MachineNewTrait for MinimalBottleSorter {
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
                    "[{}::MachineNewTrait/MinimalBottleSorter::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Role 0: EK1100 EtherCAT Coupler
            get_ethercat_device::<EK1100>(
                hardware,
                params,
                0,
                vec![EK1100_IDENTITY_A],
            )
            .await?;

            // Role 1: EL7041-0052 Stepper Motor Terminal
            let (el7041, subdevice) = get_ethercat_device::<EL7041_0052>(
                hardware,
                params,
                1,
                vec![EL7041_0052_IDENTITY_A],
            )
            .await?;

            let el7041_config = EL7041_0052Configuration {
                stm_features: ethercat_hal::shared_config::el70x1::StmFeatures {
                    operation_mode: EL70x1OperationMode::DirectVelocity,
                    speed_range: ethercat_hal::shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                    ..Default::default()
                },
                stm_motor: StmMotorConfiguration {
                    max_current: 1500, // 1.5A
                    ..Default::default()
                },
                pdo_assignment: EL7041_0052PredefinedPdoAssignment::VelocityControlCompact,
                ..Default::default()
            };

            el7041
                .write()
                .await
                .write_config(&subdevice, &el7041_config)
                .await?;
            {
                let mut device_guard = el7041.write().await;
                device_guard.set_used(true);
            }

            let stepper = StepperVelocityEL70x1::new(el7041.clone(), EL7041_0052Port::STM1);

            // Role 2: IP20 DI8/DO8 Module
            let ip20_device = get_ethercat_device::<IP20EcDi8Do8>(
                hardware,
                params,
                2,
                [IP20_EC_DI8_DO8_IDENTITY].to_vec(),
            )
            .await?
            .0;

            {
                let mut device_guard = ip20_device.write().await;
                device_guard.set_used(true);
            }

            // Create digital outputs
            let do1 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO1);
            let do2 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO2);
            let do3 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO3);
            let do4 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO4);
            let do5 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO5);
            let do6 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO6);
            let do7 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO7);
            let do8 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO8);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: MinimalBottleSorterNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                last_live_values_emit: Instant::now(),
                stepper_enabled: false,
                stepper_speed: 0.0,
                stepper_direction: true,
                outputs: [false; 8],
                pulse_remaining: [0; 8],
                main_sender: params.main_thread_channel.clone(),
                stepper,
                douts: [do1, do2, do3, do4, do5, do6, do7, do8],
            };

            machine.emit_state();
            machine.emit_live_values();

            Ok(machine)
        })
    }
}
