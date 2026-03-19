use std::time::{Duration};

use anyhow::{Error, anyhow};

use stahlwerk_extension::ClientConfig;
use stahlwerk_extension::ff01::ProxyClient;

use ethercat_hal::devices::ek1100::{EK1100, EK1100_IDENTITY_A};
use ethercat_hal::devices::el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port};
use ethercat_hal::io::digital_output::DigitalOutput;

use crate::{
    MachineChannel, 
    MachineNewHardware, 
    MachineNewHardwareEthercat, 
    MachineNewTrait, 
    get_ethercat_device, 
    validate_no_role_duplicates, 
    validate_same_machine_identification_unique,
    serial::devices::xtrem_zebra::XtremSerial,
};

use super::FF01;
use super::devices::{Scales, SignalLights};
use super::services::WorkorderService;

impl MachineNewTrait for FF01 
{
    fn new<'maindevice>(params: &crate::MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;
        let hardware: &&MachineNewHardwareEthercat<'_, '_, '_> = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/XtremZebra::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        // create channel
        let machine_uid = params.get_machine_identification_unique();
        let channel = MachineChannel::new_full(machine_uid, params.main_thread_channel.clone(), params.namespace.clone());

        tracing::error!("Created channel");

        // create scale
        let (_, serial_interface) = XtremSerial::new_serial()?;
        let scale = Scales::new(serial_interface);

        tracing::error!("Created scale");

        // create lights
        let el2004 = smol::block_on(async {
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec());

            let el2004 =
                get_ethercat_device::<EL2004>(hardware, params, 1, [EL2004_IDENTITY_A].to_vec())
                    .await?
                    .0;

            Ok::<_, anyhow::Error>(el2004)
        })?;

        let lights = SignalLights::new(
            DigitalOutput::new(el2004.clone(), EL2004Port::DO1), 
            DigitalOutput::new(el2004.clone(), EL2004Port::DO2), 
            DigitalOutput::new(el2004.clone(), EL2004Port::DO3), 
            DigitalOutput::new(el2004.clone(), EL2004Port::DO4)
        );

        tracing::error!("Created lights");

        // create service
        let config_path = "/home/qitech/config.json";
        let config = ClientConfig::from_file(config_path).map_err(|e| anyhow!("{:?}", e))?;
        let client = ProxyClient::new(config).map_err(|e| anyhow!("{:?}", e))?;
        let service = WorkorderService::new(client, Duration::from_millis(1000));

        tracing::error!("Created service");

        // create instance
        let instance = Self::new(scale, lights, service, channel);
        Ok(instance)
    }
}