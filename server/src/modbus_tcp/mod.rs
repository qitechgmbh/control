use std::{sync::Arc, time::Duration};
use crate::app_state::{HotThreadMessage, SharedState};
use control_core::ethernet::modbus_tcp_discovery::probe_modbus_tcp;
use machines::{HasMachineChannel, MACHINE_WAGO_POWER_V1, Machine, MachineAct, MachineChannel, Mutatable, VENDOR_QITECH, machine_identification::{MachineIdentification, MachineIdentificationUnique}};
use smol::future;

#[derive(Debug)]
pub struct BasicMachine {
    channel: MachineChannel
}

impl HasMachineChannel for BasicMachine {
    fn get_machine_channel(&self) -> &MachineChannel {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel {
        &mut self.channel
    }
}

impl MachineAct for BasicMachine {

    fn act(&mut self, now: std::time::Instant) {
        println!("Basic Machine Acting");
    }

    fn act_machine_message(&mut self, msg: machines::MachineMessage) {
        println!("API Message Received: {:?}", msg)
    }
}

impl Mutatable for BasicMachine {

    fn mutate(&mut self, mutation: serde_json::Value) -> anyhow::Result<()> {
        println!("Basic Machine Mutation");
        Ok(())
    }
}

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

    let machines: Vec<Box<dyn Machine>> = vec![
        Box::new(BasicMachine {
            channel
        })
    ];

    shared_state.add_machines(machines).await;

    loop {
        future::yield_now().await;
    }
}
