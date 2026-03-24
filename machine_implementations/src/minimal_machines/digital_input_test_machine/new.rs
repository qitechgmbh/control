use super::DigitalInputTestMachine;
use qitech_lib::{
    ethercat_hal::{
        EtherCATThreadChannel,
        devices::{EthercatDevice, wago_modules::wago_750_402::Wago750_402},
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};

pub fn downcast_device<T: EthercatDevice>(
    dev: T,
    identity: (u32, u32, u32),
) -> Result<T, anyhow::Error> {
    match identity {
        (0, 0, 0) => Ok(dev),
        _ => Err(anyhow::anyhow!("error")),
    }
}

impl DigitalInputTestMachine {
    pub fn new(
        hw: &Vec<Box<dyn EthercatDevice>>,
        _eth_channel: EtherCATThreadChannel,
    ) -> Result<DigitalInputTestMachine, anyhow::Error> {
        let dev = hw.get(1).unwrap();
        let downcasted_ref: &Wago750_402 = dev.as_any().downcast_ref::<Wago750_402>().unwrap();
        let res = downcast_device::<Wago750_402>(downcasted_ref.clone(), (0u32, 0u32, 0u32))?;
        let my_test = Self {
            machine_identification_unique: MachineIdentificationUnique {
                machine_ident: MachineIdentification {
                    vendor: 0,
                    machine: 67,
                },
                serial: 420,
            },
            led_on: [false; 4],
            digital_input_device: Box::new(res),
        };
        Ok(my_test)
    }
}
