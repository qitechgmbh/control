pub fn init_journald_tracing<S>() -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    Box::new(tracing_journald::layer().expect("Failed to create journald layer"))
}
