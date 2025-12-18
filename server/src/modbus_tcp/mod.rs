use crate::app_state::SharedState;
use control_core::ethernet::modbus_tcp_discovery::probe_modbus_tcp;
use control_core::futures::FutureIteratorExt;
use machines::{
    MACHINE_WAGO_POWER_V1, Machine, MachineChannel, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
    wago_power::WagoPower,
};
use std::sync::Arc;

#[cfg(not(feature = "mock-machine"))]
pub async fn start_modbus_tcp_discovery(shared_state: Arc<SharedState>) {
    let addresses = probe_modbus_tcp().await;

    let machines: Vec<Box<dyn Machine>> = addresses
        .into_iter()
        .map(|probe| {
            smol::spawn(async move {
                let machine_identification_unique = MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_WAGO_POWER_V1,
                    },
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
}

#[cfg(feature = "mock-machine")]
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
}
