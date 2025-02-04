use crate::app_state::EthercatSetup;
use crate::ethercat::config::{MAX_FRAMES, MAX_PDU_DATA};
use crate::ethercat_drivers::devices::el2008::EL2008Port;
use crate::ethercat_drivers::io::analog_output::AnalogOutput;
use crate::ethercat_drivers::io::digital_input::DigitalInput;
use crate::ethercat_drivers::io::digital_output::DigitalOutput;
use crate::ethercat_drivers::io::temperature_input::TemperatureInput;
use crate::{
    app_state::AppState,
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
    ethercat_drivers::{
        actor::Actor,
        actors::{
            analog_function_generator::{analog_multiply, analog_sine, AnalogFunctionGenerator},
            digital_input_logger::DigitalInputLogger,
            stepper_driver_max_speed::StepperDriverMaxSpeed,
            temperature_input_logger::TemperatureInputLogger,
        },
        device::{devices_from_subdevice_group, get_device, EthercatDevice},
        devices::{
            el1008::{EL1008Port, EL1008},
            el2008::EL2008,
            el3204::{EL3204Port, EL3204},
            el4008::{EL4008Port, EL4008},
        },
        utils::traits::ArcRwLock,
    },
    socketio::{event::EventData, messages::ethercat_devices_event::EthercatDevicesEvent},
};
use anyhow::Ok;
use ethercrab::std::{ethercat_now, tx_rx_task};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts};

pub async fn setup_loop(interface: &str, app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    log::info!("Launching PDU loop");
    // erase all all setup data
    {
        let mut ethercat_setup_guard = app_state.ethercat_setup.write().await;
        *ethercat_setup_guard = None;
    }

    // setup ethercrab
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

    log::info!("Ethercat setup started");

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

    log::info!("Main device created");

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
    // subdevice.write_alias_address(0x1ced).await?;

    // for subdevice in group.iter(&maindevice) {
    //     log::info!(
    //         "Subdevice: config: {}, alias: {}",
    //         subdevice.configured_address(),
    //         subdevice.alias_address()
    //     );
    // }
    // seleep a million seconds

    // put group in op state
    let group_op = group.into_op(&maindevice).await?;

    let propagation_delays: Vec<u32> = group_op
        .iter(&maindevice)
        .map(|subdevice| subdevice.propagation_delay())
        .collect();

    // create devices
    let devices: Vec<Option<Arc<RwLock<dyn EthercatDevice>>>> =
        devices_from_subdevice_group(&group_op, &maindevice);

    let actors: Vec<Arc<RwLock<dyn Actor>>> = vec![
        StepperDriverMaxSpeed::new(DigitalOutput::new(
            get_device::<EL2008>(&devices, 2).await?,
            EL2008Port::DO1,
        ))
        .to_arc_rwlock(),
        DigitalInputLogger::new(DigitalInput::new(
            get_device::<EL1008>(&devices, 1).await?,
            EL1008Port::DI2,
        ))
        .to_arc_rwlock(),
        AnalogFunctionGenerator::new(
            AnalogOutput::new(get_device::<EL4008>(&devices, 3).await?, EL4008Port::AO1),
            analog_multiply([
                analog_sine(1.0, 0.0, Duration::from_secs(1).as_nanos() as u64),
                analog_sine(1.0, 0.0, Duration::from_millis(50).as_nanos() as u64),
            ]),
        )
        .to_arc_rwlock(),
        TemperatureInputLogger::new(
            // )
            TemperatureInput::new(get_device::<EL3204>(&devices, 4).await?, EL3204Port::T1),
        )
        .to_arc_rwlock(),
    ];

    // set all setup data
    log::info!("Setting up EthercatSetup");
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
    log::info!("EthercatSetup set");

    // notify client via socketio
    // tokio::spawn(async { EthercatDevicesEvent::build().await.emit("main").await });

    let pdu_handle =
        tokio::spawn(async move { multi_loop(app_state.ethercat_setup.clone()).await });
    // Await the pdu_loop task so that it executes fully
    pdu_handle.await.expect("pdu_loop task failed");

    Ok(())
}

use resonant_scheduler::scheduler::ResonantScheduler;
use resonant_scheduler::signal::Signal;
use tokio::sync::Mutex;

pub async fn multi_loop(ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>) {
    let interval = Duration::from_nanos(1);
    let mutex = Arc::new(Mutex::new(()));
    let setup = ethercat_setup.clone();

    let cycle_fn = move || {
        let mutex = mutex.clone();
        let interval = interval.clone();
        let setup = setup.clone();
        async move {
            log::info!("cycle_fn");
            let mutex = mutex.clone();
            let interval = interval.clone();
            let setup = setup.clone();

            // let ts_1 = ethercat_now();
            // up to 16 of these can run in parallel
            {
                let setup = setup.read().await;
                let _ = match setup.as_ref() {
                    None => {
                        log::warn!("No Ethercat setup available, waiting for setup");
                        return Signal::Continue;
                    }
                    Some(setup) => {
                        log::info!("before txrx");
                        let _ = setup.group.tx_rx(&setup.maindevice).await;
                        log::info!("after txrx");
                    }
                };
            }
            // one of these can run at a time
            {
                let _guard = mutex.lock().await;
                let guard = setup.read().await;
                let setup = match guard.as_ref() {
                    None => return Signal::Continue,
                    Some(setup) => setup,
                };
                let _ = loop_once(setup, interval).await;
            }
            // let ts_2 = ethercat_now();
            // log::trace!("PDU cycle took {} ns", ts_2 - ts_1);
            Signal::Continue
        }
    };

    let runner = ResonantScheduler::new(1, Duration::from_micros(250), 0.1);
    log::info!("running runner.run(cycle_fn).await;");
    runner.run(cycle_fn).await.unwrap();
}

pub async fn loop_once<'maindevice>(
    setup: &EthercatSetup,
    interval: Duration,
) -> Result<(), anyhow::Error> {
    // TS when the TX/RX cycle starts
    let input_ts = ethercat_now();

    // Prediction when the next TX/RX cycle starts
    let output_ts = input_ts + interval.as_nanos() as u64;

    // copy inputs to devices
    for (i, subdevice) in setup.group.iter(&setup.maindevice).enumerate() {
        let mut device = match setup.devices[i].as_ref() {
            Some(device) => device.write().await,
            None => continue,
        };
        let input_ts = input_ts;
        let output_ts = output_ts;
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

    Ok(())
}
