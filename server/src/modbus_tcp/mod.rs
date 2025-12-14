use std::sync::Arc;
use crate::app_state::SharedState;
use machines::{MACHINE_WAGO_POWER_V1, Machine, MachineChannel, VENDOR_QITECH, machine_identification::{MachineIdentification, MachineIdentificationUnique}, wago_power::WagoPower};
use smol::future;

// #[cfg(not(feature = "mock-machine"))]
// pub async fn start_modbus_tcp_discovery(shared_state: Arc<SharedState>) {
//     loop {
//         let addresses = probe_modbus_tcp().await;

//         for addr in addresses.into_iter() {

//         }

//         smol::Timer::after(Duration::from_secs(1)).await;
//     }
// }

// #[cfg(feature = "mock-machine")]
pub async fn start_modbus_tcp_discovery(shared_state: Arc<SharedState>) {
    let machine_identification_unique = MachineIdentificationUnique {
        machine_identification: MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_WAGO_POWER_V1,
        },
        serial: 0xbeef,
    };

    let channel = MachineChannel::new(machine_identification_unique);
    let power = WagoPower::new(channel)
        .await
        .expect("Failed to initialize wago power supply");

    let machines: Vec<Box<dyn Machine>> = vec![Box::new(power)];

    shared_state.add_machines(machines).await;

    loop {
        future::yield_now().await;
    }
}
