use bitvec::{order::Lsb0, slice::BitSlice};
use machine_implementations::QiTechMachine;
use qitech_lib::{
    ethercat_hal::{Consumer, EtherCATAppHandle, MetaSubdevice, Producer, devices::EthercatDevice},
    machines::MachineDataRegistry,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

pub fn write_ecat_inputs<C: Consumer, P: Producer>(
    ecat: &mut EtherCATAppHandle<C, P>,
    subdevices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
) {
    loop {
        match ecat.get_inputs() {
            Some(inputs) => {
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
                ecat.input_consumer.finish_read();
                break;
            },
            None => (),
        };
    }
}

pub fn write_ecat_outputs<C: Consumer, P: Producer>(
    ecat: &mut EtherCATAppHandle<C, P>,
    subdevices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
) {
    match ecat.write_outputs() {
        Some(outputs) => {
            for i in 0..subdevices.len() {
                let meta_dev = subdevices[i].0;
                let subdevice = subdevices.get(i).unwrap();
                let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
                let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);
                {
                    let mut subdevice = subdevice.1.borrow_mut();
                    let _res = subdevice.output_pre_process();
                    let _res = subdevice.output(output_bits);
                }
            }
            ecat.send_outputs();
        }
        None => {
            // Do nothing
        }
    }
}

pub fn run_machines(
    machines: &mut Vec<Box<dyn QiTechMachine>>,
    reg: &mut MachineDataRegistry,
) -> Option<usize> {
    let machine_count = machines.len();
    let mut machine_errored_i = None;
    for i in 0..machine_count {
        let machine = machines
            .get_mut(i)
            .expect("Machine should NEVER be NONE here (run_machines)!!");
        let res = machine.act(Some(reg));
        match res {
            Ok(_) => (),
            Err(e) => match e {
                qitech_lib::machines::MachineError::RecoverableFailure(e) => {
                    println!(
                        "machine {:?} had a RecoverableFailure: {:?}",
                        machine.get_identification(),
                        e
                    );
                    machine_errored_i = Some(i);
                }
                qitech_lib::machines::MachineError::IrrecoverableFailure(e) => {
                    println!(
                        "removing machine {:?} it had an IrrecoverableFailure {:?}",
                        machine.get_identification(),
                        e
                    );
                    machine_errored_i = Some(i);
                }
            },
        }
    }
    machine_errored_i
}
