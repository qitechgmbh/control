use crate::app_state::{EtherCatDeviceMetaData, EthercatSetup};
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::ethercat_devices_event::EthercatDevicesEventBuilder;
use crate::socketio::main_namespace::machines_event::MachineObj;
use crate::{
    app_state::SharedState,
    ethercat::config::{MAX_FRAMES, MAX_PDU_DATA, MAX_SUBDEVICES, PDI_LEN},
};
use control_core::realtime::set_core_affinity;
use control_core::socketio::namespace::NamespaceCacheingLogic;
#[cfg(all(target_os = "linux", not(feature = "development-build")))]
use control_core::{irq_handling::set_irq_affinity, realtime::set_realtime_priority};
use ethercat_hal::debugging::diagnosis_history::get_most_recent_diagnosis_message;
use ethercat_hal::devices::devices_from_subdevices;
use ethercat_hal::devices::wago_750_354::{
    WAGO_750_354_PRODUCT_ID, WAGO_750_354_VENDOR_ID, Wago750_354,
};
use ethercat_hal::devices::wago_modules::ip20_ec_di8_do8::{
    IP20_EC_DI8_DO8_PRODUCT_ID, IP20_EC_DI8_DO8_VENDOR_ID, IP20EcDi8Do8,
};
use ethercrab::error::Error;
use ethercrab::subdevice_group::{DcConfiguration, HasDc, PreOpPdi, SafeOp};
use smol::stream::StreamExt;
use ta::Next;
use ta::indicators::ExponentialMovingAverage;

use crate::utils::stop_dnsmasq;
use ethercrab::std::ethercat_now;
use ethercrab::{
    DcSync, MainDevice, MainDeviceConfig, PduStorage, RegisterAddress, RetryBehaviour,
    SubDeviceGroup, Timeouts,
};
use machines::machine_identification::{
    DeviceHardwareIdentification, DeviceHardwareIdentificationEthercat, DeviceIdentification,
    DeviceIdentificationIdentified, MachineIdentificationUnique, read_device_identifications,
};
use machines::registry::{MACHINE_REGISTRY, MachineRegistry};
use machines::{Machine, MachineNewHardware, MachineNewHardwareEthercat, MachineNewParams};
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use std::time::Instant;
use std::{sync::Arc, time::Duration};

/// Structure to hold the result of grouping devices by identification
#[derive(Debug)]
pub struct DeviceGroupingResult {
    /// Devices grouped by machine identification
    pub device_groups: Vec<Vec<DeviceIdentificationIdentified>>,
    /// Devices that could not be identified
    pub unidentified_devices: Vec<DeviceIdentification>,
}

pub fn group_devices_by_identification(
    device_identifications: &Vec<DeviceIdentification>,
) -> DeviceGroupingResult {
    let mut device_groups: Vec<Vec<DeviceIdentificationIdentified>> = Vec::new();
    let mut unidentified_devices: Vec<DeviceIdentification> = Vec::new();

    for device_identification in device_identifications {
        // if vendor or serial or machine is 0, it is not a valid machine device
        if let Some(device_machine_identification) =
            device_identification.device_machine_identification.as_ref()
        {
            if !device_machine_identification.is_valid() {
                unidentified_devices.push(device_identification.clone());

                continue;
            }
        } else {
            unidentified_devices.push(device_identification.clone());
            continue;
        }

        // scan over all deice groups
        // get the first DeviceMachineIdentification
        // compare and append to the group
        let mut found = false;
        for check_group in device_groups.iter_mut() {
            // get first device in group
            let first_device = check_group.first().expect("group to not be empty");
            let first_device_machine_identification = &first_device
                .device_machine_identification
                .machine_identification_unique;

            // chek if it has machine identification
            if let Some(device_machine_identification) =
                device_identification.device_machine_identification.as_ref()
            {
                // compare with the current device
                if first_device_machine_identification
                    == &device_machine_identification.machine_identification_unique
                {
                    let device_identification_identified = device_identification
                        .clone()
                        .try_into()
                        .expect("should have Some(DeviceMachineIdentification)");
                    check_group.push(device_identification_identified);
                    found = true;
                    break;
                }
            }
        }

        if !found {
            let device_identification_identified = device_identification
                .clone()
                .try_into()
                .expect("should have Some(DeviceMachineIdentification)");
            device_groups.push(vec![device_identification_identified]);
        }
    }

    DeviceGroupingResult {
        device_groups,
        unidentified_devices,
    }
}

pub async fn set_ethercat_devices<const MAX_SUBDEVICES: usize, const MAX_PDI: usize>(
    device_identifications: &Vec<DeviceIdentification>,
    machine_registry: &MachineRegistry,
    hardware: &MachineNewHardwareEthercat<'_, '_, '_>,
    shared_state: Arc<SharedState>,
    socket_queue_tx: Sender<(SocketRef, Arc<control_core::socketio::event::GenericEvent>)>,
) -> Result<(), anyhow::Error> {
    let device_grouping_result = group_devices_by_identification(device_identifications);
    let machine_new_hardware = MachineNewHardware::Ethercat(hardware);

    let mut machines: Vec<Box<dyn Machine>> = vec![];
    let mut machine_objs: Vec<MachineObj> = vec![];

    for device_group in device_grouping_result.device_groups.iter() {
        let machine_identification_unique: MachineIdentificationUnique = match device_group.first()
        {
            Some(device_identification) => device_identification
                .device_machine_identification
                .machine_identification_unique
                .clone(),
            None => continue, // Skip this group if empty
        };

        let new_machine = machine_registry.new_machine(&MachineNewParams {
            device_group,
            hardware: &machine_new_hardware,
            socket_queue_tx: socket_queue_tx.clone(),
            namespace: None,
            main_thread_channel: Some(shared_state.main_channel.clone()),
        });

        match new_machine {
            Ok(machine) => {
                shared_state.clone().api_machines.lock().await.insert(
                    machine_identification_unique.clone(),
                    machine.api_get_sender(),
                );
                machine_objs.push(MachineObj {
                    machine_identification_unique,
                    error: None,
                });
                machines.push(machine);
            }
            Err(e) => machine_objs.push(MachineObj {
                machine_identification_unique,
                error: Some(e.to_string()),
            }),
        }
    }
    let _ = shared_state
        .rt_machine_creation_channel
        .send(crate::app_state::HotThreadMessage::AddMachines(machines))
        .await;

    shared_state.add_machines_if_not_exists(machine_objs).await;
    shared_state.clone().send_machines_event().await;

    Ok(())
}

pub async fn setup_loop(
    interface: &str,
    cycle_target: Duration,
    app_state: Arc<SharedState>,
) -> Result<EthercatSetup, anyhow::Error> {
    tracing::info!("Starting Ethercat PDU loop");
    #[cfg(not(feature = "development-build"))]
    {
        let res = stop_dnsmasq();
        match res {
            Ok(_) => tracing::info!("Stopped dnsmasq"),
            Err(e) => tracing::error!("Failed to stop dnsmasq: {:?}", e),
        };
        // Small Timeout to ensure interfaces get released
        smol::Timer::after(Duration::from_millis(1500)).await;
    }
    
    // Setup ethercrab tx/rx task
    let pdu_storage = Box::leak(Box::new(PduStorage::<MAX_FRAMES, MAX_PDU_DATA>::new()));
    let (tx, rx, pdu) = pdu_storage.try_split().expect("can only split once");
    let interface = interface.to_string();

    std::thread::Builder::new()
        .name("EthercatTxRxThread".to_owned())
        .spawn(move || {
            #[cfg(all(target_os = "linux", not(feature = "development-build")))]
            match set_irq_affinity(&interface, 3) {
                Ok(_) => tracing::info!("ethernet interrupt handler now runs on cpu:{}", 3),
                Err(e) => tracing::error!("set_irq_handler_affinity failed: {:?}", e),
            }

            // Set core affinity to 4th core
            let _ = set_core_affinity(3);

            // Set the thread to real-time priority
            #[cfg(not(feature = "development-build"))]
            let _ = set_realtime_priority();

            #[cfg(not(all(target_os = "linux", feature = "io-uring")))]
            {
                use ethercrab::std::tx_rx_task;

                let rt = smol::LocalExecutor::new();
                let _ = smol::block_on(rt.run(async {
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
            state_transition: Duration::from_millis(10000),
            // Default 30_000us
            pdu: Duration::from_micros(30_000),
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
            retry_behaviour: RetryBehaviour::Count(5),
            // Default 10_000
            dc_static_sync_iterations: 10_000,
        },
    );
    {
        let app_state_clone = app_state.clone();
        let main_namespace = &mut app_state_clone
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;

        let event = EthercatDevicesEventBuilder().initializing();
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
    }

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

    for mut subdevice in group_preop.iter_mut(&maindevice) {
        if subdevice.name() == "EL4002" {
            // Sync mode 01 = SM Synchronous, says it does dc but actually doesnt, thx?
            subdevice
                .sdo_write(0x1c32, 1, 1u16)
                .await
                .expect("Set sync mode");
        }
        // Configure SYNC0 AND SYNC1 for EL4102
        else if subdevice.name() == "EL5152" {
            // Sync mode 02 = SYNC0
            subdevice
                .sdo_write(0x1c32, 1, 2u16)
                .await
                .expect("Set sync mode");

            subdevice
                .sdo_write(0x1c32, 0x02, cycle_target.as_nanos() as u32)
                .await
                .expect("Set cycle time");

            subdevice.set_dc_sync(DcSync::Sync01 {
                // EL4102 ESI specifies SYNC1 with an offset of 100k ns
                sync1_period: Duration::from_nanos(100_000),
            });
        } else {
            match subdevice.dc_support() {
                ethercrab::DcSupport::None => (),
                ethercrab::DcSupport::RefOnly => (),
                ethercrab::DcSupport::Bits64 => subdevice.set_dc_sync(DcSync::Sync0),
                ethercrab::DcSupport::Bits32 => subdevice.set_dc_sync(DcSync::Sync0),
            }
        }
    }
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

    let mut ethercat_meta_devices = app_state.ethercat_meta_data.write().await;
    ethercat_meta_devices.clear();

    // filter devices and if Option<DeviceMachineIdentification> is Some
    // return identified_devices, identified_device_identifications, identified_subdevices
    let (identified_device_identifications,identified_devices, identified_subdevices): (
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
                acc.0.push(identified_device_identification.clone());
                acc.1.push(identified_device);
                acc.2.push(identified_subdevice);
                let meta = EtherCatDeviceMetaData::from_subdevice(identified_subdevice,identified_device_identification);
                ethercat_meta_devices.push(meta);
                acc
            },
        );

    // We always need to have atleast one subdevice anyways
    let coupler = subdevices.get(0).unwrap();
    let _resp = get_most_recent_diagnosis_message(coupler).await;

    /*
        handle special edge cases,
        Wago couplers handle all subdevices as part of the coupler instead of their own devices,
        meaning we need to convert the PDO Mappings to seperate SubDevices to enable more ease of use
        OR alternatively you could show it like TwinCAT with "slots" on the Coupler
    */
    match (coupler.identity().vendor_id, coupler.identity().product_id) {
        (WAGO_750_354_VENDOR_ID, WAGO_750_354_PRODUCT_ID) => {
            let r = Wago750_354::initialize_modules(coupler).await?;
            for module in r {
                if coupler.configured_address() == module.belongs_to_addr {
                    match ethercat_meta_devices.get(0) {
                        Some(meta) => {
                            let meta_data = EtherCatDeviceMetaData {
                                configured_address: module.slot,
                                name:module.name,
                                vendor_id:module.vendor_id,
                                product_id: module.product_id,
                                revision: 0x2,
                                device_identification: DeviceIdentification{
                                    device_machine_identification: meta.device_identification.device_machine_identification.clone(),
                                    device_hardware_identification: machines::machine_identification::DeviceHardwareIdentification::Ethercat(DeviceHardwareIdentificationEthercat{ subdevice_index: module.slot as usize }) }
                            };
                            ethercat_meta_devices.push(meta_data);
                        }
                        None => break,
                    }
                }
            }
        }
        (IP20_EC_DI8_DO8_VENDOR_ID, IP20_EC_DI8_DO8_PRODUCT_ID) => {
            let r = IP20EcDi8Do8::initialize_modules(coupler).await?;
            for module in r {
                if coupler.configured_address() == module.belongs_to_addr {
                    match ethercat_meta_devices.get(0) {
                        Some(meta) => {
                            let meta_data = EtherCatDeviceMetaData {
                                configured_address: module.slot,
                                name:module.name,
                                vendor_id:module.vendor_id,
                                product_id: module.product_id,
                                revision: 0x1,
                                device_identification: DeviceIdentification{
                                    device_machine_identification: meta.device_identification.device_machine_identification.clone(),
                                    device_hardware_identification: machines::machine_identification::DeviceHardwareIdentification::Ethercat(DeviceHardwareIdentificationEthercat{ subdevice_index: module.slot as usize }) }
                            };
                            ethercat_meta_devices.push(meta_data);
                        }
                        None => break,
                    }
                }
            }
        }
        _ => (),
    };
    drop(ethercat_meta_devices);

    // remove subdevice from devices tuple
    let devices = devices
        .iter()
        .map(|(device_identification, device, _)| (device_identification.clone(), device.clone()))
        .collect::<Vec<_>>();

    set_ethercat_devices::<MAX_SUBDEVICES, PDI_LEN>(
        &identified_device_identifications,
        &MACHINE_REGISTRY,
        &MachineNewHardwareEthercat {
            ethercat_devices: &identified_devices,
            subdevices: &identified_subdevices,
        },
        app_state.clone(),
        app_state.clone().socketio_setup.socket_queue_tx.clone(),
    )
    .await?;

    // Avoid potential Sdo write Contention on the bus, by just waiting a bit
    // Timeout is not enough ... or rather a monstrous timeout would be required for all of the sdo writes we do ...
    // TODO: Keep small timeout here and add Complete Access configuration for the stepper terminals and complex trerminals like (el5152)
    smol::Timer::after(Duration::from_millis(4000)).await;

    let mut now = Instant::now();
    let start = Instant::now();
    let mut averages = Vec::new();
    for _ in 0..group_preop.len() {
        averages.push(ExponentialMovingAverage::new(64).unwrap());
    }
    let mut tick_interval = smol::Timer::interval(cycle_target);

    tracing::info!("Moving into PRE-OP with PDI");
    let group = group_preop.into_pre_op_pdi(&maindevice).await?;
    tracing::info!("Done. PDI available. Waiting for SubDevices to align");

    loop {
        group
            .tx_rx_sync_system_time(&maindevice)
            .await
            .expect("TX/RX");

        if now.elapsed() >= Duration::from_millis(25) {
            now = Instant::now();
            let mut max_deviation = 0;
            for (s1, ema) in group.iter(&maindevice).zip(averages.iter_mut()) {
                let diff = match s1
                    .register_read::<u32>(RegisterAddress::DcSystemTimeDifference)
                    .await
                {
                    Ok(value) =>
                    // The returned value is NOT in two's compliment, rather the upper bit specifies
                    // whether the number in the remaining bits is odd or even, so we convert the
                    // value to `i32` using that logic here.
                    {
                        let flag = 0b1u32 << 31;
                        if value >= flag {
                            // Strip off negative flag bit and negate value as normal
                            -((value & !flag) as i32)
                        } else {
                            value as i32
                        }
                    }
                    Err(Error::WorkingCounter { .. }) => 0,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to read DC system time: {:?}", e));
                    }
                };

                let ema_next = ema.next(diff as f64);
                max_deviation = max_deviation.max(ema_next.abs() as u32);
            }
            if max_deviation < 100 {
                tracing::info!("Clocks settled after {} ms", start.elapsed().as_millis());
                break;
            }
        }
        tick_interval.next().await;
    }

    // SubDevice clocks are aligned. We can turn DC on now.
    let group = group
        .configure_dc_sync(
            &maindevice,
            DcConfiguration {
                // Start SYNC0 100ms in the future
                start_delay: Duration::from_millis(100),
                // SYNC0 period should be the same as the process data loop in most cases
                sync0_period: cycle_target,
                // Send process data half way through cycle
                sync0_shift: cycle_target / 2,
            },
        )
        .await?;

    // State machine to handle transition to SafeOp with process data
    enum GroupState {
        PreOp(SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, PreOpPdi, HasDc>),
        SafeOp(SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, SafeOp, HasDc>),
    }

    let mut group_container = Some(GroupState::PreOp(group));
    let mut tick = 0;

    let group = loop {
        let now = Instant::now();
        match group_container.take().unwrap() {
            GroupState::PreOp(group) => {
                let res = group.tx_rx_dc(&maindevice).await.expect("TX/RX");
                if tick > 300 {
                    let group = group.request_into_safe_op(&maindevice).await?;
                    group_container = Some(GroupState::SafeOp(group));
                    tracing::info!("Requested SAFE-OP");
                } else {
                    group_container = Some(GroupState::PreOp(group));
                }

                smol::Timer::at(now + res.extra.next_cycle_wait).await;
            }
            GroupState::SafeOp(group) => {
                let res = group.tx_rx_dc(&maindevice).await.expect("TX/RX");
                if res.all_safe_op() {
                    tracing::info!("SAFE-OP");
                    break group;
                } else {
                    group_container = Some(GroupState::SafeOp(group));
                }
                smol::Timer::at(now + res.extra.next_cycle_wait).await;
            }
        }
        tick += 1;
    };

    let group_op = group
        .request_into_op(&maindevice)
        .await
        .expect("SAFE-OP -> OP");
    tracing::info!("Started Transition to OP");

    Ok(EthercatSetup {
        devices,
        group: group_op,
        maindevice,
        all_operational: false,
    })
}
