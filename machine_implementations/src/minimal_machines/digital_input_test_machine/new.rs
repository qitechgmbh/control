use super::DigitalInputTestMachine;
use qitech_lib::{
    ethercat_hal::{
        EtherCATThreadChannel,
        devices::{EthercatDevice, downcast_subdevice, el1008::{self, EL1008}},
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};

impl DigitalInputTestMachine {
    pub fn new(
        hw: &Vec<Box<dyn EthercatDevice>>,
        _eth_channel: EtherCATThreadChannel,
    ) -> Result<DigitalInputTestMachine, anyhow::Error> {

        let el1008 : EL1008 = downcast_subdevice::<EL1008>(*hw.get(1).unwrap())?;
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
