use crate::laser::api::LiveValuesEvent as LaserLiveValues;

#[derive(Debug, Clone)]
pub enum MachinesLiveValues {
    Laser(LaserLiveValues),
}
