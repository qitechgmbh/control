use apis::socketio::queue::start_socketio_queue;
use app_state::SharedAppState;
use machine_implementations::MACHINE_LASER_V1;
use machine_implementations::registry::MACHINE_REGISTRY;
#[cfg(not(feature = "mock"))]
use machine_loop::{run_machines, write_ecat_inputs, write_ecat_outputs};
#[cfg(not(feature = "mock"))]
use qitech_lib::ethercat_hal::{
    DcConfiguration, MasterConfiguration, RtOptimizationConfig, init_ethercat,
};
use qitech_lib::{
    ethercat_hal::devices::device_from_subdevice_identity_rc, serial::get_available_ports,
};
use qitech_lib::{
    ethercat_hal::interface_discovery::{LinkType, list_ethernet_interfaces, test_interface},
    ethercat_hal::{
        BECKHOFF_VENDOR_ID, EtherCATControl, Mailbox, TripleBufConsumer, TripleBufProducer,
    },
    machines::MachineIdentificationUnique,
};
#[cfg(not(feature = "mock"))]
use std::{sync::Arc, time::Duration};

#[cfg(not(feature = "mock"))]
use crate::app_state::MainState;
use crate::{
    apis::socketio::main_namespace::{
        ethercat_devices_event::EcatState,
        ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent,
    },
    app_state::get_async_runtime,
};

pub mod apis;
mod app_state;
mod machine_loop;
#[cfg(feature = "mock")]
mod mock;
pub mod persist;

fn setup_ethercat(
    state: Arc<SharedAppState>,
    main_state: &mut MainState,
    eth_control: &EtherCATControl<Arc<Mailbox>, TripleBufProducer, TripleBufConsumer, Arc<Mailbox>>,
) {
    let _res = eth_control
        .channel
        .request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);

    // Require 2 consecutive stable polls (~100 ms) in PreOp before proceeding.
    // One poll is not enough: the state machine may still be mid-iteration on first observation,
    // causing EEPROM reads to contend with its ongoing preop_group tick.
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut stable_ticks: u32 = 0;
    while stable_ticks < 2 {
        // State-machine thread died, or timeout — bail for a clean restart.
        if eth_control
            .join_handle
            .as_ref()
            .map_or(false, |h| h.is_finished())
            || std::time::Instant::now() >= deadline
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
        // Happy path: bus is in PreOp and subdevices have been enumerated.
        let preop_ready = eth_control.controller.get_state()
            == qitech_lib::ethercat_hal::EtherCATState::PreOp
            && eth_control.controller.get_subdevice_count() > 0;
        if preop_ready {
            stable_ticks += 1
        } else {
            stable_ticks = 0
        }
    }

    let mut idents = vec![];
    println!(
        "Initialized {} subdevices",
        eth_control.controller.get_subdevice_count()
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
        Ok(eeprom_idents) => {
            let mut machine_idents = eeprom_idents;

            match persist::read_machine_device_info() {
                Ok(saved_idents) if !saved_idents.is_empty() => {
                    println!(
                        "Applying {} saved machine device assignment(s)",
                        saved_idents.len()
                    );

                    for saved_ident in saved_idents {
                        if let Some(ident) = machine_idents
                            .iter_mut()
                            .find(|ident| ident.device_address == saved_ident.device_address)
                        {
                            *ident = saved_ident;
                        } else {
                            machine_idents.push(saved_ident);
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => println!("Could not read saved machine device assignments: {:?}", e),
            }

            main_state.generate_machine_hardware_from_ethercat(
                &machine_idents,
                main_state.subdevices.clone(),
                eth_control.channel.clone(),
            );
            idents.append(&mut machine_idents);
        }
        Err(e) => {
            println!("Could not read device identifications from eeprom: {:?}", e);
        }
    };
    let _res = state.fill_ethercat_metadata(eth_control.controller.clone(), idents);
}

fn add_laser(
    main_state: &mut MainState,
    shared_state: Arc<SharedAppState>,
) -> Result<(), anyhow::Error> {
    let machine_index_to_remove = main_state
        .machines
        .iter()
        .position(|m| m.get_identification().machine_ident.machine == MACHINE_LASER_V1);
    let machine_obj_index = shared_state.machines.try_read()?.iter().position(|m| {
        m.machine_identification_unique
            .machine_identification
            .machine
            == MACHINE_LASER_V1
    });

    match machine_index_to_remove {
        Some(index) => {
            main_state.machines.remove(index);
        }
        None => (),
    }

    match machine_obj_index {
        Some(index) => {
            shared_state.machines.try_write()?.remove(index);
        }
        None => (),
    }
    // Port is not used right now, so check if port exists
    let ports = get_available_ports()?;
    for port in ports {
        if port.port_name == "/dev/ttyUSB0" || port.port_name == "/dev/ttyUSB1" {
            main_state.generate_machine_hardware_from_serial(&port.port_name)?;
            detect_and_build_machines(shared_state.clone(), main_state);
            send_machines_event(shared_state);
            println!("send_setup_done_events");
            break;
        }
    }
    Ok(())
}

fn laser_hotplug(
    main_state: &mut MainState,
    shared_state: Arc<SharedAppState>,
) -> Result<(), anyhow::Error> {
    match main_state
        .machines
        .iter()
        .any(|x| x.get_identification().machine_ident.machine == MACHINE_LASER_V1)
    {
        true => Ok(()),
        false => {
            add_laser(main_state, shared_state.clone())?;
            Ok(())
        }
    }
}

fn send_machines_event(state: Arc<SharedAppState>) {
    let rt = get_async_runtime();
    rt.spawn(async move {
        let _res = state.send_machines_event().await;
    });
}

fn finalize_ethercat(
    main_state: &mut MainState,
    eth_control: &EtherCATControl<Arc<Mailbox>, TripleBufProducer, TripleBufConsumer, Arc<Mailbox>>,
) {
    let _res = eth_control
        .channel
        .request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);
    while !eth_control.controller.is_all_operational() {
        if eth_control
            .join_handle
            .as_ref()
            .map_or(false, |h| h.is_finished())
        {
            // State machine died before reaching OP — bail so main_logic can exit cleanly.
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
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

fn send_ethercat_devices_event(state: Arc<SharedAppState>) {
    let rt = get_async_runtime();
    rt.spawn(async move {
        let _res = state.send_ethercat_setup_done().await;
    });
}

fn send_setup_done_events(state: Arc<SharedAppState>) {
    let rt = get_async_runtime();
    rt.spawn(async move {
        let _res = state.send_ethercat_setup_done().await;
        let _res = state.send_machines_event().await;
    });
}

fn send_ecat_state(state: Arc<SharedAppState>, ecat_state: EcatState) {
    let rt = get_async_runtime();
    rt.spawn(async move {
        let _res = state.send_ethercat_state(ecat_state).await;
    });
}

fn setup_api_and_websock(state: Arc<SharedAppState>) {
    let rt = get_async_runtime();
    rt.spawn(apis::init_api(state.clone()));
    rt.spawn(start_socketio_queue(state));
}

fn detect_and_build_machines(state: Arc<SharedAppState>, main_state: &mut MainState) {
    let idents: Vec<MachineIdentificationUnique> = main_state
        .machines
        .iter()
        .map(|machine| machine.get_identification())
        .collect();

    for key in main_state.hardware.keys() {
        if idents.contains(key) {
            continue;
        }
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
                println!("detect_and_build_machines {:?}", e);
                if !main_state.machine_errors.contains_key(key) {
                    let _res =
                        state.add_machine_sync(key.clone().into(), Some(e.to_string()), None);
                }
                main_state.machine_errors.insert(*key, e.to_string());
            }
        };
    }
}

fn optimized_ethercat_init(
    interface: &str,
) -> EtherCATControl<Arc<Mailbox>, TripleBufProducer, TripleBufConsumer, Arc<Mailbox>> {
    let target_cycle_time_us: u64 = 1000;
    let dc_config: DcConfiguration = DcConfiguration {
        start_delay: Duration::from_millis(100),
        sync0_period: Duration::from_micros(target_cycle_time_us),
        sync0_shift: Duration::from_micros(target_cycle_time_us / 2),
        target_dc_tick: 500,
    };

    let opt_config: RtOptimizationConfig = RtOptimizationConfig {
        ethercat_loop_thread_core: 2,
        ethercat_loop_thread_priority: 99,
        ethercat_io_thread_core: 3,
        ethercat_io_thread_priority: 99,
        pin_irq_core: None,
    };

    let config: MasterConfiguration = MasterConfiguration {
        target_cycle_time_us: target_cycle_time_us as usize,
        tx_rx_config: qitech_lib::ethercat_hal::MasterTxRxConfig::TxRxIoUring,
        realtime_optimizations: Some(opt_config),
        dc_config,
        wkc_mismatch_threshold: 5,
        op_ramp_grace_cycles: 10000,
    };
    init_ethercat(interface, Some(config))
}

pub fn remove_machines(
    main_state: &mut MainState,
    shared_state: Arc<SharedAppState>,
    machines_to_remove: Option<usize>,
) {
    match machines_to_remove {
        Some(i) => {
            let machine = main_state
                .machines
                .get(i)
                .expect("Should not be none as we got an index into the machines vec");
            let ident = machine.get_identification();
            main_state.machine_data_reg.zero_entry(ident);
            main_state.machines.remove(i);
            let mut guard = shared_state
                .machines
                .try_write()
                .expect("sharedstate.machines Should never be locked here!!!"); // Is expected to never be locked at this point
            let pos = guard
                .iter()
                .position(|x| x.machine_identification_unique == ident.into())
                .expect("Machine has to still exist as metadata at this point");
            main_state.hardware.remove(&ident);
            guard.remove(pos);
            drop(guard);
            send_machines_event(shared_state.clone());
        }
        None => (),
    }
}

fn find_ethercat_interface(state: &SharedAppState) -> String {
    loop {
        let _ = state
            .emit_ethercat_interface_discovery(EthercatInterfaceDiscoveryEvent::Discovering(true));
        let interfaces = list_ethernet_interfaces();
        match interfaces {
            Ok(interfaces) => {
                for interface in interfaces {
                    match interface.link_type {
                        LinkType::Link => (),
                        LinkType::Unknown => {
                            continue;
                        }
                        LinkType::Ipv4 => continue,
                        LinkType::Ipv6 => continue,
                    };

                    let res = test_interface(&interface.name);
                    match res {
                        Ok(_) => {
                            println!("{} is ethercat", &interface.name);
                            let _ = state.emit_ethercat_interface_discovery(
                                EthercatInterfaceDiscoveryEvent::Done(interface.name.clone()),
                            );
                            return interface.name;
                        }
                        Err(_) => println!("{} is not ethercat", &interface.name),
                    }
                }
                println!("No EtherCAT interface found, retrying in 2s...");
            }
            Err(e) => {
                println!(
                    "Could not list ethernet interfaces ({:?}), retrying in 2s...",
                    e
                );
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(not(feature = "mock"))]
fn main_logic() {
    let stay_in_preop = std::env::var("QITECH_MODE").unwrap_or_default() == "preop"
        || std::env::args().any(|a| a == "preop");
    let mut shared_state = SharedAppState::new();
    let mut main_state = MainState::new();
    let interface = find_ethercat_interface(&shared_state);
    let eth_control = optimized_ethercat_init(&interface);
    shared_state.ethercat_thread_channel = Some(eth_control.channel.clone());
    let mut eth_control = Some(eth_control);

    let state = Arc::new(shared_state);
    match &eth_control {
        Some(ecat) => {
            send_ecat_state(state.clone(), ecat.controller.get_state().into());
        }
        None => (),
    }

    setup_api_and_websock(state.clone());

    match &eth_control {
        Some(control) => setup_ethercat(state.clone(), &mut main_state, control),
        None => (),
    };

    match &eth_control {
        Some(ecat) => {
            send_ecat_state(state.clone(), ecat.controller.get_state().into());
        }
        None => (),
    }

    // Subdevices are known after PreOp — show them in the frontend now
    send_ethercat_devices_event(state.clone());

    if stay_in_preop && eth_control.is_some() {
        send_setup_done_events(state.clone());
        println!("Staying in PreOp as requested, exiting after setup.");
        loop {
            std::thread::sleep(core::time::Duration::from_secs(1));
        }
    }

    // detect_and_build_machines must run in PreOp (machines initialize assuming PreOp)
    detect_and_build_machines(state.clone(), &mut main_state);

    // finalize_ethercat transitions to OP and waits until all subdevices confirm OP
    match &eth_control {
        Some(ecat) => {
            finalize_ethercat(&mut main_state, ecat);
            send_ecat_state(state.clone(), ecat.controller.get_state().into());
        }
        None => (),
    };

    // Only emit machines to frontend after OP state is confirmed
    send_machines_event(state.clone());

    let mut last_check = std::time::Instant::now();
    let hotplug_duration = Duration::from_secs(4);

    loop {
        let now = std::time::Instant::now();
        match &mut eth_control {
            Some(control) => {
                if control
                    .join_handle
                    .as_ref()
                    .expect("Join handle should be some")
                    .is_finished()
                {
                    return;
                }
                write_ecat_inputs(&mut control.app_handle, main_state.subdevices.clone());
            }
            None => (),
        };

        let machines_to_remove =
            run_machines(&mut main_state.machines, &mut main_state.machine_data_reg);
        if machines_to_remove.is_some() {
            remove_machines(&mut main_state, state.clone(), machines_to_remove);
        }

        if now.duration_since(last_check) >= hotplug_duration {
            let _ = laser_hotplug(&mut main_state, state.clone());
            last_check = now;
        }

        match &mut eth_control {
            Some(control) => {
                write_ecat_outputs(&mut control.app_handle, main_state.subdevices.clone());
            }
            None => (),
        };
        std::thread::sleep(Duration::from_micros(100));
    }
}

fn main() {
    #[cfg(not(feature = "mock"))]
    main_logic();
}
