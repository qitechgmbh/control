use crate::app_state::EthercatSetup;
use crate::ethercat::config::{MAX_FRAMES, MAX_PDU_DATA};
// use crate::ethercat::device_identification::group_devices;
use crate::{
    app_state::AppState,
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
    socketio::{event::EventData, messages::ethercat_devices_event::EthercatDevicesEvent},
};
use anyhow::Ok;
use ethercrab::std::{ethercat_now, tx_rx_task};
use ethercrab_machines::actor::Actor;
use ethercrab_machines::actors::analog_function_generator::{
    analog_multiply, analog_sine, AnalogFunctionGenerator,
};
use ethercrab_machines::actors::digital_input_logger::DigitalInputLogger;
use ethercrab_machines::actors::stepper_driver_max_speed::StepperDriverMaxSpeed;
use ethercrab_machines::actors::temperature_input_logger::TemperatureInputLogger;
use ethercrab_machines::coe::Configuration;
use ethercrab_machines::device::{devices_from_subdevice_group, get_device, EthercatDevice};
use ethercrab_machines::devices::el1008::{EL1008Port, EL1008};
use ethercrab_machines::devices::el2008::{EL2008Port, EL2008};
use ethercrab_machines::devices::el2521::{
    EL2521Configuration, EL2521OperatingMode, EL2521PdoAssignment,
};
use ethercrab_machines::devices::el3204::{EL3204Port, EL3204};
use ethercrab_machines::devices::el4008::{EL4008Port, EL4008};
use ethercrab_machines::io::analog_output::AnalogOutput;
use ethercrab_machines::io::digital_input::DigitalInput;
use ethercrab_machines::io::digital_output::DigitalOutput;
use ethercrab_machines::io::temperature_input::TemperatureInput;
use ethercrab_machines::utils::traits::ArcRwLock;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts};

pub async fn setup_loop(interface: &str, app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    log::info!("Setting up Ethercat network");

    // erase all all setup data
    {
        let mut ethercat_setup_guard = app_state.ethercat_setup.write().await;
        *ethercat_setup_guard = None;
    }

    // setup ethercrab tx/rx task
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

    // create maindevice
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

    // notify client via socketio
    tokio::spawn(async {
        EthercatDevicesEvent::build_warning("Configuring Devices...".to_string())
            .emit("main")
            .await
    });

    // initialize all subdevices
    // Fails if DC setup detects a mispatching working copunter, then just try again in loop
    let group = loop {
        let group = maindevice
            .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
            .await;
        // if ok return
        if group.is_ok() {
            break group.unwrap();
        }
    };

    log::info!("Initialized {} subdevices", group.len());

    // let subdevice = group.subdevice(&maindevice, 2)?;
    // subdevice.eeprom().write(0x0028, 0x5678).await?;

    // let group_vec: Vec<_> = group.iter(&maindevice).collect();
    // let groups = group_devices(&group_vec).await?;

    // for machine in groups.0 {
    //     log::info!("Machine: {}", machine.machine_identification);
    //     for subdevice in machine.devices {
    //         log::info!(
    //             "-> Device {}",
    //             group.subdevice(&maindevice, subdevice.0)?.name()
    //         );
    //     }
    // }
    // for (i, subdevice) in groups.1 {
    //     log::warn!(
    //         "Unidentified subdevice: config: {}, alias: {}",
    //         i,
    //         group.subdevice(&maindevice, i)?.name()
    //     );
    // }

    let subdevice_el2521 = group.subdevice(&maindevice, 5)?;
    log::info!(
        "Subdevice: config: {:x}, alias: {} name: {}",
        subdevice_el2521.configured_address(),
        subdevice_el2521.alias_address(),
        subdevice_el2521.name()
    );

    let config = EL2521Configuration {
        operating_mode: EL2521OperatingMode::PulseDirectionSpecification,
        direct_input_mode: true,
        ..EL2521Configuration::default()
    };

    config.write_to(&subdevice_el2521).await?;

    // for subdevice in group.iter(&maindevice) {
    //     log::info!(
    //         "Subdevice: config: {}, alias: {}",
    //         subdevice.configured_address(),
    //         subdevice.alias_address()
    //     );
    // }
    // seleep a million seconds

    // put group in op state
    let mut group_op = group
        .into_op(&maindevice)
        .await
        .expect("Failed to put group in OP state");

    let propagation_delays: Vec<u32> = group_op
        .iter(&maindevice)
        .map(|subdevice| subdevice.propagation_delay())
        .collect();

    // create devices
    let devices: Vec<Option<Arc<RwLock<dyn EthercatDevice>>>> =
        devices_from_subdevice_group(&mut group_op, &maindevice);

    let actors: Vec<Arc<RwLock<dyn Actor>>> = vec![
        // StepperDriverMaxSpeed::new(DigitalOutput::new(
        //     get_device::<EL2008>(&devices, 2).await?,
        //     EL2008Port::DO1,
        // ))
        // .to_arc_rwlock(),
        // DigitalInputLogger::new(DigitalInput::new(
        //     get_device::<EL1008>(&devices, 1).await?,
        //     EL1008Port::DI2,
        // ))
        // .to_arc_rwlock(),
        // AnalogFunctionGenerator::new(
        //     AnalogOutput::new(get_device::<EL4008>(&devices, 3).await?, EL4008Port::AO1),
        //     analog_multiply([
        //         analog_sine(1.0, 0.0, Duration::from_secs(1).as_nanos() as u64),
        //         analog_sine(1.0, 0.0, Duration::from_millis(50).as_nanos() as u64),
        //     ]),
        // )
        // .to_arc_rwlock(),
        // TemperatureInputLogger::new(
        //     // )
        //     TemperatureInput::new(get_device::<EL3204>(&devices, 4).await?, EL3204Port::T1),
        // )
        // .to_arc_rwlock(),
    ];

    // set all setup data
    {
        let mut ethercat_setup_guard = app_state.ethercat_setup.write().await;
        *ethercat_setup_guard = Some(EthercatSetup {
            maindevice,
            group: group_op,
            devices,
            actors,
            delays: propagation_delays.into_iter().map(Some).collect(),
        });
    }

    // notify client via socketio
    // tokio::spawn(async { EthercatDevicesEvent::build().await.emit("main").await });

    log::info!("Starting contorl loop");
    let pdu_handle = tokio::spawn(async move {
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
    log::info!("TX/RX cycle");
    setup.group.tx_rx(&setup.maindevice).await?;
    log::info!("TX/RX cycle done");

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
        device.input_checked(input.as_ref())?;
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
        device.output_checked(output.as_mut())?;
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
