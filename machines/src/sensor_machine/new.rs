use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    devices::ek1100::{EK1100, EK1100_IDENTITY_A},
    devices::el3001::{EL3001, EL3001_IDENTITY_A, EL3001Port},
    io::analog_input::AnalogInput,
};
use smol::{block_on, channel::unbounded};

use crate::{
    get_ethercat_device, validate_no_role_dublicates,
    validate_same_machine_identification_unique, AsyncThreadMessage, MachineMessage,
    MachineNewHardware, MachineNewParams, MachineNewTrait, SENSOR_MACHINE, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

#[derive(Debug)]
pub struct SensorMachine {
    pub api_receiver: smol::channel::Receiver<MachineMessage>,
    pub api_sender: smol::channel::Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<smol::channel::Sender<AsyncThreadMessage>>,

    pub ai1: AnalogInput,

    pub last_print: Instant,
}

impl SensorMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: SENSOR_MACHINE,
    };
}

impl MachineNewTrait for SensorMachine {
    fn new(params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, Error> {
        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::SensorMachine::new] Hardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Role 0 - EK1100 (coupler)
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec())
                    .await?;

            // Role 1 - EL3001 (1-channel analog input, ±10V)
            let el3001 = get_ethercat_device::<EL3001>(
                hardware,
                params,
                1,
                [EL3001_IDENTITY_A].to_vec(),
            )
            .await?
            .0;

            let ai1 = AnalogInput::new(el3001.clone(), EL3001Port::AI1);

            let (sender, receiver) = unbounded();

            tracing::info!(
                "SensorMachine initialized with EK1100 (role 0) and EL3001 (role 1)"
            );

            Ok(Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                ai1,
                last_print: Instant::now(),
            })
        })
    }
}