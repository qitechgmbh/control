use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    devices::{
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el1124::{EL1124, EL1124_IDENTITY_A, EL1124Port},
        el9505::{EL9505, EL9505_IDENTITY_A},
    },
    io::{
        digital_input::DigitalInput,
        ufm_flow_input::{Ufm02Type, UfmFlowData, UfmFlowInput},
    },
};
use smol::block_on;

use super::{UfmFlowInputMachine, api::UfmFlowInputMachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

impl MachineNewTrait for UfmFlowInputMachine {
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
                    "[{}::MachineNewTrait/UfmFlowInputMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Role 0 — EK1100 bus coupler
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec())
                    .await?
                    .0;

            // Role 1 — EL9505 5V power supply terminal
            let _el9505 =
                get_ethercat_device::<EL9505>(hardware, params, 1, [EL9505_IDENTITY_A].to_vec())
                    .await?
                    .0;

            // Role 2 — EL1124 4-channel 5V digital input
            let el1124 =
                get_ethercat_device::<EL1124>(hardware, params, 2, [EL1124_IDENTITY_A].to_vec())
                    .await?
                    .0;

            // IO0 → DI1 (pulse), IO1 → DI2 (error)
            let pulse_input = DigitalInput::new(el1124.clone(), EL1124Port::DI1);
            let error_input = DigitalInput::new(el1124.clone(), EL1124Port::DI2);

            let flow_sensor = UfmFlowInput::new(pulse_input, error_input, Ufm02Type::Ufm02_05);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: UfmFlowInputMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                flow_sensor,
                last_data: UfmFlowData {
                    flow_lph: 0.0,
                    total_volume_m3: 0.0,
                    error: false,
                    total_pulses: 0,
                },
            };

            machine.emit_state();
            Ok(machine)
        })
    }
}
