use bitvec::{order::Lsb0, slice::BitSlice};
use machine_implementations::QiTechMachine;
use qitech_lib::{
    ethercat_hal::{
        Consumer, MetaSubdevice, Producer, controller::{EtherCATAppHandle}, devices::EthercatDevice
    },
    machines::MachineDataRegistry,
};
use std::{cell::RefCell, rc::Rc};

pub fn write_ecat_inputs<C : Consumer,P: Producer>(
    ecat: &mut EtherCATAppHandle<C,P>,
    subdevices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
) {
    let inputs = ecat.get_inputs(); 

    for i in 0..subdevices.len() {
        let meta_dev = subdevices[i].0;
        let subdevice = subdevices.get(i).unwrap();
        let input_slice = &inputs[meta_dev.start_tx..meta_dev.end_tx];
        let input_bits_slice = BitSlice::<u8, Lsb0>::from_slice(input_slice);        
        {
            let mut subdevice = subdevice.1.borrow_mut();
            let _res = subdevice.input(input_bits_slice);
            let _res = subdevice.input_post_process();
        }
    }    
}

pub fn write_ecat_outputs<C : Consumer,P: Producer>(
    ecat: &mut EtherCATAppHandle<C,P>,
    subdevices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
) {    
    let outputs = ecat.write_outputs();    
    for i in 0..subdevices.len() {
        let meta_dev = subdevices[i].0;
        let subdevice = subdevices.get(i).unwrap();
        let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
        let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);        
        {
            let mut subdevice = subdevice.1.borrow_mut();
            let _res = subdevice.output(output_bits);
            let _res = subdevice.output_pre_process();
        }
    }
    ecat.send_outputs();
}

pub fn run_machines(machines: &mut Vec<Box<dyn QiTechMachine>>, reg: &mut MachineDataRegistry) {
    let machine_count = machines.len();
    for i in 0..machine_count {
        let machine = machines
            .get_mut(i)
            .expect("Machine should NEVER be NONE here (run_machines)!!");
        machine.act(Some(reg));
    }

    for machine in machines {
        machine.react(reg);
    }
}
