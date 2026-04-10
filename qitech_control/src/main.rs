use apis::socketio::queue::start_socketio_queue;
use app_state::SharedAppState;
use machine_implementations::registry::MACHINE_REGISTRY;
use machine_implementations::{Hardware, IdentifiedEthercat, MachineHardware, QiTechMachine};
use machine_loop::{run_machines, write_ecat_inputs, write_ecat_outputs};
use qitech_lib::ethercat_hal::{EtherCATThreadChannel, MetaSubdevice, init_ethercat};
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use qitech_lib::{
    ethercat_hal::devices::{EthercatDevice, device_from_subdevice_identity_rc},
    machines::MachineDataRegistry,
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
    pub hardware: HashMap<MachineIdentificationUnique, MachineHardware>,
    pub machines: Vec<Box<dyn QiTechMachine>>,
    pub machine_errors: HashMap<MachineIdentificationUnique, String>,
    pub machine_data_reg: MachineDataRegistry,
}

impl MainState {
    pub fn new() -> Self {
        let machines = vec![];
        let machine_data_reg = MachineDataRegistry {
            storage: HashMap::new(),
        };
        MainState {
            machines,
            machine_data_reg,
            subdevices: vec![],
            hardware: HashMap::new(),
            machine_errors: HashMap::new(),
        }
    }

    pub fn generate_machine_hardware_from_ethercat(
        &mut self,
        device_infos: Vec<MachineDeviceInfo>,
        mut ethercat_devices_mapped: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
        ethercat_interface : EtherCATThreadChannel,
    ) {
        let mut hw_map: HashMap<u16, (MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)> =
            ethercat_devices_mapped
                .drain(..)
                .map(|(meta, device)| (meta.device_address, (meta, device)))
                .collect();

        let mut combined_list: Vec<(
            MachineDeviceInfo,
            MetaSubdevice,
            Rc<RefCell<dyn EthercatDevice>>,
        )> = device_infos
            .into_iter()
            .filter_map(|info| {
                let address = info.device_address;
                hw_map.remove(&address).map(|hw| {
                    (info, hw.0, hw.1) // This is a 3-tuple
                })
            })
            .collect();

        combined_list.sort_by_key(|f| (f.0.machine_id, f.0.machine_serial));
        for (dev_info, _, eth) in combined_list.drain(0..combined_list.len()) {
            let identification = MachineIdentificationUnique {
                machine_ident: MachineIdentification {
                    vendor: dev_info.machine_vendor,
                    machine: dev_info.machine_id,
                },
                serial: dev_info.machine_serial as u32,
            };
            if self.hardware.get(&identification).is_none() {
                self.hardware.insert(
                    identification,
                    MachineHardware {
                        hw: vec![],
                        identification,
                        ethercat_interface: Some(ethercat_interface.clone()),
                    },
                );
            }
            let ethercat_hw = Hardware::Ethercat(IdentifiedEthercat {
                hw: eth,
                ident: dev_info,
            });
            let hw = self.hardware.get_mut(&identification).unwrap();
            hw.hw.push(ethercat_hw);
        }
    }
}

fn mock_logic(){

}

fn main_logic(){
let rt = get_async_runtime();
    let state = Arc::new(SharedAppState::new());
    let _api = rt.spawn(apis::init_api(state.clone()));
    let mut main_state = MainState::new();
    let eth_control = init_ethercat("enp101s0f4u1u2");

    let mut ecat_handle = eth_control.app_handle;
    let ecat_channel = eth_control.channel;
    let ecat_controller = eth_control.controller;

    let state_clone = state.clone();
    rt.spawn(start_socketio_queue(state_clone));

    let _res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);
    std::thread::sleep(Duration::from_millis(5000));

    let res = ecat_channel.read_device_identifications();
    println!("{:?}",res);
    let idents = match res {
        Ok(idents) => Some(idents),
        Err(_e) => {
            println!("{:?}",_e);
            None
        },
    };

    let mut mapped_ecat_devices = vec![];
    println!("Initialized {} subdevices",ecat_controller.subdevice_count);

    for i in 0..ecat_controller.subdevice_count {
        let meta = ecat_controller.subdevices[i];
        let dev = device_from_subdevice_identity_rc(meta).unwrap();
        main_state.subdevices.push(dev.clone());
        mapped_ecat_devices.push((meta, dev));
    }
    
    match idents.clone() {
        Some(idents) => {
            main_state.generate_machine_hardware_from_ethercat(idents, mapped_ecat_devices,ecat_channel.clone())
        }
        None => (),
    }
    let _res = state.fill_ethercat_metadata(ecat_controller.clone(), idents);

    println!("Initialized {} hw",main_state.hardware.keys().len());

    for key in main_state.hardware.keys() {
        println!("{:?}",key);
        let result = MACHINE_REGISTRY
            .new_machine(key.clone(), main_state.hardware.get(key).unwrap().clone());
        match result {
            Ok(machine) => {
                let _res = state.add_machine_sync(
                    key.clone().into(),
                    None,
                    Some(machine.get_api_sender()),
                );
                main_state.machines.push(machine);
            }
            Err(e) => {
                println!("{:?}", e);
                main_state.machine_errors.insert(*key, e.to_string());
            }
        };
    }

    let _res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);

    let state_clone = state.clone();
    rt.spawn(async move {
        let _res = state_clone.send_ethercat_setup_done().await;
        let _res = state_clone.send_machines_event().await;
    });

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
        std::thread::sleep(Duration::from_micros(100));
    }
}

fn main() {
    #[cfg(feature = "mock")]
    mock_logic();

    #[cfg(not(feature = "mock"))]
    main_logic();
}
