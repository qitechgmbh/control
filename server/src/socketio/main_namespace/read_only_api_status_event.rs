use std::sync::Arc;

use control_core::socketio::event::Event;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadOnlyApiStatusEvent {
    pub enabled: bool,
}

pub struct ReadOnlyApiStatusEventBuilder();

impl ReadOnlyApiStatusEventBuilder {
    const NAME: &'static str = "ReadOnlyApiStatusEvent";

    pub fn build(&self, app_state: Arc<AppState>) -> Event<ReadOnlyApiStatusEvent> {
        let enabled = app_state
            .read_only_api_enabled
            .load(std::sync::atomic::Ordering::Relaxed);

        Event::new(Self::NAME, ReadOnlyApiStatusEvent { enabled })
    }
}
