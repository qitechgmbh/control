use crate::actors::Actor;
use api::MachineApi;
use new::MachineNewTrait;
use std::any::Any;
use std::fmt::Debug;
pub mod api;
pub mod identification;
pub mod manager;
pub mod manager_iter;
pub mod new;
pub mod registry;

pub trait Machine: MachineNewTrait + MachineApi + Actor + Any + Debug {}
