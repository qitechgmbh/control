use crate::laser::api::{LiveValuesEvent as LaserLiveValues, StateEvent as LaserState};

#[derive(Debug, Clone)]
pub enum MachineData
{
    Laser(LaserState, LaserLiveValues),
    None,
}