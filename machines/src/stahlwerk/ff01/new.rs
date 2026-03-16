use std::time::Instant;

use crate::serial::devices::xtrem_zebra::XtremSerial;
use super::api::FF01Namespace;
use super::{SignalLight, FF01};
use crate::{
    MachineNewHardware, MachineNewHardwareEthercat, MachineNewTrait, get_ethercat_device,
    validate_same_machine_identification_unique,
    validate_no_role_duplicates
};

use anyhow::Error;
use ethercat_hal::devices::ek1100::{EK1100, EK1100_IDENTITY_A};
use ethercat_hal::devices::el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port};
use ethercat_hal::io::digital_output::DigitalOutput;
use stahlwerk_extension::ClientConfig;
use stahlwerk_extension::ff01::ProxyClient;

use super::api::Configuration;

impl MachineNewTrait for FF01 {
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
        // This creates the "driver" for the serial connection to the scales.
        let hardware_serial = XtremSerial::new_serial();
        let (_device_id, xtrem_serial) = hardware_serial?;

        let Ok(config) = ClientConfig::from_file("") else {
            //TODO: log proper error
            return Err(Error::msg("Could not find config"));
        };

        let Ok(proxy_client) = ProxyClient::new(config) else {
            //TODO: log proper error
            return Err(Error::msg("Failed initialize client"));
        };

        smol::block_on(async {
            // Role 0: Buscoupler EK1100
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec());

            let el2004 =
                get_ethercat_device::<EL2004>(hardware, params, 1, [EL2004_IDENTITY_A].to_vec())
                    .await?
                    .0;

            let digital_out_1 = DigitalOutput::new(el2004.clone(), EL2004Port::DO1); // Green Light
            let digital_out_2 = DigitalOutput::new(el2004.clone(), EL2004Port::DO2); // Yellow Light
            let digital_out_3 = DigitalOutput::new(el2004.clone(), EL2004Port::DO3); // Red Light
            let digital_out_4 = DigitalOutput::new(el2004.clone(), EL2004Port::DO4); // Beep Sound

            let (sender, receiver) = smol::channel::unbounded();

            let xtrem_zebra_machine = Self {
                main_sender: params.main_thread_channel.clone(),
                xtrem_serial,
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: FF01Namespace {
                    namespace: params.namespace.clone(),
                },
                last_measurement_emit: Instant::now(),
                _emitted_default_state: false,
                total_weight: 0.0,
                current_weight: 0.0,
                last_weight: 0.0,
                cycle_max_weight: 0.0,
                in_accumulation: false,
                plate_counter: 0,
                tolerance: 0.3,
                tare_weight: 0.0,
                last_raw_weight: 0.0,
                signal_light: SignalLight {
                    green_light: digital_out_1,
                    green_light_on_since: None,
                    yellow_light: digital_out_2,
                    _yellow_light_on_since: None,
                    red_light: digital_out_3,
                    red_light_on_since: None,
                    _beeper: digital_out_4,
                },
                entry: None,
                client: proxy_client,
                last_request_ts: Instant::now(),
            };
            Ok(xtrem_zebra_machine)
        })
    }
}
