use std::{
    sync::Arc,
    thread,
    time::Duration,
};

use smol::channel::Sender;
use smol::lock::RwLock;

use control_core::serial::registry::SerialRegistry;

use crate::panic::PanicDetails;
use crate::serial::dre::Dre;
use crate::app_state::AppState;
use crate::panic::send_panic;
use super::serial_detection::SerialDetection;

pub fn init_serial(
    thread_panic_tx: Sender<PanicDetails>,
    _app_state: Arc<AppState>,
    registry: SerialRegistry,
    sd: Arc<RwLock<SerialDetection>>,
) {
    
    let thread_panic_tx_clone = thread_panic_tx.clone();
    let sd_clone = sd.clone();

    thread::Builder::new()
        .name("SerialTxRxThread".to_owned())
        .spawn(move || {
            smol::block_on(async move {
                send_panic("SerialThread", thread_panic_tx_clone);

                let rt = smol::LocalExecutor::new();
                rt.run(async {
                    loop {
                        {
                            let mut sd_locked = sd_clone.write().await;
                            sd_locked.cycle(); // Call cycle on the SerialDetection object

                            println!("SerialRegistry: {:?}", sd_locked.connected_serial_usb.keys());

                            let connected_devices: Vec<_> = sd_locked.connected_serial_usb.iter()
                                .map(|(key, value)| (key.clone(), value))
                                .collect();
                            let mut updates = Vec::new();

                            // Process all connected devices
                            for (key, value) in connected_devices {
                                match value {
                                    Ok(device) => {
                                        match registry.downcast::<Dre>(device.clone()) {
                                            Ok(dre) => {
                                                let mut dre_locked = dre.write().unwrap();
                                                match dre_locked.diameter_request() {
                                                    Ok(diam) => {
                                                        println!("diameter from {} = {}", key, diam);
                                                    }
                                                    Err(e) => {
                                                        println!("Failed diameter_request on {}: {}", key, e);
                                                        updates.push((
                                                            key.clone(),
                                                            Err(anyhow::anyhow!(
                                                                "Device {} disconnected: {}",
                                                                key,
                                                                e
                                                            )),
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!(
                                                    "Error initializing DRE device on port {}: {}",
                                                    key, e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error on port {}: {}", key, e);
                                    }
                                }
                            }

                            // Update the connected_serial_usb map with any new error states
                            for (key, value) in updates {
                                sd_locked.connected_serial_usb.insert(key, value);
                            }
                        }

                        // Sleep for a while to avoid excessive CPU usage
                        thread::sleep(Duration::from_millis(300));
                    }
                })
                .await;
            });
        })
        .expect("Failed to spawn SerialTxRxThread");

}
