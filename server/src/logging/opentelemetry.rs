//! OpenTelemetry tracing layer implementation with channel-based async exporter.
//!
//! This module provides a custom OpenTelemetry tracing layer that solves the runtime
//! incompatibility between the smol async runtime (used by the main application) and
//! the Tokio runtime (required by OpenTelemetry gRPC exporters).
//!
//! The solution uses a channel-based approach where spans are sent from the main thread
//! to a dedicated Tokio thread that handles the actual gRPC export to Jaeger.

use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    error::OTelSdkResult,
    trace::{SdkTracerProvider, SpanData, SpanExporter},
};
use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, filter::filter_fn};

// Service configuration constants
const SERVICE_NAME: &str = "qitech-control-server";
const SERVICE_NAMESPACE: &str = "qitech";
const OTLP_ENDPOINT: &str = "http://localhost:4317";
const EXPORTER_THREAD_NAME: &str = "OtelExporter";

// Flag to control detailed h2/HTTP debug logging
// Set to true to see detailed HTTP/2 protocol logs, false to suppress them
const ENABLE_H2_DEBUG_LOGS: bool = false;

/// A custom span exporter that bridges the gap between async runtimes.
///
/// This exporter receives spans from the main application (running on smol) and
/// forwards them through a channel to a dedicated Tokio thread that handles the
/// actual gRPC export to the OpenTelemetry collector.
#[derive(Debug)]
struct ChannelSpanExporter {
    sender: Sender<Vec<SpanData>>,
}

impl ChannelSpanExporter {
    /// Creates a new channel-based span exporter.
    fn new(sender: Sender<Vec<SpanData>>) -> Self {
        Self { sender }
    }
}

impl SpanExporter for ChannelSpanExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        match self.sender.send(batch) {
            Ok(()) => Ok(()),
            Err(_) => {
                tracing::warn!("Failed to send spans to exporter thread - channel closed");
                // Return success to avoid failing the application when export fails
                Ok(())
            }
        }
    }
}

/// Initializes OpenTelemetry tracing with a channel-based exporter.
///
/// This function sets up a complete OpenTelemetry tracing pipeline that works
/// seamlessly with the smol async runtime. It creates a dedicated Tokio thread
/// for handling gRPC exports to avoid runtime conflicts.
///
/// # Returns
///
/// A boxed tracing layer that can be used with `tracing_subscriber`.
pub fn init_opentelemetry_tracing_with_tokio<S>()
-> Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber
        + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>
        + Send
        + Sync,
{
    let (span_sender, span_receiver) = mpsc::channel::<Vec<SpanData>>();

    // Start the dedicated exporter thread
    spawn_exporter_thread(span_receiver);

    // Build the service resource with metadata
    let resource = Resource::builder()
        .with_attribute(KeyValue::new("service.name", SERVICE_NAME))
        .build();

    // Create the tracer provider with our custom exporter
    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(ChannelSpanExporter::new(span_sender))
        .with_resource(resource)
        .build();

    // Register the global tracer provider
    let _ = global::set_tracer_provider(tracer_provider.clone());

    // Create the tracer and layer
    let tracer = tracer_provider.tracer(SERVICE_NAME);

    tracing::info!(
        "OpenTelemetry tracer initialized with channel-based exporter service_name={} namespace={} endpoint={}",
        SERVICE_NAME,
        SERVICE_NAMESPACE,
        OTLP_ENDPOINT
    );

    Box::new(tracing_opentelemetry::layer().with_tracer(tracer))
}

/// Spawns a dedicated thread with a Tokio runtime for gRPC span export.
///
/// This function creates a background thread that runs a Tokio runtime specifically
/// for handling OpenTelemetry gRPC exports. This isolation prevents runtime conflicts
/// with the main application's smol runtime.
///
/// # Arguments
///
/// * `receiver` - Channel receiver for incoming span batches
fn spawn_exporter_thread(receiver: Receiver<Vec<SpanData>>) {
    std::thread::Builder::new()
        .name(EXPORTER_THREAD_NAME.to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime for OpenTelemetry exporter");

            runtime.block_on(async {
                let exporter = match create_otlp_exporter().await {
                    Ok(exporter) => {
                        tracing::info!("OTLP gRPC exporter initialized successfully");
                        exporter
                    }
                    Err(error) => {
                        tracing::error!("Failed to create OTLP exporter error={}", error);
                        return;
                    }
                };

                // Process incoming span batches
                while let Ok(spans) = receiver.recv() {
                    if !spans.is_empty() {
                        if let Err(error) = export_spans(&exporter, spans).await {
                            tracing::warn!("Failed to export span batch error={}", error);
                        }
                    }
                }
            });
        })
        .expect("Failed to spawn OpenTelemetry exporter thread");
}

/// Creates an OTLP gRPC exporter for sending spans to Jaeger.
///
/// This function must be called within a Tokio runtime context as it uses
/// async gRPC operations.
///
/// # Returns
///
/// A configured OTLP span exporter that sends data to the local Jaeger instance.
///
/// # Errors
///
/// Returns an error if the exporter fails to initialize with the specified endpoint.
async fn create_otlp_exporter() -> anyhow::Result<opentelemetry_otlp::SpanExporter> {
    // If debug logs are disabled, configure the underlying tonic to be quiet
    if !ENABLE_H2_DEBUG_LOGS {
        // Silence the underlying HTTP/2 debug logs
        let _ = tracing::subscriber::set_global_default(tracing_subscriber::registry().with(
            tracing_subscriber::EnvFilter::new(
                "error,h2=error,tower=error,tonic=error,hyper=error,opentelemetry_otlp=error",
            ),
        ));
    }

    opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(OTLP_ENDPOINT)
        .build()
        .map_err(|error| anyhow::anyhow!("Failed to build OTLP exporter: {}", error))
}

/// Exports a batch of spans using the OTLP exporter.
///
/// This function handles the actual gRPC export operation and must be called
/// within a Tokio runtime context.
///
/// # Arguments
///
/// * `exporter` - The OTLP exporter instance
/// * `spans` - Batch of spans to export
///
/// # Errors
///
/// Returns an error if the export operation fails.
async fn export_spans(
    exporter: &opentelemetry_otlp::SpanExporter,
    spans: Vec<SpanData>,
) -> anyhow::Result<()> {
    exporter
        .export(spans)
        .await
        .map_err(|error| anyhow::anyhow!("Span export failed: {:?}", error))
}

/// A custom layer that filters out all events from the OtelExporter thread.
///
/// This layer is used to suppress the verbose debug logs from the OpenTelemetry
/// exporter thread, allowing control of verbosity without modifying the underlying
/// libraries directly.
#[derive(Debug)]
struct ThreadNameFilter<S> {
    blocked_thread_name: String,
    inner: S,
}

impl<S> ThreadNameFilter<S> {
    fn new(blocked_thread_name: impl Into<String>, inner: S) -> Self {
        Self {
            blocked_thread_name: blocked_thread_name.into(),
            inner,
        }
    }
}

impl<S> Layer<S> for ThreadNameFilter<S>
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Only pass the event through if it's not from the blocked thread
        if let Some(thread_name) = thread::current().name() {
            if thread_name == self.blocked_thread_name {
                // Skip the event, preventing it from being recorded
                return;
            }
        }

        // For all other threads, proceed normally
    }
}

/// Creates a filter layer that blocks all events from the OtelExporter thread.
///
/// This is used to prevent the verbose debug logs from the OpenTelemetry exporter
/// thread from flooding the console.
pub fn create_thread_filter_layer<S>() -> impl tracing_subscriber::Layer<S> + Send + Sync + 'static
where
    S: tracing::Subscriber
        + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>
        + Send
        + Sync,
{
    // Create a filter that rejects events from the OtelExporter thread
    filter_fn(move |metadata| {
        // If this is a debug level event and current thread is OtelExporter, don't allow it
        if metadata.level() <= &tracing::Level::DEBUG {
            if let Some(thread_name) = std::thread::current().name() {
                if thread_name == EXPORTER_THREAD_NAME {
                    return false;
                }
            }
        }

        // For all other cases, allow the event
        true
    })
}
