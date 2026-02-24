use crate::laser::api::LiveValuesEvent as LaserLiveValues;

#[derive(Debug, Clone)]
pub enum LiveValues {
    Laser(LaserLiveValues),
}
