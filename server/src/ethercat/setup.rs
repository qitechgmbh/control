use crate::app_state::EthercatSetup;
use crate::machines::registry::MACHINE_REGISTRY;
use crate::panic::send_panic;
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::ethercat_devices_event::EthercatDevicesEventBuilder;
use crate::socketio::main_namespace::machines_event::MachinesEventBuilder;
use crate::{
    app_state::AppState,
    ethercat::config::{MAX_FRAMES, MAX_PDU_DATA, MAX_SUBDEVICES, PDI_LEN},
};
use control_core::ethercat::eeprom_identification::read_device_identifications;
use control_core::machines::identification::{
    DeviceHardwareIdentification, DeviceHardwareIdentificationEthercat, DeviceIdentification,
};
use control_core::machines::new::MachineNewHardwareEthercat;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::devices::devices_from_subdevices;
use ethercrab::std::{ethercat_now, tx_rx_task};
use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts};
use smol::channel::Sender;
use std::{sync::Arc, time::Duration};

pub async fn setup_loop(
    thread_panic_tx: Sender<&'static str>,
    interface: &str,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    log::info!("Starting Ethercat PDU loop");

    // Erase all all setup data from `app_state`
    {
        log::info!("Setting up Ethercat network");
        let mut ethercat_setup_guard = app_state.ethercat_setup.write().await;
        *ethercat_setup_guard = None;
    }

    // Setup ethercrab tx/rx task
    let pdu_storage = Box::leak(Box::new(PduStorage::<MAX_FRAMES, MAX_PDU_DATA>::new()));
    let (tx, rx, pdu) = pdu_storage.try_split().expect("can only split once");
    let interface = interface.to_string();
    let thread_panic_tx_clone = thread_panic_tx.clone();
    std::thread::Builder::new()
        .name("EthercatTxRxThread".to_owned())
        .spawn(move || {
            send_panic("EthercatTxRxThread", thread_panic_tx_clone);
            let rt = smol::LocalExecutor::new();
            let _ = smol::block_on(rt.run(async {
                tx_rx_task(&interface, tx, rx)
                    .expect("spawn TX/RX task")
                    .await
            }));
        })
        .expect("Building thread");

    // Create maindevice
    let maindevice = MainDevice::new(
        pdu,
        Timeouts {
            // Default 5000ms
            state_transition: Duration::from_millis(10 * 1000),
            // Default 30_000us
            pdu: Duration::from_millis(1000),
            // Default 10ms
            eeprom: Duration::from_millis(10),
            // Default 0ms
            wait_loop_delay: Duration::from_millis(0),
            // Default 100ms
            mailbox_echo: Duration::from_millis(100),
            // Default 1000ms
            mailbox_response: Duration::from_millis(1000),
        },
        MainDeviceConfig {
            // Default RetryBehaviour::None
            retry_behaviour: RetryBehaviour::None,
            // Default 10_000
            dc_static_sync_iterations: 10_000,
        },
    );

    let _ = smol::block_on({
        let app_state_clone = app_state.clone();
        async move {
            let main_namespace = &mut app_state_clone
                .socketio_setup
                .namespaces
                .write()
                .await
                .main_namespace;
            let event = EthercatDevicesEventBuilder().initializing();
            main_namespace.emit_cached(MainNamespaceEvents::EthercatDevicesEvent(event));
        }
    });

    // Initalize subdevices
    // Fails if DC setup detects a mispatching working copunter, then just try again in loop
    let mut group_preop = match maindevice
        .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
        .await
    {
        Ok(group) => {
            log::info!("Initialized {} subdevices", &group.len());
            group
        }
        Err(err) => Err(anyhow::anyhow!(
            "[{}::setup_loop] Failed to initialize subdevices: {:?}",
            module_path!(),
            err
        ))?,
    };

    // create devices
    let devices =
        devices_from_subdevices::<MAX_SUBDEVICES, PDI_LEN>(&mut group_preop, &maindevice)?;
    let subdevices = group_preop.iter(&maindevice).collect::<Vec<_>>();

    // extract device identifications
    let device_identifications = read_device_identifications(&subdevices, &maindevice)
        .await
        .into_iter()
        .enumerate()
        .map(|(i, result)| (i, result.ok()))
        .map(
            |(subdevice_index, device_machine_identification)| DeviceIdentification {
                device_machine_identification,
                device_hardware_identification: DeviceHardwareIdentification::Ethercat(
                    DeviceHardwareIdentificationEthercat { subdevice_index },
                ),
            },
        )
        .collect::<Vec<_>>();

    let devices = device_identifications
        .into_iter()
        .zip(devices)
        .zip(&subdevices)
        .map(|((a, b), c)| (a, b, c))
        .collect::<Vec<_>>();

    // filter devices and if Option<DeviceMachineIdentification> is Some
    // return identified_devices, identified_device_identifications, identified_subdevices
    let (identified_device_identifications, identified_devices, identified_subdevices): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = devices
        .iter()
        .filter(|(device_identification, _, _)| {
            match device_identification {
                DeviceIdentification {
                    device_machine_identification: Some(_),
                    ..
                } => true,
                _ => false
            }
        })
        .map(|(device_identification, device, subdevice)| {
            (
                device_identification.clone(),
                device.clone(),
                *subdevice,
            )
        })
        .fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |mut acc, (identified_device_identification, identified_device, identified_subdevice)| {
                acc.0.push(identified_device_identification);
                acc.1.push(identified_device);
                acc.2.push(identified_subdevice);
                acc
            },
        );

    // construct machines
    let mut machines_guard = app_state.machines.write().await;
    machines_guard.set_ethercat_devices::<MAX_SUBDEVICES, PDI_LEN>(
        &identified_device_identifications,
        &MACHINE_REGISTRY,
        &MachineNewHardwareEthercat {
            ethercat_devices: &identified_devices,
            subdevices: &identified_subdevices,
        },
    );
    drop(machines_guard);

    // remove subdevice from devices tuple
    let devices = devices
        .iter()
        .map(|(device_identification, device, _)| (device_identification.clone(), device.clone()))
        .collect::<Vec<_>>();

    // Notify client via socketio
    let app_state_clone = app_state.clone();
    let _ = smol::block_on(async move {
        let main_namespace = &mut app_state_clone
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = MachinesEventBuilder().build(app_state_clone.clone()).await;
        main_namespace.emit_cached(MainNamespaceEvents::MachinesEvent(event));
    });

    // Put group in operational state
    let group_op = match group_preop.into_op(&maindevice).await {
        Ok(group_op) => {
            log::info!("Group in OP state");
            group_op
        }
        Err(err) => Err(anyhow::anyhow!(
            "[{}::setup_loop] Failed to put group in OP state: {:?}",
            module_path!(),
            err
        ))?,
    };

    // Write all this stuff to `app_state`
    {
        let mut ethercat_setup_guard = app_state.ethercat_setup.write().await;
        *ethercat_setup_guard = Some(EthercatSetup {
            devices,
            group: group_op,
            maindevice,
        });
    }

    // Notify client via socketio
    let app_state_clone = app_state.clone();
    let _ = smol::block_on(async move {
        let main_namespace = &mut app_state_clone
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatDevicesEventBuilder()
            .build(app_state_clone.clone())
            .await;
        main_namespace.emit_cached(MainNamespaceEvents::EthercatDevicesEvent(event));
    });

    Ok(())
}
