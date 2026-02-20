use control_core_derive::BuildEvent;
use serde::{ Deserialize, Serialize };

use super::DerivedMachine;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {}

#[derive(Serialize, Debug, Clone, PartialEq, BuildEvent)]
pub struct StateEvent 
{
    pub is_default_state: bool,
}

#[derive(Deserialize, Serialize)]
pub enum Mutation {}

impl DerivedMachine
{
    pub fn get_live_values(&self) -> LiveValuesEvent
    {
        LiveValuesEvent {}
    }

    pub fn get_state(&self) -> StateEvent
    {
        StateEvent { is_default_state: true }
    }

    pub fn mutate(&mut self, mutation: Mutation) 
    {
        match mutation {}
    }
}