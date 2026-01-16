use crate::app_state::SharedState;
use machines::{
    Machine, MachineChannel, machine_identification::MachineIdentificationUnique,
    wago_power::WagoPower,
};
use std::sync::Arc;

#[cfg(not(feature = "mock-machine"))]
mod imports {
    pub use control_core::ethernet::modbus_tcp_discovery::probe_modbus_tcp;
    pub use control_core::futures::FutureIteratorExt;
    pub use smol::Timer;
    pub use std::time::Duration;
}

#[cfg(not(feature = "mock-machine"))]
use imports::*;

#[cfg(not(feature = "mock-machine"))]
pub async fn start_modbus_tcp_discovery(shared_state: Arc<SharedState>) {
    loop {
        let addresses = probe_modbus_tcp().await;

        if addresses.is_empty() {
            Timer::after(Duration::from_secs(1)).await;
            continue;
        }

        let machines: Vec<Box<dyn Machine>> = addresses
            .into_iter()
            .map(|probe| {
                smol::spawn(async move {
                    let machine_identification_unique = MachineIdentificationUnique {
                        machine_identification: WagoPower::MACHINE_IDENTIFICATION,
                        serial: probe.serial,
                    };

                    let channel = MachineChannel::new(machine_identification_unique);
                    let power = WagoPower::new(channel, probe.addr)
                        .await
                        .expect("Failed to initialize wago power supply");

                    Box::new(power) as Box<dyn Machine>
                })
            })
            .join_all()
            .await;

        shared_state.add_machines(machines).await;
        return;
    }
}

#[cfg(feature = "mock-machine")]
pub async fn start_modbus_tcp_discovery(shared_state: Arc<SharedState>) {
    let machine_identification_unique = MachineIdentificationUnique {
        machine_identification: WagoPower::MACHINE_IDENTIFICATION,
        serial: 0xbeef,
    };

    let channel = MachineChannel::new(machine_identification_unique);
    let power = WagoPower::new(channel)
        .await
        .expect("Failed to initialize wago power supply");

    let machines: Vec<Box<dyn Machine>> = vec![Box::new(power)];

    shared_state.add_machines(machines).await;
}
