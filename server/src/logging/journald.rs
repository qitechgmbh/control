pub fn init_journald_tracing<S>() -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    let journald_layer = tracing_journald::layer()
        .expect("Failed to create journald layer")
        // Include file and line number information
        // Example: CODE_FILE=src/main.rs, CODE_LINE=42
        .with_file(true)
        .with_line_number(true)
        // Include target (module path) information
        // Example: TARGET=control_server::ethercat::master (shows exact module path)
        .with_target(true)
        // Include thread names for better debugging
        // Example: THREAD_NAME=tokio-runtime-worker
        .with_thread_names(true)
        .with_thread_ids(false)
        // Include the level as a structured field
        // Example: PRIORITY=4 (for WARN), PRIORITY=6 (for INFO), etc.
        .with_level(true);

    Box::new(journald_layer)
}
