use serde_json::Value;
use smol::lock::Mutex;
use std::sync::Arc;

use crate::socketio::namespace::Namespace;

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>>;

    /// Read-only query for machine state and live data
    /// Returns the current state and live values as JSON, filtered by requested fields
    /// Fields should be in the format: "live_values.field_name" or "state.field_name"
    /// Note: Takes &mut self to allow reading from hardware sensors and cached values
    fn api_query(&mut self, fields: &[String]) -> Result<Value, anyhow::Error>;

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
