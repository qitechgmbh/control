pub mod new;
pub mod registry;
pub mod winder1;

use ethercat_hal::actors::Actor;
use new::MachineNewTrait;
use serde_json::Value;
use std::{any::Any, fmt::Debug};

pub trait Machine: MachineNewTrait + MachineApi + Actor + Any + Debug {}

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_subscribe(&self) -> ();
}
