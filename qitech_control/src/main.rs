use machine_implementations::minimal_machines::digital_input_test_machine::DigitalInputTestMachine;
use qitech_lib::ethercat_hal::{EtherCATThreadChannel, devices::{device_from_subdevice_identity, downcast_subdevice, el2004::EL2004}, start_ethercat_thread};

fn main() {
    let res = start_ethercat_thread("eth0");
    let result = res.0;
    let ecat_controller = result.0;
    let ecat_channel: EtherCATThreadChannel = result.1;

    let dev = device_from_subdevice_identity(ecat_controller.subdevices[0]).unwrap();
    let el2004 : EL2004 = downcast_subdevice::<EL2004>(dev).unwrap();

    //let di_machine : DigitalInputTestMachine = DigitalInputTestMachine::new(,ecat_channel).unwrap();
}
