#![allow(unused_imports)]

use crate::app_state::EthercatSetup;
use crate::ethercat::device_identification::identify_device_groups;
use crate::machines::Machines;
use crate::{
    app_state::AppState,
    ethercat::config::{MAX_FRAMES, MAX_PDU_DATA, MAX_SUBDEVICES, PDI_LEN},
    socketio::{event::EventData, events::ethercat_devices_event::EthercatDevicesEvent},
};
use bitvec::prelude::*;
use ethercat_hal::actors::analog_function_generator::{
    analog_multiply, analog_sine, AnalogFunctionGenerator,
};
use ethercat_hal::actors::digital_input_logger::DigitalInputLogger;
use ethercat_hal::actors::stepper_driver_max_speed::StepperDriverMaxSpeed;
use ethercat_hal::actors::stepper_driver_pulse_train::StepperDriverPulseTrain;
use ethercat_hal::actors::temperature_input_logger::TemperatureInputLogger;
use ethercat_hal::actors::Actor;
use ethercat_hal::coe::Configuration;
use ethercat_hal::devices::el2521::{EL2521Configuration, EL2521OperatingMode, EL2521Port, EL2521};
use ethercat_hal::devices::{devices_from_subdevices, specific_device_from_devices, Device};
use ethercat_hal::io::analog_output::AnalogOutput;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::pulse_train_output::PulseTrainOutput;
use ethercat_hal::io::temperature_input::TemperatureInput;
use ethercat_hal::types::{
    EthercrabSubDeviceGroupOperational, EthercrabSubDeviceGroupPreoperational,
};
use ethercrab::std::{ethercat_now, tx_rx_task};
use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

pub async fn setup_loop(interface: &str, app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
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
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let _ = rt.block_on(async move {
            tx_rx_task(&interface, tx, rx)
                .expect("spawn TX/RX task")
                .await
        });
    });

    // Create maindevice
    let maindevice = MainDevice::new(
        pdu,
        Timeouts {
            wait_loop_delay: Duration::from_millis(1),
            mailbox_response: Duration::from_millis(100000000),
            ..Default::default()
        },
        MainDeviceConfig {
            retry_behaviour: RetryBehaviour::Forever,
            ..Default::default()
        },
    );

    // Notify client via socketio
    tokio::spawn(async {
        EthercatDevicesEvent::build_warning("Configuring Devices...".to_string())
            .emit("main")
            .await
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
            "Failed to initialize subdevices: {:?}",
            err
        ))?,
    };

    // create devices
    let devices: Vec<Option<Arc<RwLock<dyn Device>>>> =
        devices_from_subdevices::<MAX_SUBDEVICES, PDI_LEN>(&mut group_preop, &maindevice);

    // Identify machines
    // - Read identification values from devices
    let (identified_device_groups, unidentified_devices) =
        identify_device_groups(&group_preop, &maindevice).await?;
    // - Create Machines
    let subdevices = group_preop.iter(&maindevice).collect::<Vec<_>>();
    let machines = identified_device_groups
        .iter()
        .map(|identified_device_group| {
            Machines::new(identified_device_group, &subdevices, &devices)
        })
        .collect::<Vec<_>>();

    // Put group in operational state
    let group_op = match group_preop.into_op(&maindevice).await {
        Ok(group_op) => {
            log::info!("Group in OP state");
            group_op
        }
        Err(err) => Err(anyhow::anyhow!(
            "Failed to put group in OP state: {:?}",
            err
        ))?,
    };

    // Get propagation delays
    let propagation_delays: Vec<u32> = group_op
        .iter(&maindevice)
        .map(|subdevice| subdevice.propagation_delay())
        .collect();

    // create actors
    // push all machines into the actors vector
    let mut actors: Vec<Arc<RwLock<dyn Actor>>> = Vec::new();
    for machine in machines {
        if let Ok(machine) = machine {
            actors.push(Arc::new(RwLock::new(machine)));
        }
    }

    // Write all this stuff to `app_state`
    {
        let mut ethercat_setup_guard = app_state.ethercat_setup.write().await;
        *ethercat_setup_guard = Some(EthercatSetup {
            maindevice,
            group: group_op,
            devices,
            identified_device_groups,
            unidentified_devices,
            actors,
            delays: propagation_delays.into_iter().map(Some).collect(),
        });
    }

    // Notify client via socketio
    tokio::spawn(async { EthercatDevicesEvent::build().await.emit("main").await });

    // Start control loop
    let pdu_handle = tokio::spawn(async move {
        log::info!("Starting control loop");
        let mut average_nanos = Duration::from_micros(250).as_nanos() as u64;
        loop {
            let res = loop_once(app_state.ethercat_setup.clone(), &mut average_nanos).await;
            if let Err(err) = res {
                log::error!("Loop failed: {:?}", err);
            }
        }
    });
    // Await the pdu_loop task so that it executes fully
    pdu_handle.await.expect("pdu_loop task failed");

    Ok(())
}

pub async fn loop_once<'maindevice>(
    setup: Arc<RwLock<Option<EthercatSetup>>>,
    average_nanos: &mut u64,
) -> Result<(), anyhow::Error> {
    let setup_guard = setup.read().await;
    let setup = setup_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No setup"))?;

    // TS when the TX/RX cycle starts
    let input_ts = ethercat_now();

    // TX/RX cycle
    setup.group.tx_rx(&setup.maindevice).await?;

    // Prediction when the next TX/RX cycle starts
    let output_ts = input_ts + *average_nanos;

    // copy inputs to devices
    for (i, subdevice) in setup.group.iter(&setup.maindevice).enumerate() {
        let mut device = match setup.devices[i].as_ref() {
            Some(device) => device.write().await,
            None => continue,
        };
        device.ts(input_ts, output_ts);
        let input = subdevice.inputs_raw();
        let input_bits = input.view_bits::<Lsb0>();
        device.input_checked(input_bits)?;
    }

    // execute actors
    for actor in setup.actors.iter() {
        let mut actor = actor.write().await;
        actor.act(output_ts).await;
    }

    // copy outputs from devices
    for (i, subdevice) in setup.group.iter(&setup.maindevice).enumerate() {
        let device = match setup.devices[i].as_ref() {
            Some(device) => device.read().await,
            None => continue,
        };
        let mut output = subdevice.outputs_raw_mut();
        let output_bits = output.view_bits_mut::<Lsb0>();
        device.output_checked(output_bits)?;
    }
    Ok(())
}
