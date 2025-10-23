use control_core::{
    machines::identification::MachineIdentificationUnique,
    serial::{
        SerialDeviceIdentification, SerialDeviceNew, SerialDeviceNewParams,
        serial_detection::SerialDetection,
    },
    socketio::namespace::NamespaceCacheingLogic,
};
use serialport::UsbPortInfo;
use smol::Task;
use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    app_state::AppState,
    machines::{laser::LaserMachine, registry::MACHINE_REGISTRY},
    socketio::main_namespace::{MainNamespaceEvents, machines_event::MachinesEventBuilder},
};

use super::devices::laser::Laser;

// Async function that runs until at least one serial device is found.
pub async fn find_serial() -> Option<HashMap<String, UsbPortInfo>> {
    smol::Timer::after(Duration::from_secs(1)).await;
    let devices = SerialDetection::detect_devices();
    if !devices.is_empty() {
        return Some(devices);
    } else {
        return None;
    }
}

// Returns a smol::Task that resolves when atleast one device is found.
pub fn start_serial_discovery() -> Task<Option<HashMap<String, UsbPortInfo>>> {
    smol::spawn(find_serial())
}

// This function handles a disconnection and or reconnection of Laser
// You can easily rewrite it when we have more serialdevices
// TODO: Add A static list of known SeriadeviceIdents
pub async fn handle_serial_device_hotplug(
    app_state: Arc<AppState>,
    map: Option<HashMap<String, UsbPortInfo>>,
) {
    let laser_ident = SerialDeviceIdentification {
        vendor_id: 0x0403,
        product_id: 0x6001,
    };

    let laser = match map {
        Some(map) => SerialDetection::get_path_by_id(laser_ident, map),
        None => None,
    };

    let mut unique_ident: Option<MachineIdentificationUnique> = None;

    {
        let machines = app_state.machines.read_arc_blocking();
        for (id, _) in machines.iter() {
            if id.machine_identification == LaserMachine::MACHINE_IDENTIFICATION {
                unique_ident = Some(id.clone());
                break;
            }
        }
    }

    // Machine isnt connected, so add it back
    if laser.is_some() && unique_ident.is_none() {
        let serial_params = SerialDeviceNewParams {
            path: laser.unwrap(),
        };
        let _ = match Laser::new_serial(&serial_params) {
            Ok((device_identification, serial_device)) => {
                {
                    let mut machines = app_state.machines.write().await;
                    machines.add_serial_device(
                        &device_identification,
                        serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                        Arc::downgrade(&app_state.machines),
                    );
                }

                let app_state_event = app_state.clone();
                let main_namespace = &mut app_state_event
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;
                let event = MachinesEventBuilder().build(app_state_event.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
            }
            _ => (),
        };
    } else if laser.is_none() && unique_ident.is_some() {
        tracing::info!("hello non existing laser");
        let serial_params = SerialDeviceNewParams {
            path: "".to_string(),
        };

        let _ = match Laser::new_serial(&serial_params) {
            Ok((device_identification, _)) => {
                app_state
                    .machines
                    .write()
                    .await
                    .remove_serial_device(&device_identification);
                drop(app_state.machines.clone());

                app_state
                    .machines
                    .write_arc()
                    .await
                    .serial_machines
                    .remove(&unique_ident.clone().unwrap());

                app_state
                    .machines
                    .write_arc()
                    .await
                    .ethercat_machines
                    .remove(&unique_ident.clone().unwrap());

                drop(app_state.machines.clone());

                let main_namespace = &mut app_state
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;

                tracing::info!("{:?}", app_state.machines);
                let event = MachinesEventBuilder().build(app_state.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
            }
            _ => (),
        };
    }
}
