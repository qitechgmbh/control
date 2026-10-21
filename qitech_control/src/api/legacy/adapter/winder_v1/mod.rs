use crate::api::legacy::MachineLegacyDataAdapter;

mod request;
use request::convert_request;

mod measurements;
use measurements::init_measurements_event;

mod state;
use state::init_state_event;

pub const ADAPTER: MachineLegacyDataAdapter = MachineLegacyDataAdapter {
    convert_request,
    init_state_event,
    init_measurements_event,
};
