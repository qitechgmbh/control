use stahlwerk_extension::{Time, Date, ff01::Entry};

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Zero,
    One(StateOneData),
    Two(StateTwoData),
}

#[derive(Debug)]
pub struct StateOneData {
    pub entry: Entry,
    pub start_date: Date,
    pub from_time:  Time,
}

#[derive(Debug)]
pub struct StateTwoData {
    pub state_one_data: StateOneData,

    pub personnel_id: String,
    pub quantity_scrap: f64,
}