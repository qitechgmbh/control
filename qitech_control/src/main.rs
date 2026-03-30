use app_state::SharedAppState;
use bitvec::{order::Lsb0, slice::BitSlice};
use machine_implementations::MachineApi;
use machine_implementations::minimal_machines::digital_input_test_machine::DigitalInputTestMachine;
use qitech_lib::{
    ethercat_hal::{
        controller::{EtherCATAppHandle, EtherCATController},
        devices::{EthercatDevice, device_from_subdevice_identity_rc},
        start_ethercat_thread,
    },
    machines::{Machine, MachineDataRegistry},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::runtime::Runtime;

pub mod apis;
mod app_state;

fn write_ecat_inputs(
    ecat: &mut EtherCATAppHandle,
    ecat_controller: Arc<EtherCATController>,
    subdevices: Vec<Rc<RefCell<dyn EthercatDevice>>>,
) {
    assert!(ecat_controller.subdevice_count == subdevices.len());
    let inputs = ecat.get_inputs();
    for i in 0..ecat_controller.subdevice_count {
        let meta_dev = ecat_controller.subdevices[i];
        let subdevice = subdevices.get(i).unwrap();
        let input_slice = &inputs[meta_dev.start_tx..meta_dev.end_tx];
        let input_bits_slice = BitSlice::<u8, Lsb0>::from_slice(input_slice);
        {
            let mut subdevice = subdevice.borrow_mut();
            let _res = subdevice.input(input_bits_slice);
        }
    }
}

fn write_ecat_outputs(
    ecat: &mut EtherCATAppHandle,
    ecat_controller: Arc<EtherCATController>,
    subdevices: Vec<Rc<RefCell<dyn EthercatDevice>>>,
) {
    assert!(ecat_controller.subdevice_count == subdevices.len());
    let outputs = ecat.write_outputs();
    for i in 0..ecat_controller.subdevice_count {
        let meta_dev = ecat_controller.subdevices[i];
        let subdevice = subdevices.get(i).unwrap();
        let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
        let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);
        let subdevice = subdevice.borrow();
        let _res = subdevice.output(output_bits);
    }
    ecat.send_outputs();
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_async_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio Runtime")
    })
}

fn main() {
    let rt = get_async_runtime();
    let state = Arc::new(SharedAppState::new());
    let _api = rt.spawn(apis::init_api(state.clone()));
    let eth_control = start_ethercat_thread("enp101s0f4u1u2");
    
    let mut ecat_handle = eth_control.app_handle;
    let ecat_channel = eth_control.channel;
    let ecat_controller = eth_control.controller;

    let _res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);
    std::thread::sleep(Duration::from_millis(1000));

    let _res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);
    std::thread::sleep(Duration::from_millis(1000));

    let mut subdevices: Vec<Rc<RefCell<dyn EthercatDevice>>> = vec![];
    let mut machine_data_reg: MachineDataRegistry = MachineDataRegistry {
        storage: HashMap::new(),
    };

    for i in 0..ecat_controller.subdevice_count {
        let dev = device_from_subdevice_identity_rc(ecat_controller.subdevices[i]).unwrap();
        subdevices.push(dev.clone());
    }

    let mut di_machine: DigitalInputTestMachine =
        DigitalInputTestMachine::new(subdevices.clone()).unwrap();
    let sender = di_machine.get_api_sender();
    let state_clone = Arc::clone(&state);
    
    rt.spawn(async move {
        let _res = state.send_ethercat_setup_done();
        state_clone
            .add_machine(
                di_machine.machine_identification_unique.into(),
                None,
                sender,
            )
            .await; // Assuming add_machine is async
        let _res = state.send_machines_event().await;
        
    });




    loop {
        write_ecat_inputs(
            &mut ecat_handle,
            ecat_controller.clone(),
            subdevices.clone(),
        );

        di_machine.act(Some(&mut machine_data_reg));
        di_machine.react(&machine_data_reg);

        write_ecat_outputs(
            &mut ecat_handle,
            ecat_controller.clone(),
            subdevices.clone(),
        );
    }
}
