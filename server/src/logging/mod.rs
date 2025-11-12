use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "tracing-fmt")]
pub mod fmt;

#[cfg(feature = "tracing-journald")]
pub mod journald;

/// Initialize the basic tracing system (without OpenTelemetry if enabled)
/// OpenTelemetry layer is deferred until async runtime is available
pub fn init_tracing() {
    // First try to get filter from env, then use default
    // Use RUST_LOG env var to control logging, e.g.:
    // RUST_LOG=info,h2=error,tower=error,tonic=error,hyper=error,opentelemetry_otlp=error
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Set very strict filters for the noisy OpenTelemetry components
        EnvFilter::new(
            "info,\
             tower_http=debug,\
             axum=debug,\
             ethercrab=info,\
             h2=error,\
             tower=error,\
             tonic=error,\
             hyper=error,\
             opentelemetry_otlp=error",
        )
    });

    let subscriber = tracing_subscriber::registry().with(env_filter);

    // Add fmt layer if enabled
    let subscriber = {
        #[cfg(feature = "tracing-fmt")]
        {
            subscriber.with(fmt::init_fmt_tracing())
        }
        #[cfg(not(feature = "tracing-fmt"))]
        {
            subscriber
        }
    };

    // Add journald layer if enabled
    let subscriber = {
        #[cfg(feature = "tracing-journald")]
        {
            subscriber.with(journald::init_journald_tracing())
        }
        #[cfg(not(feature = "tracing-journald"))]
        {
            subscriber
        }
    };

    subscriber.init();
}
