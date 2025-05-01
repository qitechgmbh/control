use crate::app_state::{APP_STATE, EthercatSetup, Machines};
use crate::machines::registry::MACHINE_REGISTRY;
use crate::panic::{PanicDetails, send_panic};
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::ethercat_setup_event::EthercatSetupEventBuilder;
use crate::{
    app_state::AppState,
    ethercat::config::{MAX_FRAMES, MAX_PDU_DATA, MAX_SUBDEVICES, PDI_LEN},
};
use bitvec::prelude::*;
use control_core::identification::identify_device_groups;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::devices::devices_from_subdevices;
use ethercrab::std::{ethercat_now, tx_rx_task};
use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts};
use smol::channel::Sender;
use smol::lock::RwLock;
use std::collections::HashMap;
use std::{sync::Arc, time::Duration};

pub async fn setup_loop(
    thread_panic_tx: Sender<PanicDetails>,
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
            wait_loop_delay: Duration::from_millis(0),
            mailbox_response: Duration::from_millis(1000 * 10),
            state_transition: Duration::from_millis(1000 * 10),
            pdu: Duration::from_millis(1000 * 1),
            eeprom: Duration::from_millis(1000 * 1),
            mailbox_echo: Duration::from_millis(1000 * 1),
        },
        MainDeviceConfig {
            retry_behaviour: RetryBehaviour::Forever,
            ..Default::default()
        },
    );

    let _ = smol::block_on(async move {
        let main_namespace = &mut APP_STATE
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatSetupEventBuilder().initializing();
        main_namespace.emit_cached(MainNamespaceEvents::EthercatSetupEvent(event));
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

    // Identify machines
    // - Read identification values from devices
    let (identified_device_groups, unidentified_devices) =
        identify_device_groups(&group_preop, &maindevice).await?;
    // - Create Machines
    let subdevices = group_preop.iter(&maindevice).collect::<Vec<_>>();
    let mut machines: HashMap<_, _> = HashMap::new();
    for identified_device_group in identified_device_groups.iter() {
        let machine = MACHINE_REGISTRY.new_machine(identified_device_group, &subdevices, &devices);
        machines.insert(
            identified_device_group
                .first()
                .expect("There should always be a first device")
                .machine_identification_unique
                .clone(),
            machine,
        );
    }

    log::debug!("{:?}", &subdevices);

    //log machines
    for (k, v) in machines.iter() {
        log::info!("Machine: {:?} {:?}", k, v);
    }

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

    // Get propagation delays
    let propagation_delays: Vec<u32> = group_op
        .iter(&maindevice)
        .map(|subdevice| subdevice.propagation_delay())
        .collect();

    // Write all this stuff to `app_state`
    {
        let mut ethercat_setup_guard = app_state.ethercat_setup.write().await;
        *ethercat_setup_guard = Some(EthercatSetup {
            identified_device_groups,
            unidentified_devices,
            devices,
            delays: propagation_delays.into_iter().map(Some).collect(),
            group: group_op,
            maindevice,
        });
    }

    // Notify client via socketio
    let _ = smol::block_on(async move {
        let main_namespace = &mut APP_STATE
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatSetupEventBuilder().build();
        main_namespace.emit_cached(MainNamespaceEvents::EthercatSetupEvent(event));
    });

    // Start control loop
    std::thread::Builder::new()
        .name("EthercatLoopThread".to_owned())
        .spawn(move || {
            loop {
                send_panic("EthercatLoopThread", thread_panic_tx.clone());
                let rt = smol::LocalExecutor::new();
                let res = smol::block_on(rt.run(async {
                    loop_once(app_state.ethercat_setup.clone(), app_state.machines.clone()).await
                }));
                if let Err(err) = res {
                    log::error!("Loop failed\n{:?}", err);
                }
            }
        })
        .expect("Building thread");

    Ok(())
}

pub async fn loop_once<'maindevice>(
    setup: Arc<RwLock<Option<EthercatSetup>>>,
    machines: Arc<RwLock<Machines>>,
) -> Result<(), anyhow::Error> {
    let setup_guard = setup.read().await;
    let setup = setup_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("[{}::loop_once] No setup", module_path!()))?;

    // TX/RX cycle
    setup.group.tx_rx(&setup.maindevice).await?;

    // copy inputs to devices
    for (i, subdevice) in setup.group.iter(&setup.maindevice).enumerate() {
        let mut device = setup.devices[i].as_ref().write().await;
        let input = subdevice.inputs_raw();
        let input_bits = input.view_bits::<Lsb0>();

        device.input_checked(input_bits).or_else(|e| {
            Err(anyhow::anyhow!(
                "[{}::loop_once] SubDevice with index {} failed to copy inputs\n{:?}",
                module_path!(),
                i,
                e
            ))
        })?;

        device.input_post_process().or_else(|e| {
            Err(anyhow::anyhow!(
                "[{}::loop_once] SubDevice with index {} failed to copy post_process\n{:?}",
                module_path!(),
                i,
                e
            ))
        })?;
    }

    // execute machines
    let mut machines_guard = machines.write().await;
    let now = std::time::Instant::now();
    for machine_result in machines_guard.values_mut() {
        if let Ok(machine) = machine_result {
            machine.act(now);
        }
    }

    // copy outputs from devices
    for (i, subdevice) in setup.group.iter(&setup.maindevice).enumerate() {
        let mut device = setup.devices[i].as_ref().write().await;
        let mut output = subdevice.outputs_raw_mut();
        let output_bits = output.view_bits_mut::<Lsb0>();

        device.output_pre_process().or_else(|e| {
            Err(anyhow::anyhow!(
                "[{}::loop_once] SubDevice with index {} failed to pre process outputs \n{:?}",
                module_path!(),
                i,
                e
            ))
        })?;

        device.output_checked(output_bits).or_else(|e| {
            Err(anyhow::anyhow!(
                "[{}::loop_once] SubDevice with index {} failed to copy outputs\n{:?}",
                module_path!(),
                i,
                e
            ))
        })?;
    }
    Ok(())
}
