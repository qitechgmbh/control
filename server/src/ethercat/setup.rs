use crate::app_state::EthercatSetup;
use crate::machines::registry::MACHINE_REGISTRY;
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
use ethercrab::std::{ethercat_now, tx_rx_task};
use ethercrab::{
    MainDevice, MainDeviceConfig, PduLoop, PduRx, PduStorage, PduTx, RetryBehaviour, Timeouts,
};
use std::{sync::Arc, time::Duration};

const SM_OUTPUT: u16 = 0x1C32;
const SM_INPUT: u16 = 0x1C33;

pub struct EtherCatBackend<'maindevice> {
    pub tx: PduTx<'maindevice>,
    pub rx: PduRx<'maindevice>,
    pub interface: String,
}

/*
pub async hack_el5125(){
        for subdevice in subdevices.iter() {
            if subdevice.name() == "EL5152" {
                subdevice.sdo_write(SM_OUTPUT, 0x1, 0x00u16).await; //set sync mode (1) for free run (0)
                subdevice.sdo_write(SM_INPUT, 0x1, 0x00u16).await; //set sync mode (1) for free run (0)
            }
        }
}
*/
pub async fn setup_loop(
    interface: &str,
    app_state: Arc<AppState>,
) -> Result<EtherCatBackend<'_>, anyhow::Error> {
    tracing::info!("Starting EtherCAT PDU loop");

    // Set real-time priority and CPU affinity
    set_realtime_priority();
    #[cfg(all(target_os = "linux", not(feature = "development-build")))]
    set_irq_affinity(interface, 3)
        .map(|_| tracing::info!("Ethernet IRQ on CPU 3"))
        .map_err(|e| tracing::error!("Failed IRQ affinity: {:?}", e))
        .ok();

    *app_state.ethercat_setup.write().await = None;

    // Setup PDU storage
    let pdu_storage = Box::leak(Box::new(PduStorage::<MAX_FRAMES, MAX_PDU_DATA>::new()));
    let (tx, rx, pdu) = pdu_storage.try_split().expect("can only split once");
    let interface: String = interface.to_string();
    let cloned_interface = interface.clone();
    let ethercat_thread = std::thread::Builder::new()
        .name("EthercatTxRxThread".to_owned())
        .spawn(move || {
            #[cfg(all(target_os = "linux", not(feature = "development-build")))]
            match set_irq_affinity(&cloned_interface, 3) {
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
                block_on(rt.run(async {
                    tx_rx_task(&cloned_interface, tx, rx)
                        .expect("spawn TX/RX task")
                        .await
                }))
            }
            #[cfg(all(target_os = "linux", feature = "io-uring"))]
            {
                use ethercrab::std::tx_rx_task_io_uring;

                tx_rx_task_io_uring(&cloned_interface, tx, rx)
                    .expect("Failed to spawn TX/RX task (io_uring)")
            }
        })
        .expect("Building thread");

    // Create maindevice
    let maindevice = MainDevice::new(
        pdu,
        Timeouts {
            state_transition: Duration::from_millis(5000),
            pdu: Duration::from_micros(30_000),
            eeprom: Duration::from_millis(10),
            wait_loop_delay: Duration::from_millis(0),
            mailbox_echo: Duration::from_millis(100),
            mailbox_response: Duration::from_millis(1000),
        },
        MainDeviceConfig {
            retry_behaviour: RetryBehaviour::Count(5),
            dc_static_sync_iterations: 10_000,
        },
    );

    // Emit "initializing" event
    {
        let mut main_namespace = &mut app_state
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatDevicesEventBuilder().initializing();
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
    }

    // Initialize subdevices
    let mut group_preop = maindevice
        .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize subdevices: {:?}", e))?;

    // Create devices
    let devices =
        devices_from_subdevices::<MAX_SUBDEVICES, PDI_LEN>(&mut group_preop, &maindevice)?;
    let subdevices = group_preop.iter(&maindevice).collect::<Vec<_>>();

    // Extract device identifications
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

    let devices_tuple = device_identifications
        .into_iter()
        .zip(devices)
        .zip(&subdevices)
        .map(|((a, b), c)| (a, b, c))
        .collect::<Vec<_>>();

    let (identified_device_identifications, identified_devices, identified_subdevices): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = devices_tuple
        .iter()
        .filter(|(id, _, _)| id.device_machine_identification.is_some())
        .map(|(id, dev, sub)| (id.clone(), dev.clone(), *sub))
        .fold((Vec::new(), Vec::new(), Vec::new()), |mut acc, x| {
            acc.0.push(x.0);
            acc.1.push(x.1);
            acc.2.push(x.2);
            acc
        });

    // Construct machines
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

    // Remove subdevice from devices tuple for app_state
    let devices_clean = devices_tuple
        .iter()
        .map(|(id, dev, _)| (id.clone(), dev.clone()))
        .collect::<Vec<_>>();

    // Emit Machines event
    {
        let mut main_namespace = &mut app_state
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = MachinesEventBuilder().build(app_state.clone());
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
    }

    // Put group in operational state
    let group_op = group_preop
        .into_op(&maindevice)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to put group in OP state: {:?}", e))?;

    // Save final setup into app_state
    *app_state.ethercat_setup.write().await = Some(EthercatSetup {
        devices: devices_clean,
        group: group_op,
        maindevice,
    });

    // Emit final EthercatDevicesEvent
    {
        let mut main_namespace = &mut app_state
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatDevicesEventBuilder().build(app_state.clone()).await;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
    }
    let res: Result<(PduTx<'_>, PduRx<'_>), ethercrab::error::Error> =
        ethercat_thread.join().expect("Thread panicked");

    let tx_rx_tuple = match res {
        Ok(tuple) => tuple,
        Err(e) => return Err(anyhow::anyhow!("HELP")),
    };

    let ethercat_backend = EtherCatBackend {
        tx: tx_rx_tuple.0,
        rx: tx_rx_tuple.1,
        interface,
    };

    Ok(ethercat_backend)
}
