use new::MachineNewTrait;
use std::any::Any;
use std::fmt::Debug;

use crate::machines::{
    api::MachineApi, identification::MachineIdentificationUnique, new::MachineAct,
};
pub mod api;
pub mod connection;
pub mod identification;
pub mod manager;
pub mod manager_iter;
pub mod new;
pub mod registry;

pub trait Machine: MachineAct + MachineNewTrait + MachineApi + Any + Debug + Send + Sync {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique;
}
