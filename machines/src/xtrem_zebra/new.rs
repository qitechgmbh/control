use std::time::Instant;

use crate::serial::devices::xtrem_zebra::XtremSerial;
use crate::xtrem_zebra::api::XtremZebraNamespace;
use crate::xtrem_zebra::{SignalLight, XtremZebra};
use crate::{
    MachineNewHardware, MachineNewHardwareEthercat, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::ek1100::{EK1100, EK1100_IDENTITY_A};
use ethercat_hal::devices::el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port};
use ethercat_hal::io::digital_output::DigitalOutput;

impl MachineNewTrait for XtremZebra {
    fn new<'maindevice>(params: &crate::MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.to_vec();

        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

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
                namespace: XtremZebraNamespace {
                    namespace: params.namespace.clone(),
                },
                last_measurement_emit: Instant::now(),
                emitted_default_state: false,
                total_weight: 0.0,
                current_weight: 0.0,
                last_weight: 0.0,
                cycle_max_weight: 0.0,
                in_accumulation: false,
                plate1_target: 10.0,
                plate2_target: 30.0,
                plate3_target: 50.0,
                plate1_counter: 0,
                plate2_counter: 0,
                plate3_counter: 0,
                tolerance: 0.3,
                tare_weight: 0.0,
                last_raw_weight: 0.0,
                signal_light: SignalLight {
                    green_light: digital_out_1,
                    green_light_on_since: None,
                    yellow_light: digital_out_2,
                    yellow_light_on_since: None,
                    red_light: digital_out_3,
                    red_light_on_since: None,
                    beeper: digital_out_4,
                },
            };
            Ok(xtrem_zebra_machine)
        })
    }
}
