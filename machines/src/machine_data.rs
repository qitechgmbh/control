use crate::laser::api::{LiveValuesEvent as LaserLiveValues, StateEvent as LaserState};

#[derive(Debug, Clone)]
pub enum MachinesData
{
    Laser(LaserState, LaserLiveValues),
    None,
}