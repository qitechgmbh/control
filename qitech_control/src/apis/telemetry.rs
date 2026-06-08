use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter};
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct OpenTelemetryHandle {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: SdkLoggerProvider,
}

impl OpenTelemetryHandle {
    pub fn shutdown(&self) { 
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {err:?}");
        }

        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Error shutting down meter provider: {err:?}");
        }
 
        if let Err(err) = self.logger_provider.shutdown() {
            eprintln!("Error shutting down logger provider: {err:?}");
        }
    }
}

impl Drop for OpenTelemetryHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn init(resource: &Resource) -> OpenTelemetryHandle {
    let handle = OpenTelemetryHandle {
        tracer_provider: init_tracer_provider(resource),
        meter_provider: init_meter_provider(resource),
        logger_provider: init_logger_provider(resource),
    };

    println!("Initialized Telemetry");
    handle
}

fn init_tracer_provider(resource: &Resource) -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create span exporter");
    
    let provider = SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(exporter)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());

    provider
}

fn init_meter_provider(resource: &Resource) -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create meter exporter");

    let provider = SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_periodic_exporter(exporter)
        .build();

    global::set_meter_provider(provider.clone());
    provider
}

fn init_logger_provider(resource: &Resource) -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create log exporter");

    let provider = SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_simple_exporter(exporter)
        .build();

    // Logs: wire the tracing bridge so tracing::info! etc. go to OTel
    // let otel_layer = OpenTelemetryTracingBridge::new(&provider);
    // tracing_subscriber::registry()
    //     .with(otel_layer)
    //     .init();

    provider
}