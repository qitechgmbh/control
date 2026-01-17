use crate::ip20_test_machine::IP20TestMachine;
use crate::ip20_test_machine::api::IP20TestMachineNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::wago_modules::ip20_ec_di8_do8::{
    IP20_EC_DI8_DO8_IDENTITY, IP20EcDi8Do8, IP20EcDi8Do8InputPort, IP20EcDi8Do8OutputPort,
};
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::digital_output::DigitalOutput;

impl MachineNewTrait for IP20TestMachine {
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
                    "[{}::EtherCATMachine/IP20TestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            let ip20_device = get_ethercat_device::<IP20EcDi8Do8>(
                hardware,
                params,
                0,
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

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: IP20TestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                last_live_values_emit: Instant::now(),
                outputs: [false; 8],
                inputs: [false; 8],
                main_sender: params.main_thread_channel.clone(),
                douts: [do1, do2, do3, do4, do5, do6, do7, do8],
                dins: [di1, di2, di3, di4, di5, di6, di7, di8],
            };

            machine.emit_state();
            machine.emit_live_values();

            Ok(machine)
        })
    }
}
