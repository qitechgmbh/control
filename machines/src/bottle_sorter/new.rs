use crate::bottle_sorter::api::BottleSorterNamespace;
use crate::bottle_sorter::BottleSorter;
use smol::block_on;
use std::time::Instant;

use crate::{
    get_ethercat_device, validate_no_role_dublicates,
    validate_same_machine_identification_unique, MachineNewHardware, MachineNewParams,
    MachineNewTrait,
};

use anyhow::Error;
use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::ek1100::{EK1100, EK1100_IDENTITY_A};
use ethercat_hal::devices::el7041_0052::{
    coe::EL7041_0052Configuration, EL7041_0052, EL7041_0052Port, EL7041_0052_IDENTITY_A,
};
use ethercat_hal::devices::wago_modules::ip20_ec_di8_do8::{
    IP20EcDi8Do8, IP20EcDi8Do8OutputPort, IP20_EC_DI8_DO8_IDENTITY,
};
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};

impl MachineNewTrait for BottleSorter {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // Validate general stuff
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
                    "[{}::EtherCATMachine/BottleSorter::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Get EK1100 bus coupler (role 0)
            let _ek1100_device = get_ethercat_device::<EK1100>(
                hardware,
                params,
                0,
                vec![EK1100_IDENTITY_A],
            )
            .await?
            .0;

            // Get EL7041_0052 stepper terminal (role 1)
            let el7041_device = get_ethercat_device::<EL7041_0052>(
                hardware,
                params,
                1,
                vec![EL7041_0052_IDENTITY_A],
            )
            .await?;

            // Configure the stepper motor for velocity mode
            let el7041_config = EL7041_0052Configuration {
                stm_features: ethercat_hal::shared_config::el70x1::StmFeatures {
                    operation_mode: EL70x1OperationMode::DirectVelocity,
                    ..Default::default()
                },
                stm_motor: StmMotorConfiguration {
                    max_current: 2800,
                    ..Default::default()
                },
                ..Default::default()
            };

            el7041_device
                .0
                .write()
                .await
                .write_config(&el7041_device.1, &el7041_config)
                .await?;

            let el7041_device = el7041_device.0;

            // Get IP20 DI8/DO8 module (role 2)
            let ip20_device = get_ethercat_device::<IP20EcDi8Do8>(
                hardware,
                params,
                2,
                vec![IP20_EC_DI8_DO8_IDENTITY],
            )
            .await?
            .0;

            // Create stepper motor controller
            let stepper = StepperVelocityEL70x1::new(el7041_device.clone(), EL7041_0052Port::STM1);

            // Create digital outputs for IP20 module
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
                namespace: BottleSorterNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                last_live_values_emit: Instant::now(),
                outputs: [false; 8],
                stepper_speed_mm_s: 0.0,
                stepper_enabled: false,
                main_sender: params.main_thread_channel.clone(),
                douts: [do1, do2, do3, do4, do5, do6, do7, do8],
                stepper,
                steps_per_mm: 200.0, // Default: 200 steps per mm (configurable)
                pulse_outputs: [None; 8],
                pulse_duration_ms: 100, // 100ms pulse duration
            };

            machine.emit_state();
            machine.emit_live_values();

            Ok(machine)
        })
    }
}
