use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    devices::{
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el1124::{EL1124, EL1124_IDENTITY_A, EL1124Port},
    },
    io::{
        digital_input::DigitalInput,
        ufm_flow_input::{Ufm02Type, UfmFlowInput},
    },
};
use smol::channel::unbounded;

use super::{UfmFlowTestMachine, api::UfmFlowTestMachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

impl MachineNewTrait for UfmFlowTestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.iter().cloned().collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::UfmFlowTestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        smol::block_on(async {
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, vec![EK1100_IDENTITY_A]).await?;
            let el1124 =
                get_ethercat_device::<EL1124>(hardware, params, 1, vec![EL1124_IDENTITY_A])
                    .await?
                    .0;

            let pulse_input = DigitalInput::new(el1124.clone(), EL1124Port::DI1);
            let error_input = DigitalInput::new(el1124, EL1124Port::DI2);
            let flow_input = UfmFlowInput::new(pulse_input, error_input, Ufm02Type::default());

            let (sender, receiver) = unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: UfmFlowTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                flow_lph: 0.0,
                total_volume_m3: 0.0,
                sensor_error: false,
                main_sender: params.main_thread_channel.clone(),
                flow_input,
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
