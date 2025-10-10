use crate::app_state::EthercatSetup;
use crate::machines::registry::MACHINE_REGISTRY;
use crate::panic::{PanicDetails, send_panic};
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::ethercat_devices_event::EthercatDevicesEventBuilder;
use crate::socketio::main_namespace::machines_event::MachinesEventBuilder;
use crate::{
    app_state::AppState,
    ethercat::config::{MAX_FRAMES, MAX_PDU_DATA, MAX_SUBDEVICES, PDI_LEN},
};
use control_core::ethercat::eeprom_identification::read_device_identifications;
#[cfg(all(target_os = "linux", not(feature = "development-build")))]
use control_core::irq_handling::set_irq_affinity;
use control_core::machines::identification::{
    DeviceHardwareIdentification, DeviceHardwareIdentificationEthercat, DeviceIdentification,
};
use control_core::machines::new::MachineNewHardwareEthercat;
use control_core::realtime::{set_core_affinity, set_realtime_priority};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::devices::devices_from_subdevices;
use ethercrab::std::ethercat_now;
use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts};
use smol::channel::Sender;
use std::{sync::Arc, time::Duration};

const SM_OUTPUT: u16 = 0x1C32;
const SM_INPUT: u16 = 0x1C33;

pub async fn setup_loop(
    thread_panic_tx: Sender<PanicDetails>,
    interface: &str,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    tracing::info!("Starting Ethercat PDU loop");

    // Erase all all setup data from `app_state`
    {
        tracing::debug!("Setting up Ethercat network");
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
            send_panic(thread_panic_tx_clone);

            #[cfg(all(target_os = "linux", not(feature = "development-build")))]
            match set_irq_affinity(&interface, 3) {
                Ok(_) => tracing::info!("ethernet interrupt handler now runs on cpu:{}", 3),
                Err(e) => tracing::error!("set_irq_handler_affinity failed: {:?}", e),
            }

            // Set core affinity to 4th core
            let _ = set_core_affinity(3);

            // Set the thread to real-time priority
            let _ = set_realtime_priority();

            #[cfg(not(all(target_os = "linux", feature = "io-uring")))]
            {
                use ethercrab::std::tx_rx_task;
                use futures::executor::block_on;

                let rt = smol::LocalExecutor::new();
                let _ = block_on(rt.run(async {
                    tx_rx_task(&interface, tx, rx)
                        .expect("spawn TX/RX task")
                        .await
                }));
            }
            #[cfg(all(target_os = "linux", feature = "io-uring"))]
            {
                use ethercrab::std::tx_rx_task_io_uring;

                let _ = tx_rx_task_io_uring(&interface, tx, rx)
                    .expect("Failed to spawn TX/RX task (io_uring)");
            }
        })
        .expect("Building thread");

    // Create maindevice
    let maindevice = MainDevice::new(
        pdu,
        Timeouts {
            // Default 5000ms
            state_transition: Duration::from_millis(5000),
            // Default 30_000us
            pdu: Duration::from_micros(30_000),
            // Default 10ms
            eeprom: Duration::from_millis(100),
            // Default 0ms
            wait_loop_delay: Duration::from_millis(0),
            // Default 100ms
            mailbox_echo: Duration::from_millis(1000),
            // Default 1000ms
            mailbox_response: Duration::from_millis(10000),
        },
        MainDeviceConfig {
            // Default RetryBehaviour::None
            retry_behaviour: RetryBehaviour::Count(5),
            // Default 10_000
            dc_static_sync_iterations: 10_000,
        },
    );

    smol::block_on({
        let app_state_clone = app_state.clone();
        async move {
            let main_namespace = &mut app_state_clone
                .socketio_setup
                .namespaces
                .write()
                .await
                .main_namespace;
            let event = EthercatDevicesEventBuilder().initializing();
            main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
        }
    });

    // Initalize subdevices
    // Fails if DC setup detects a mispatching working copunter, then just try again in loop
    let mut group_preop = match maindevice
        .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
        .await
    {
        Ok(group) => {
            tracing::info!("Initialized {} subdevices", &group.len());
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
    {
        let mut machines_guard = app_state.machines.write().await;
        machines_guard.set_ethercat_devices::<MAX_SUBDEVICES, PDI_LEN>(
            &identified_device_identifications,
            &MACHINE_REGISTRY,
            &MachineNewHardwareEthercat {
                ethercat_devices: &identified_devices,
                subdevices: &identified_subdevices,
            },
            app_state.socketio_setup.socket_queue_tx.clone(),
            Arc::downgrade(&app_state.machines),
        );
    }

    // remove subdevice from devices tuple
    let devices = devices
        .iter()
        .map(|(device_identification, device, _)| (device_identification.clone(), device.clone()))
        .collect::<Vec<_>>();

    for subdevice in subdevices.iter() {
        if subdevice.name() == "EL5152" {
            subdevice.sdo_write(SM_OUTPUT, 0x1, 0x00u16).await?; //set sync mode (1) for free run (0)
            subdevice.sdo_write(SM_INPUT, 0x1, 0x00u16).await?; //set sync mode (1) for free run (0)
        }
    }
    // Notify client via socketio
    let app_state_clone = app_state.clone();
    smol::block_on(async move {
        let main_namespace = &mut app_state_clone
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = MachinesEventBuilder().build(app_state_clone.clone());
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
    });

    // Put group in operational state
    let group_op = match group_preop.into_op(&maindevice).await {
        Ok(group_op) => {
            tracing::info!("Group in OP state");
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
    smol::block_on(async move {
        let main_namespace = &mut app_state_clone
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatDevicesEventBuilder()
            .build(app_state_clone.clone())
            .await;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
    });

    Ok(())
}
