use std::sync::Arc;
use std::time::Instant;

use anyhow::Error;
use ethercat_hal::devices::wago_modules::ip20_ec_di8_do8::{
    IP20_EC_DI8_DO8_IDENTITY, IP20EcDi8Do8, IP20EcDi8Do8InputPort, IP20EcDi8Do8OutputPort,
};
use ethercat_hal::devices::wago_modules::wago_750_671::Wago750_671;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::stepper_velocity_wago_750_671::StepperVelocityWago750671;
use smol::{block_on, lock::RwLock};

use super::{BottlecapsTestMachine, api::BottlecapsTestMachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};
use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
use ethercat_hal::devices::{EthercatDevice, downcast_device};
use ethercat_hal::io::digital_output::DigitalOutput;

impl MachineNewTrait for BottlecapsTestMachine {
    fn new(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params
            .device_group
            .iter()
            .map(|d| d.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/BottlecapsTestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            let (coupler_dev, coupler_subdev) = get_ethercat_device::<Wago750_354>(
                hardware,
                params,
                0,
                [WAGO_750_354_IDENTITY_A].to_vec(),
            )
            .await?;

            let modules = Wago750_354::initialize_modules(coupler_subdev).await?;
            let mut coupler = coupler_dev.write().await;
            for module in modules {
                coupler.set_module(module);
            }
            coupler.init_slot_modules(coupler_subdev);

            // Get the device at slot index 0 (first expansion module).
            let dev = coupler
                .slot_devices
                .get(0)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::BottlecapsTestMachine::new] slot 0 not configured",
                        module_path!()
                    )
                })?
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::BottlecapsTestMachine::new] slot 0 is empty",
                        module_path!()
                    )
                })?;

            let ip20_device = get_ethercat_device::<IP20EcDi8Do8>(
                hardware,
                params,
                1,
                [IP20_EC_DI8_DO8_IDENTITY].to_vec(),
            )
            .await?
            .0;

            // Create digital outputs
            let do1 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO1);
            let do2 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO2);
            let do3 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO3);
            let do4 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO4);
            let do5 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO5);
            let do6 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO6);
            let do7 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO7);
            let do8 = DigitalOutput::new(ip20_device.clone(), IP20EcDi8Do8OutputPort::DO8);

            // Create digital inputs
            let di1 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI1);
            let di2 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI2);
            let di3 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI3);
            let di4 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI4);
            let di5 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI5);
            let di6 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI6);
            let di7 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI7);
            let di8 = DigitalInput::new(ip20_device.clone(), IP20EcDi8Do8InputPort::DI8);

            let wago_750_671: Arc<RwLock<Wago750_671>> =
                downcast_device::<Wago750_671>(dev).await?;
            drop(coupler);

            let stepper = StepperVelocityWago750671::new(wago_750_671);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: BottlecapsTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),

                outputs: [false; 8],
                inputs: [false; 8],
                override_inputs: [false; 8],
                douts: [do1, do2, do3, do4, do5, do6, do7, do8],
                dins: [di1, di2, di3, di4, di5, di6, di7, di8],
                stepper,
            };

            // Emit initial state so subscribers get values immediately.
            machine.emit_state();
            Ok(machine)
        })
    }
}
