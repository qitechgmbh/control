use anyhow::Error;

use ethercat_hal::devices::ek1100::{EK1100, EK1100_IDENTITY_A};
use ethercat_hal::devices::el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port};
use ethercat_hal::io::digital_output::DigitalOutput;

use crate::MachineNewParams;
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

impl MachineNewTrait for FF01 
{
    fn new<'maindevice>(params: &crate::MachineNewParams) -> Result<Self, Error> {
        validate_params(params)?;

        let hardware = get_hardware(params)?;

        let channel = create_channel(params);
        let scales  = create_scales()?;
        let lights  = create_lights(hardware, params)?;

        let instance = Self::new(channel, scales, lights);
        Ok(instance)
    }
}

// helpers
fn validate_params(
    params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>
) -> anyhow::Result<()> {
    let device_identification = params.device_group.to_vec();
    validate_same_machine_identification_unique(&device_identification)?;
    validate_no_role_duplicates(&device_identification)?;
    Ok(())
}

fn get_hardware<'a, 'b, 'c, 'd, 'e>(
    params: &MachineNewParams<'a, 'b, '_, 'c, '_, '_, 'd>
) -> anyhow::Result<&'e &'d MachineNewHardwareEthercat<'a, 'b, 'c>> {
    match &params.hardware {
        MachineNewHardware::Ethercat(v) => Ok(v),
        _ => {
            return Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/XtremZebra::new] MachineNewHardware is not Ethercat",
                module_path!()
            ));
        }
    }
}

fn create_channel(
    params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>
) -> MachineChannel {
    let machine_uid = params.get_machine_identification_unique();
    let main_sender = params.main_thread_channel.clone();
    let namespace = params.namespace.clone();

    MachineChannel::new_full(machine_uid, main_sender, namespace)
}

fn create_scales() -> anyhow::Result<Scales> {
    let (_, serial_interface) = XtremSerial::new_serial()?;
    let instance = Scales::new(serial_interface);
    Ok(instance)
}

fn create_lights(
    hardware: &&MachineNewHardwareEthercat<'_, '_, '_>, 
    params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>
) -> anyhow::Result<SignalLights> {
    let el2004 = smol::block_on(async {
        let _ek1100 =
            get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec());

        let el2004 =
            get_ethercat_device::<EL2004>(hardware, params, 1, [EL2004_IDENTITY_A].to_vec())
                .await?
                .0;

        Ok::<_, anyhow::Error>(el2004)
    })?;

    let instance = SignalLights::new(
        DigitalOutput::new(el2004.clone(), EL2004Port::DO1), 
        DigitalOutput::new(el2004.clone(), EL2004Port::DO2), 
        DigitalOutput::new(el2004.clone(), EL2004Port::DO3), 
        DigitalOutput::new(el2004.clone(), EL2004Port::DO4)
    );

    Ok(instance)
}