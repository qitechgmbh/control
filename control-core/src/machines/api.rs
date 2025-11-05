use serde_json::Value;
use smol::lock::Mutex;
use std::sync::Arc;

use crate::socketio::namespace::Namespace;

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>>;

    /// Read-only query for machine events
    /// Returns the requested events as JSON object with event names as keys
    /// Example: { "State": {...}, "LiveValues": {...} }
    ///
    /// events parameter:
    /// - None: returns all available events (LiveValues and State) with all fields
    /// - Some(vec): returns only the event types listed in the array
    ///   - Contains "LiveValues" = include all LiveValues fields
    ///   - Contains "State" = include all State fields
    ///   - Empty array = no events returned
    ///
    /// Note: Takes &mut self to allow reading from hardware sensors and cached values
    fn api_event(&mut self, events: Option<&Vec<String>>) -> Result<Value, anyhow::Error>;

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
