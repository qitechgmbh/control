use apis::socketio::queue::start_socketio_queue;
use app_state::SharedAppState;
use machine_implementations::registry::MACHINE_REGISTRY;
use machine_loop::{run_machines, write_ecat_inputs, write_ecat_outputs};
use qitech_lib::ethercat_hal::controller::{TripleBufConsumer, TripleBufProducer};
use qitech_lib::modbus::clients::example_client::ExampleClient;
use qitech_lib::modbus::managers::ExampleDeviceManager;
use qitech_lib::modbus::start_modbus_async_task;
use qitech_lib::{ethercat_hal::devices::{device_from_subdevice_identity_rc}};
use qitech_lib::ethercat_hal::{EtherCATControl, init_ethercat};
use crate::app_state::get_async_runtime;
use crate::app_state::MainState;

use std::{
    collections::HashMap,
    sync::{Arc},
    time::Duration,
};

pub mod apis;
pub mod persist;
mod app_state;
mod machine_loop;
mod mock;
mod mock_main;

fn setup_ethercat(
    state : Arc<SharedAppState>,
    main_state : &mut MainState ,
    eth_control : &EtherCATControl<TripleBufConsumer,TripleBufProducer>)
{

    let _res = eth_control.channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);
    std::thread::sleep(Duration::from_millis(5000));

    let mut idents = vec![];
    println!("Initialized {} subdevices",eth_control.controller.subdevice_count);

    let mut devices_by_address = eth_control.controller
        .get_subdevices()
        .into_iter()
        .map(|meta| {
            let dev = device_from_subdevice_identity_rc(meta).unwrap();
            (meta.device_address, (meta.clone(), dev))
        })
        .collect::<HashMap<_, _>>();

    for (_, dev) in devices_by_address.values() {
        main_state.subdevices.push(dev.clone());
    }

    match eth_control.channel.read_device_identifications() {
        Ok(mut eeprom_idents) => {
            main_state.generate_machine_hardware_from_ethercat(
                &eeprom_idents,
                &mut devices_by_address,
                eth_control.channel.clone(),
            );
            idents.append(&mut eeprom_idents);
        }
        Err(e) => {
            println!("Could not read device identifications from eeprom: {:?}", e);
        }
    };
    let _res = state.fill_ethercat_metadata(eth_control.controller.clone(), idents);
}

fn setup_serial(main_state : &mut MainState){
    let rt = get_async_runtime();
    let (tx, rx) = ExampleClient::create_channels();
    rt.spawn(start_modbus_async_task("/dev/ttyUSB0".to_string(),1,38400,rx));
    let modbus_mgr = ExampleDeviceManager::new(tx);
    main_state.generate_machine_hardware_from_serial(modbus_mgr);
}

fn finalize_ethercat(eth_control : &EtherCATControl<TripleBufConsumer,TripleBufProducer>) {
    let _res = eth_control.channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);
    std::thread::sleep(Duration::from_secs(1));
}

fn send_setup_done_events(state : Arc<SharedAppState>){
    let rt = get_async_runtime();
    rt.spawn(async move {
        let _res = state.send_ethercat_setup_done().await;
        let _res = state.send_machines_event().await;
    });
}

fn setup_api(state : Arc<SharedAppState>){
    let rt = get_async_runtime();
    rt.spawn(apis::init_api(state.clone()));
    rt.spawn(start_socketio_queue(state));
}

fn detect_and_build_machines(state : Arc<SharedAppState>,main_state : &mut MainState){
    for key in main_state.hardware.keys() {
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
}

#[cfg(not(feature = "mock"))]
fn main_logic(){
    let state = Arc::new(SharedAppState::new());
    let mut main_state = MainState::new();
    let mut eth_control = init_ethercat("enp4s0");
    setup_api(state.clone());
    setup_ethercat(state.clone(),&mut main_state,&eth_control);
    setup_serial(&mut main_state);
    println!("Initialized {} hw",main_state.hardware.keys().len());
    detect_and_build_machines(state.clone(),&mut main_state);
    // Order is important here, because when detect_and_build_machines is called
    // Every machine assumes ethercat devices are in PreOp finalize moves to OP
    finalize_ethercat(&eth_control);
    send_setup_done_events(state);

    loop {
        write_ecat_inputs(
            &mut eth_control.app_handle,
            eth_control.controller.clone(),
            main_state.subdevices.clone(),
        );
        run_machines(&mut main_state.machines, &mut main_state.machine_data_reg);
        write_ecat_outputs(
            &mut eth_control.app_handle,
            eth_control.controller.clone(),
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
