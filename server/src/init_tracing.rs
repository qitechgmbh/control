use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

pub fn init_tracing() {
    let registry = tracing_subscriber::registry();
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug,axum=debug"));

    // Check if QITECH_OS environment variable is set
    let layer: Box<dyn Layer<_> + Send + Sync> = if std::env::var("QITECH_OS").is_ok() {
        let journald_layer = init_journald_tracing();
        Box::new(journald_layer)
    } else {
        let fmt_layer = init_fmt_tracing();
        Box::new(fmt_layer)
    };

    let subscriber = registry.with(layer).with(env_filter);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}

fn init_journald_tracing() -> tracing_journald::Layer {
    tracing_journald::layer().expect("Failed to create journald layer")
}

fn init_fmt_tracing() -> impl Layer<tracing_subscriber::Registry> {
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

    fmt_layer
}
