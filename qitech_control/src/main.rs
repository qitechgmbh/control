use apis::socketio::queue::start_socketio_queue;
use app_state::SharedAppState;
use machine_implementations::registry::MACHINE_REGISTRY;
#[cfg(not(feature = "mock"))]
use machine_loop::{run_machines, write_ecat_inputs, write_ecat_outputs};
use qitech_lib::ethercat_hal::devices::device_from_subdevice_identity_rc;
#[cfg(not(feature = "mock"))]
use qitech_lib::ethercat_hal::init_ethercat;
use qitech_lib::ethercat_hal::{
    BECKHOFF_VENDOR_ID, EtherCATControl, TripleBufConsumer, TripleBufProducer,
};
#[cfg(not(feature = "mock"))]
use qitech_lib::ethercat_hal::{DcConfiguration, MasterConfiguration, RtOptimizationConfig};
use qitech_lib::modbus::clients::example_client::ExampleClient;
use qitech_lib::modbus::managers::ExampleDeviceManager;
use qitech_lib::modbus::start_modbus_async_task;
use std::{sync::Arc, time::Duration};

use crate::app_state::{MainState, get_async_runtime};

pub mod apis;
mod app_state;
mod machine_loop;
mod mock;
pub mod persist;

fn setup_ethercat(
    state: Arc<SharedAppState>,
    main_state: &mut MainState,
    eth_control: &EtherCATControl<TripleBufConsumer, TripleBufProducer>,
) {
    let _res = eth_control
        .channel
        .request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);
    std::thread::sleep(Duration::from_millis(5000));

    let mut idents = vec![];
    println!(
        "Initialized {} subdevices",
        eth_control.controller.subdevice_count
    );

    for meta in eth_control.controller.get_subdevices() {
        let dev = device_from_subdevice_identity_rc(&meta);

        let dev = match dev {
            Ok(d) => d,
            Err(_) => {
                println!("{:?} is not implemented", meta.get_name());
                continue;
            }
        };

        main_state.subdevices.push((meta.clone(), dev.clone()));
        if meta.vendor == BECKHOFF_VENDOR_ID {
            let _res = eth_control
                .channel
                .set_mut_beckhoff_eeprom_lock_active(meta.device_address);
        }
    }

    match eth_control.channel.read_device_identifications() {
        Ok(mut eeprom_idents) => {
            main_state.generate_machine_hardware_from_ethercat(
                &eeprom_idents,
                main_state.subdevices.clone(),
                eth_control.channel.clone(),
            );
            idents.append(&mut eeprom_idents);
        }
        Err(e) => {
            println!("Could not read device identifications from eeprom: {:?}", e);
        }
    };
    let _res = state.fill_ethercat_metadata(
        eth_control.controller.clone(),
        eth_control.channel.clone(),
        idents,
    );
}

fn setup_serial(main_state: &mut MainState) {
    let rt = get_async_runtime();
    let (tx, rx) = ExampleClient::create_channels();
    rt.spawn(start_modbus_async_task(
        "/dev/ttyUSB0".to_string(),
        1,
        38400,
        rx,
    ));
    let modbus_mgr = ExampleDeviceManager::new(tx);
    main_state.generate_machine_hardware_from_serial(modbus_mgr);
}

fn finalize_ethercat(
    main_state: &mut MainState,
    eth_control: &EtherCATControl<TripleBufConsumer, TripleBufProducer>,
) {
    let _res = eth_control
        .channel
        .request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);
    std::thread::sleep(Duration::from_secs(5));
    for meta in &mut main_state.subdevices {
        let m = eth_control
            .controller
            .get_subdevices()
            .iter()
            .find(|m| m.device_address == meta.0.device_address)
            .expect("Ethercat Device Suddenly Missing in finalize_ethercat");

        meta.0.start_tx = m.start_tx;
        meta.0.end_tx = m.end_tx;
        meta.0.start_rx = m.start_rx;
        meta.0.end_rx = m.end_rx;
    }
}

fn send_setup_done_events(state: Arc<SharedAppState>) {
    let rt = get_async_runtime();
    rt.spawn(async move {
        let _res = state.send_ethercat_setup_done().await;
        let _res = state.send_machines_event().await;
    });
}

fn setup_api_and_websock(state: Arc<SharedAppState>) {
    let rt = get_async_runtime();
    rt.spawn(apis::init_api(state.clone()));
    rt.spawn(start_socketio_queue(state));
}

fn detect_and_build_machines(state: Arc<SharedAppState>, main_state: &mut MainState) {
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

fn optimized_ethercat_init(
    interface: &str,
) -> EtherCATControl<TripleBufConsumer, TripleBufProducer> {
    let target_cycle_time_us: u64 = 700;
    let dc_config: DcConfiguration = DcConfiguration {
        start_delay: Duration::from_millis(100),
        sync0_period: Duration::from_micros(target_cycle_time_us),
        sync0_shift: Duration::from_micros(target_cycle_time_us / 2),
        target_dc_tick: 300,
    };

    let opt_config: RtOptimizationConfig = RtOptimizationConfig {
        ethercat_loop_thread_core: 2,
        ethercat_loop_thread_priority: 99,
        ethercat_io_thread_core: 3,
        ethercat_io_thread_priority: 99,
        pin_irq_core: Some(3),
    };

    let config: MasterConfiguration = MasterConfiguration {
        target_cycle_time_us: target_cycle_time_us as usize,
        tx_rx_config: qitech_lib::ethercat_hal::MasterTxRxConfig::TxRxIoUring,
        realtime_optimizations: Some(opt_config),
        dc_config,
    };
    init_ethercat(interface, Some(config))
}

fn find_ethercat_interface() -> Result<String, anyhow::Error> {
    let interfaces = qitech_lib::ethercat_hal::interface_discovery::list_ethernet_interfaces();
    println!("{:?}", interfaces);
    match interfaces {
        Ok(interfaces) => {
            for interface in interfaces {
                match interface.link_type {
                    qitech_lib::ethercat_hal::interface_discovery::LinkType::Link => (),
                    qitech_lib::ethercat_hal::interface_discovery::LinkType::Unknown => continue,
                    qitech_lib::ethercat_hal::interface_discovery::LinkType::Ipv4 => continue,
                    qitech_lib::ethercat_hal::interface_discovery::LinkType::Ipv6 => continue,
                };

                let res =
                    qitech_lib::ethercat_hal::interface_discovery::test_interface(&interface.name);
                match res {
                    Ok(_) => {
                        println!("{} is ethercat", &interface.name);
                        return Ok(interface.name);
                    }
                    Err(_) => println!("{} is not ethercat", &interface.name),
                }
            }
            return Err(anyhow::anyhow!("No EtherCAT Interface Found"));
        }
        Err(_) => {
            return Err(anyhow::anyhow!("No EtherCAT Interface Found"));
        }
    }
}

#[cfg(not(feature = "mock"))]
fn main_logic() {
    let state = Arc::new(SharedAppState::new());
    let mut main_state = MainState::new();

    let res = find_ethercat_interface();
    let mut eth_control = match res {
        Ok(interface) => Some(optimized_ethercat_init(&interface)),
        Err(_) => return,
    };
    setup_api_and_websock(state.clone());
    match &eth_control {
        Some(control) => setup_ethercat(state.clone(), &mut main_state, control),
        None => (),
    };

    setup_serial(&mut main_state);
    detect_and_build_machines(state.clone(), &mut main_state);
    // Order is important here, because when detect_and_build_machines is called
    // Every machine assumes ethercat devices are in PreOp, finalize moves to OP
    match &eth_control {
        Some(control) => finalize_ethercat(&mut main_state, control),
        None => (),
    };
    send_setup_done_events(state);
    match &mut eth_control {
        Some(control) => loop {
            write_ecat_inputs(&mut control.app_handle, main_state.subdevices.clone());

            run_machines(&mut main_state.machines, &mut main_state.machine_data_reg);

            write_ecat_outputs(&mut control.app_handle, main_state.subdevices.clone());

            std::thread::sleep(Duration::from_micros(control.controller.cycle_time_us));
        },
        None => (),
    }
}
fn main() {
    #[cfg(not(feature = "mock"))]
    main_logic();
}
