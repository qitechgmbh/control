use crate::app_state::EthercatSetup;
use crate::ethercat::config::{MAX_FRAMES, MAX_PDU_DATA};
use crate::ethercat::mainloop::pdu_loop;
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
        io::{analog_output::AnalogOutputDevice, temperature_input::TemperatureInputDevice},
        utils::traits::ArcRwLock,
    },
    socketio::{event::EventData, messages::ethercat_devices_event::EthercatDevicesEvent},
};
use anyhow::Ok;
use ethercrab::std::{ethercat_now, tx_rx_task};
use parking_lot::RwLock;
use std::thread;
use std::{sync::Arc, time::Duration};

use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts};

pub async fn launch_pdu_loop(
    interface: &str,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    // erase all all setup data
    let ethercat_setup = app_state.ethercat_setup.clone();
    *ethercat_setup.write() = None;

    // setup ethercrab
    let pdu_storage = Box::leak(Box::new(PduStorage::<MAX_FRAMES, MAX_PDU_DATA>::new()));
    let (tx, rx, pdu) = pdu_storage.try_split().expect("can only split once");

    let interface = interface.to_string();
    thread::Builder::new()
        .name("tx_rx_thread".to_string())
        .spawn(move || {
            let tx_rx_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("spawn TX/RX runtime");
            tx_rx_runtime.block_on(tx_rx_task(&interface, tx, rx).expect("spawn TX/RX task"))
        })?;
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

    let subdevice = group.subdevice(&maindevice, 2)?;
    subdevice.write_alias_address(0x1CED);

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
            EL2008Port::DO2,
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
    *ethercat_setup.write() = Some(EthercatSetup {
        maindevice,
        group: group_op,
        devices,
        actors,
        delays: propagation_delays.into_iter().map(Some).collect(),
    });

    // notify client via socketio
    tokio::spawn(async { EthercatDevicesEvent::build().await.emit("main").await });

    // spawn an OS thread for the cycle task
    thread::Builder::new()
        .name("pdu_loop".to_string())
        .spawn(move || {
            let result = pdu_loop(ethercat_setup.clone());
            if let Err(err) = result {
                log::error!("Cycle task failed: {:?}", err);
            }
        })?;

    Ok(())
}
