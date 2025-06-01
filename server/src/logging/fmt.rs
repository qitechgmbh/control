use tracing_subscriber::Layer;

pub fn init_fmt_tracing<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_test_writer()
        .with_target(true);

    // if in debug mode, only show time not date
    #[cfg(debug_assertions)]
    let fmt_layer = fmt_layer.with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
        "%H:%M:%S%.3f".to_string(),
    ));

    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt_layer.with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339());

    Box::new(fmt_layer)
}
