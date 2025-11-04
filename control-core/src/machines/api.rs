use serde_json::Value;
use smol::lock::Mutex;
use std::sync::Arc;

use crate::rest::mutation::EventFields;
use crate::socketio::namespace::Namespace;

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>>;

    /// Read-only query for machine events
    /// Returns the requested events as JSON object with event names as keys
    /// Example: { "State": {...}, "LiveValues": {...} }
    ///
    /// events parameter:
    /// - None: returns all available events with all fields
    /// - Some(EventFields): returns specified events with specified fields
    ///   - live_values: None = all LiveValues fields, Some([]) = no LiveValues, Some(["field"]) = specific fields
    ///   - state: None = all State fields, Some([]) = no State, Some(["field"]) = specific fields
    ///
    /// Note: Takes &mut self to allow reading from hardware sensors and cached values
    fn api_event(&mut self, events: Option<&EventFields>) -> Result<Value, anyhow::Error>;

    /// Returns a list of available video stream identifiers for this machine
    #[cfg(feature = "video-streaming")]
    fn api_video_streams(&self) -> Vec<String> {
        Vec::new()
    }

    /// Get a video stream receiver for the specified stream ID
    /// Returns None if the stream doesn't exist
    #[cfg(feature = "video-streaming")]
    fn api_get_video_stream(
        &mut self,
        stream_id: &str,
    ) -> Option<tokio::sync::broadcast::Receiver<Vec<u8>>> {
        let _ = stream_id;
        None
    }
}
