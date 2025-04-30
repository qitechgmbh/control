/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 29.04.2025
*@last_update: 30.04.2025
*@description: This module is responsible for serial devices detection and validation
*/


use std::str::FromStr;

use crate::app_state::APP_STATE;
use control_core::socketio::namespace_id::NamespaceId;

use super::usb_detection::*;
use super::config::*;

pub async fn init_serial(
    thread_panic_tx: Sender<PanicDetails>,
    app_state: Arc<AppState>,
    &mut shared_variable: Arc<Mutex<Vec<SerialPortInfo>>>) {

    let writer_var = Arc::clone(&shared_variable);
    let thread_panic_tx_clone = thread_panic_tx.clone();

    // Spawn a new thread to run the smol executor and handle async tasks
    std::thread::Builder::new()
    .name("EthercatTxRxThread".to_owned())
    .spawn(move || {
        smol::block_on(async {
            send_panic("SerialTxRxThread", thread_panic_tx_clone);

            // Create the LocalExecutor instance
            let rt = smol::LocalExecutor::new();
            rt.spawn(async move {
                detection(writer_var).await;
            }).detach();
            
            // Start the init_serial task to keep updating the shared variable
            rt.run(async {
                loop {
                    {
                        let data = shared_variable.lock().unwrap();
                        println!("Ports: {:?}", *data); // This will print the updated ports
                    }
                    Timer::after(Duration::from_secs(1)).await; // Adding a delay to avoid tight loop
                }
            }).await;
        });
    })
    .expect("Building thread failed");
    // Keep the main thread alive so the spawned thread can keep running
    loop {
        // You can add any additional logic here
        std::thread::sleep(Duration::from_secs(1));
    }
}

/* @param: shared_variable -> Arc<Mutex<HashMap<String, SerialPortInfo>>> which is shared between threads and 
 * is used to store the list of available ports
 * 
 * @description: This function is used to update the list of available ports every second
 */
pub async fn detection(shared_variable: Arc<Mutex<HashMap<String, SerialPortInfo>>>) {
    loop{
        {
        println!("Updating ports...");
        let mut var = shared_variable.lock().unwrap();
        validate_usb(
            &mut *var,
            update(),
            config::VID,
            config::PID
        );
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}











fn setup_namespace(socket: SocketRef) {
    let namespace_id = match NamespaceId::from_str(socket.ns()) {
        Ok(namespace_id) => namespace_id,
        Err(err) => {
            log::error!(
                "[{}::setup_namespace] Failed to parse namespace id: {}",
                module_path!(),
                err
            );
            return;
        }
    };

    // Set up disconnect handler
    setup_disconnection(&socket, namespace_id.clone());

    // Set up connection
    smol::block_on(setup_connection(socket, namespace_id));
}

fn setup_disconnection(socket: &SocketRef, namepsace_id: NamespaceId) {
    socket.on_disconnect(|socket: SocketRef| {
        smol::block_on(async move {
            log::debug!("Socket disconnected {}", socket.id);
            let mut socketio_namespaces_guard = APP_STATE.socketio_setup.namespaces.write().await;

            // remove from machine namespace
            socketio_namespaces_guard
                .apply_mut(
                    namepsace_id.clone(),
                    |namespace_interface| match namespace_interface {
                        Ok(namespace_interface) => {
                            namespace_interface.unsubscribe(socket.clone());
                        }
                        Err(err) => {
                            log::error!(
                                "[{}::on_disconnect_machine_ns] Namespace {:?} not found: {}",
                                module_path!(),
                                namepsace_id,
                                err
                            );
                        }
                    },
                )
                .await;
        });
    });
}

async fn setup_connection(socket: SocketRef, namespace_id: NamespaceId) {
    log::info!("Socket connected {}", socket.id);
    let mut socketio_namespaces_guard = APP_STATE.socketio_setup.namespaces.write().await;
    socketio_namespaces_guard
        .apply_mut(
            namespace_id.clone(),
            |namespace_interface| match namespace_interface {
                Ok(namespace_interface) => {
                    namespace_interface.subscribe(socket.clone());
                    namespace_interface.reemit(socket);
                }
                Err(err) => {
                    log::error!(
                        "[{}::on_connect_machine_ns] Namespace {:?} not found: {}",
                        module_path!(),
                        namespace_id,
                        err
                    );
                }
            },
        )
        .await;
}
