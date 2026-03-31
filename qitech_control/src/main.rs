use apis::socketio::queue::start_socketio_queue;
use app_state::SharedAppState;
use machine_implementations::{Hardware, IdentifiedEthercat, MachineApi, MachineHardware, MachineNew, QiTechMachine};
use machine_implementations::minimal_machines::digital_input_test_machine::DigitalInputTestMachine;
use machine_loop::{run_machines, write_ecat_inputs, write_ecat_outputs};

use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::machine_ident_read::{MachineDeviceInfo, read_device_identifications};
use qitech_lib::machines::Machine;
use qitech_lib::{
    ethercat_hal::{
        devices::{EthercatDevice, device_from_subdevice_identity_rc},
        start_ethercat_thread,
    },
    machines::{MachineDataRegistry},
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
mod machine_loop;


static RUNTIME: OnceLock<Runtime> = OnceLock::new();
fn get_async_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio Runtime")
    })
}

struct MainState {
    pub subdevices: Vec<Rc<RefCell<dyn EthercatDevice>>>,    
    pub hardware : Vec<MachineHardware>,
    pub machines : Vec<Box<dyn QiTechMachine>>,
    pub machine_data_reg : MachineDataRegistry,
}

impl MainState {
    pub fn new() -> Self {
        let machines = vec![];
        let machine_data_reg = MachineDataRegistry{ storage: HashMap::new() };
        MainState { machines, machine_data_reg, subdevices : vec![],hardware : vec![]}
    }
    
    pub fn initialize_machines_by_identified_hw(&mut self, device_infos : Vec<MachineDeviceInfo>, mut ethercat_devices_mapped : Vec<(MetaSubdevice,Rc<RefCell<dyn EthercatDevice>>)>) {            
        println!("lol");
        let mut hw_map: HashMap<u16, (MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)> = 
            ethercat_devices_mapped
                .drain(..)
                .map(|(meta, device)| (meta.device_address, (meta, device)))
                .collect();

    
       let combined_list: Vec<(MachineDeviceInfo, MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)> = 
        device_infos
            .into_iter()
            .filter_map(|info| {
                let address = info.device_address;                
                hw_map.remove(&address).map(|hw| {
                    (info, hw.0, hw.1) // This is a 3-tuple
                })
            })
            .collect();
        
        for (i,v,g) in combined_list {
            println!("{:?} {:?} {:?}",i,v,g);
        }

    }
}



fn main() {
    let rt = get_async_runtime();
    let state = Arc::new(SharedAppState::new());
    let _api = rt.spawn(apis::init_api(state.clone()));
    let mut main_state = MainState::new();
    let eth_control = start_ethercat_thread("enp101s0f4u1u2");
    let mut ecat_handle = eth_control.app_handle;
    let ecat_channel = eth_control.channel;
    let ecat_controller = eth_control.controller;
    
    let state_clone = state.clone();
    rt.spawn(
        start_socketio_queue(state_clone)
    );    

    let _res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);
    std::thread::sleep(Duration::from_millis(1000));
    let res = ecat_channel.read_device_identifications();
    let idents = match res {
        Ok(idents) => {
            println!("Succ to read idents {:?}",idents);
            Some(idents)
        },
        Err(e) => {
            println!("Failed to read idents {:?}",e);
            None
        },
    };

    let res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);

    let mut mapped_ecat_devices = vec![];    
    for i in 0..ecat_controller.subdevice_count {
        let meta = ecat_controller.subdevices[i];
        let dev = device_from_subdevice_identity_rc(meta).unwrap();   
        main_state.subdevices.push(dev.clone());
        mapped_ecat_devices.push((meta,dev));
    }

    let _res = state.fill_ethercat_metadata(ecat_controller.clone());

    match idents {
        Some(idents) => main_state.initialize_machines_by_identified_hw(idents,mapped_ecat_devices),
        None => (),
    }


//    let state_clone = state.clone();
/*
    rt.spawn(async move {
        let _res = state_clone.send_ethercat_setup_done().await;
        state_clone
            .add_machine(
                ident.into(),
                None,
                sender,
            )
            .await; // Assuming add_machine is async
        let _res = state_clone.send_machines_event().await;
    });
*/
/*
    loop {        
        write_ecat_inputs(
            &mut ecat_handle,
            ecat_controller.clone(),
            main_state.subdevices.clone(),
        );
        
        run_machines(&mut main_state.machines, &mut main_state.machine_data_reg);
        
        write_ecat_outputs(
            &mut ecat_handle,
            ecat_controller.clone(),
            main_state.subdevices.clone(),
        );
    }*/
}
