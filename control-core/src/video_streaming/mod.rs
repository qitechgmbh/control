use anyhow::Error;
use std::collections::HashMap;
use tokio::sync::broadcast;

/// Video stream manager that handles multiple video streams per machine with true broadcasting
/// Uses tokio broadcast channels to ensure all subscribers receive all messages without stealing
#[derive(Debug)]
pub struct VideoStreamManager {
    /// Each stream has its own broadcast transmitter and template receiver
    streams: HashMap<String, (broadcast::Sender<Vec<u8>>, broadcast::Receiver<Vec<u8>>)>,
}

impl VideoStreamManager {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    /// Send a frame to all subscribers of a stream
    /// Returns Ok if the frame was sent, Err if the stream doesn't exist
    pub fn send_frame(&self, stream_id: &str, frame: Vec<u8>) -> Result<(), Error> {
        if let Some((tx, _rx)) = self.streams.get(stream_id) {
            match tx.send(frame) {
                Ok(_) => Ok(()),
                Err(_) => Ok(()), // No receivers, but that's not an error
            }
        } else {
            Err(anyhow::anyhow!("Stream '{}' not found", stream_id))
        }
    }

    /// Register a new video stream with the given ID
    pub fn add_stream(&mut self, stream_id: String) {
        let (tx, rx) = broadcast::channel::<Vec<u8>>(256); // 256 frame buffer capacity
        self.streams.insert(stream_id, (tx, rx));
    }

    /// Get list of available stream IDs
    pub fn get_streams(&self) -> Vec<String> {
        self.streams.keys().cloned().collect()
    }

    /// Check if a stream exists
    pub fn has_stream(&self, stream_id: &str) -> bool {
        self.streams.contains_key(stream_id)
    }

    /// Remove a stream and all its subscribers
    pub fn remove_stream(&mut self, stream_id: &str) -> bool {
        self.streams.remove(stream_id).is_some()
    }

    /// Subscribe to a video stream and get a receiver
    /// Returns the receiver for the new subscription
    pub fn subscribe_to_stream(&self, stream_id: &str) -> Option<broadcast::Receiver<Vec<u8>>> {
        self.streams
            .get(stream_id)
            .map(|(_tx, rx)| rx.resubscribe()) // Resubscribe to create a new independent receiver
    }

    /// Get subscriber count for a stream
    pub fn get_subscriber_count(&self, stream_id: &str) -> usize {
        self.streams
            .get(stream_id)
            .map_or(0, |(tx, _rx)| tx.receiver_count())
    }

    /// Get a stream transmitter for sending frames (used by machines to send video)
    /// Returns None if the stream doesn't exist
    pub fn get_sender(&self, stream_id: &str) -> Option<broadcast::Sender<Vec<u8>>> {
        self.streams.get(stream_id).map(|(tx, _rx)| tx.clone())
    }
}

impl Default for VideoStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_video_stream_manager() {
        let mut manager = VideoStreamManager::new();

        // Register streams
        manager.register_stream("camera1".to_string());
        manager.register_stream("camera2".to_string());

        assert_eq!(manager.get_stream_ids().len(), 2);
        assert!(manager.has_stream("camera1"));
        assert!(manager.has_stream("camera2"));
        assert!(!manager.has_stream("camera3"));

        // Subscribe to streams
        let mut rx1 = manager.subscribe_to_stream("camera1").unwrap();
        let mut rx2 = manager.subscribe_to_stream("camera1").unwrap();
        let mut rx3 = manager.subscribe_to_stream("camera2").unwrap();

        assert_eq!(manager.get_subscriber_count("camera1"), 3); // Original + 2 clones
        assert_eq!(manager.get_subscriber_count("camera2"), 2); // Original + 1 clone
        assert_eq!(manager.get_subscriber_count("nonexistent"), 0);

        // Send frames
        let frame1 = vec![1, 2, 3, 4];
        let frame2 = vec![5, 6, 7, 8];

        assert!(manager.send_single_frame("camera1", frame1.clone()));
        assert!(manager.send_single_frame("camera2", frame2.clone()));
        assert!(!manager.send_single_frame("nonexistent", vec![]));

        // Both camera1 subscribers should receive the frame
        assert_eq!(rx1.recv().await.unwrap(), frame1);
        assert_eq!(rx2.recv().await.unwrap(), frame1);

        // Camera2 subscriber should receive its frame
        assert_eq!(rx3.recv().await.unwrap(), frame2);
    }

    #[tokio::test]
    async fn test_video_frame_broadcasting() {
        let mut manager = VideoStreamManager::new();
        manager.register_stream("main_camera".to_string());

        // Multiple clients subscribe to the same camera
        let mut rx1 = manager.subscribe_to_stream("main_camera").unwrap();
        let mut rx2 = manager.subscribe_to_stream("main_camera").unwrap();
        let mut rx3 = manager.subscribe_to_stream("main_camera").unwrap();

        // Simulate video frames
        let frames = vec![
            vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG header
            vec![0x00, 0x01, 0x02, 0x03], // Frame 1
            vec![0x04, 0x05, 0x06, 0x07], // Frame 2
        ];

        for frame in &frames {
            assert!(manager.send_single_frame("main_camera", frame.clone()));
        }

        // All clients should receive all frames in order
        for frame in &frames {
            assert_eq!(rx1.recv().await.unwrap(), *frame);
            assert_eq!(rx2.recv().await.unwrap(), *frame);
            assert_eq!(rx3.recv().await.unwrap(), *frame);
        }
    }

    #[test]
    fn test_stream_management() {
        let mut manager = VideoStreamManager::new();

        // Initially no streams
        assert_eq!(manager.get_stream_ids().len(), 0);

        // Add streams
        manager.register_stream("front_camera".to_string());
        manager.register_stream("rear_camera".to_string());
        manager.register_stream("side_camera".to_string());

        let mut stream_ids = manager.get_stream_ids();
        stream_ids.sort(); // Sort for consistent testing
        assert_eq!(
            stream_ids,
            vec!["front_camera", "rear_camera", "side_camera"]
        );

        // Remove a stream
        assert!(manager.remove_stream("rear_camera"));
        assert!(!manager.remove_stream("rear_camera")); // Already removed
        assert!(!manager.has_stream("rear_camera"));

        assert_eq!(manager.get_stream_ids().len(), 2);
    }

    #[tokio::test]
    async fn test_send_frame_results() {
        let mut manager = VideoStreamManager::new();
        manager.register_stream("test_camera".to_string());

        // One initial receiver (the original one created in register_stream)
        let result = manager.send_frame("test_camera", vec![1, 2, 3]).unwrap();
        assert_eq!(result, 1); // Number of subscribers

        // Add subscribers by cloning the receiver
        let _rx1 = manager.subscribe_to_stream("test_camera").unwrap();
        let _rx2 = manager.subscribe_to_stream("test_camera").unwrap();

        // Send frame to multiple subscribers (original + 2 clones = 3 total)
        let result = manager.send_frame("test_camera", vec![4, 5, 6]).unwrap();
        assert_eq!(result, 3); // Number of subscribers

        // Try sending to non-existent stream
        let result = manager.send_frame("nonexistent", vec![7, 8, 9]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Stream 'nonexistent' not found");
    }

    #[test]
    fn test_default_video_stream_manager() {
        let manager = VideoStreamManager::default();
        assert_eq!(manager.get_stream_ids().len(), 0);
    }
}
