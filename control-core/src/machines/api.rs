use serde_json::Value;
use smol::lock::Mutex;
use std::sync::Arc;

use crate::socketio::namespace::Namespace;

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>>;

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
