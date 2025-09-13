use smol::channel::Sender;

use crate::serial::serial_detection::SerialDeviceRemoval;

/// Device-level panic handler for individual device crashes
/// Sends device identifier and error information for cleanup
/// Used by serial devices and other individual components that need to be removed on panic
pub fn send_serial_device_panic<T>(
    device_identifier: T,
    thread_panic_tx: Sender<SerialDeviceRemoval<T>>,
) where
    T: Send + Sync + Clone + 'static,
{
    // Ensure backtrace is enabled for panics
    if std::env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "full");
        }
    }

    std::panic::set_hook(Box::new(move |panic_info| {
        let thread_name = std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_string();
        let location = panic_info.location().map_or("unknown".to_string(), |loc| {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        });
        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"Box<dyn Any>")
            .to_string();

        let panic_message = format!(
            "Device panicked in thread '{}' at {}: {}",
            thread_name, location, message
        );

        // Send device panic info through channel
        smol::block_on(async {
            let _ = thread_panic_tx
                .send(SerialDeviceRemoval::Error(
                    device_identifier.clone(),
                    anyhow::anyhow!("{}", panic_message),
                ))
                .await;
        });

        // Note: We don't call old_hook to avoid duplicate logging
        // All logging is handled in the main thread when receiving panic details
    }));
}
