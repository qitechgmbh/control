# Logging and Tracing Configuration

This document describes the logging and tracing system for the QiTech control server, including structured logging, distributed tracing, and different output formats.

## Overview

The QiTech control server uses a modular tracing system built on top of the `tracing` crate, supporting multiple output formats and observability backends. The system is designed to be flexible and configurable through Cargo features.

## Features

The logging system supports three main features that can be enabled independently:

- **`tracing-fmt`**: Human-readable console output (default)
- **`tracing-journald`**: systemd journal integration for production Linux systems
- **`tracing-otel`**: OpenTelemetry integration for distributed tracing

### Default Configuration

By default, the server uses `tracing-fmt` for development-friendly console output:

```bash
cargo run  # Uses tracing-fmt by default
```

## Feature-Based Configuration

### Format Logging (`tracing-fmt`)

The fmt logger provides structured, human-readable output to the console. This is the default feature and is ideal for development and debugging.

**Features:**
- Colored output (when supported)
- Thread names and IDs
- Line numbers and file locations
- Configurable time formats (debug vs release)
- Target filtering

**Example output:**
```
2025-06-01T10:30:45.123Z INFO server::main: Starting QiTech Control Server
2025-06-01T10:30:45.125Z DEBUG ethercat::init: Initializing EtherCAT master
```

**Usage:**
```bash
# Default behavior
cargo run

# Explicit feature selection
cargo run --features tracing-fmt --no-default-features
```

### Journal Logging (`tracing-journald`)

The journald logger integrates with systemd's journal for production Linux deployments. **This is the standard logging backend used on NixOS systems** and other systemd-based distributions.

**Features:**
- Native systemd journal integration
- Structured metadata preservation
- System-level log aggregation
- Log rotation and retention policies
- Remote log collection support

**Usage:**
```bash
# Enable journald logging
cargo run --features tracing-journald --no-default-features

# Combined with other features
cargo run --features "tracing-journald,tracing-otel"
```

**Viewing logs on NixOS:**
```bash
# View all logs from the service
journalctl -u qitech-control-server -f

# Filter by log level
journalctl -u qitech-control-server -p info

# JSON output for structured data
journalctl -u qitech-control-server -o json-pretty

# Follow logs with timestamp
journalctl -u qitech-control-server -f --since "1 hour ago"
```

### OpenTelemetry Tracing (`tracing-otel`)

OpenTelemetry provides distributed tracing capabilities by exporting traces to observability platforms like Jaeger, Zipkin, or commercial APM solutions.

**Features:**
- Custom channel-based OTLP gRPC exporter for async runtime isolation
- Hardcoded service metadata (no environment variables)
- Dedicated Tokio thread for span export (isolated from main smol runtime)
- Batch span processing for efficient export
- Distributed trace correlation across services
- Integration with HTTP middleware and database operations

**Technical Implementation:**
The implementation uses a custom `ChannelSpanExporter` that sends spans via `std::sync::mpsc::channel` to a dedicated Tokio thread. This design completely isolates the OpenTelemetry Tokio dependency from the main application's smol-based async runtime, preventing runtime conflicts.

**Configuration:**
- Service Name: `qitech-control-server`
- Service Version: Automatically set from `Cargo.toml`
- Service Namespace: `qitech`
- OTLP Endpoint: `http://localhost:4317` (Jaeger gRPC endpoint)
- Export Method: Channel-based with dedicated Tokio thread

**Setup Jaeger for Development:**

```bash
# Run Jaeger with OTLP support
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 14268:14268 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest

# Access Jaeger UI
open http://localhost:16686
```

**Usage:**
```bash
# Enable OpenTelemetry with OTLP export to Jaeger
cargo run --features tracing-otel --no-default-features

# Combined with console output
cargo run --features "tracing-fmt,tracing-otel"
```

The server will automatically export traces to Jaeger at `http://localhost:4317`. All HTTP requests, database operations, and custom spans will be visible in the Jaeger UI. The implementation uses a custom channel-based exporter that spawns a dedicated Tokio thread for OTLP gRPC communication, ensuring compatibility with the main application's smol async runtime.

## Multiple Feature Combinations

You can enable multiple logging features simultaneously:

```bash
# All logging features enabled
cargo run --features "tracing-fmt,tracing-journald,tracing-otel"

# Production setup: journald + OpenTelemetry
cargo run --features "tracing-journald,tracing-otel" --no-default-features

# Development setup: console + OpenTelemetry
cargo run --features "tracing-fmt,tracing-otel" --no-default-features
```

## Environment Variables

### Log Level Configuration

The log level is controlled through the `RUST_LOG` environment variable:

```bash
# Basic log levels
RUST_LOG=debug cargo run
RUST_LOG=info cargo run  # Default
RUST_LOG=warn cargo run
RUST_LOG=error cargo run

# Module-specific filtering
RUST_LOG=server=debug,ethercat=info cargo run

# Complex filtering
RUST_LOG="info,tower_http=debug,axum=debug" cargo run
```

### OpenTelemetry Configuration

When using the `tracing-otel` feature, the service configuration is hardcoded and uses a custom channel-based exporter for async runtime compatibility:

#### Service Configuration
- **Service Name**: "qitech-control-server" 
- **Service Version**: Automatically set from Cargo.toml package version
- **Service Namespace**: "qitech"
- **OTLP Endpoint**: "http://localhost:4317" (Jaeger gRPC endpoint)

#### Implementation Details
The OpenTelemetry implementation uses a custom `ChannelSpanExporter` that:
1. Receives spans from the tracing layer via an MPSC channel
2. Spawns a dedicated Tokio thread for OTLP gRPC export
3. Isolates OpenTelemetry's async runtime from the main smol-based application
4. Provides reliable span export to Jaeger without runtime conflicts

This design ensures that the OpenTelemetry tracing system works seamlessly with applications using different async runtimes (smol, tokio, etc.).

## Usage Examples

### Development Setup

For local development with console output:

```bash
RUST_LOG=debug cargo run --features tracing-fmt
```

### Production Setup (NixOS)

For production deployment on NixOS systems, we use journald logging as the primary backend:

```bash
RUST_LOG=info cargo run --features "tracing-journald,tracing-otel" --no-default-features
```

This configuration is automatically used in our NixOS deployments through the system service configuration.

### Distributed Tracing with OpenTelemetry

The OpenTelemetry implementation provides full distributed tracing capabilities with automatic export to Jaeger:

```bash
RUST_LOG=debug cargo run --features "tracing-fmt,tracing-otel"
```

This starts the application with both console logging and OpenTelemetry tracing enabled. Spans are automatically exported to Jaeger via the custom channel-based exporter, allowing you to view distributed traces in the Jaeger UI at `http://localhost:16686`.

### Setting Up Jaeger to Receive Traces

To collect and visualize traces from your application, you can run Jaeger as a trace receiver:

#### 1. Start Jaeger with OTLP Support

Run Jaeger using Docker with OTLP endpoints enabled:

```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 14268:14268 \
  -p 14250:14250 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

#### 2. Access Jaeger UI

Open your browser and go to:
- **URL**: http://localhost:16686
- **Service**: Select "qitech-control-server" from the dropdown
- **Operation**: Choose specific operations to trace
- **Find Traces**: Click to view collected traces

#### 3. Run Your Application with OpenTelemetry

The OpenTelemetry implementation automatically exports traces to Jaeger:

```bash
# Run with OpenTelemetry tracing enabled
cargo run --features "tracing-fmt,tracing-otel"
```

The application uses these hardcoded service identifiers:
- **Service Name**: "qitech-control-server"
- **Service Version**: From Cargo.toml package version
- **Service Namespace**: "qitech"
- **OTLP Endpoint**: "http://localhost:4317" (gRPC)

Traces will automatically appear in the Jaeger UI under the service name "qitech-control-server". The custom channel-based exporter ensures reliable trace delivery even when using different async runtimes.

### Alternative: Using Docker Compose

Create a `docker-compose.yml` for easier management:

```yaml
version: '3.8'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "14268:14268"  # HTTP collector
      - "14250:14250"  # gRPC collector
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

Run with:
```bash
docker-compose up -d jaeger
```

### Other Observability Platforms

The current channel-based OpenTelemetry exporter is configured for Jaeger (OTLP gRPC endpoint). For other platforms like Grafana, DataDog, or custom OTLP collectors, you can modify the endpoint configuration in `src/logging/opentelemetry.rs` by updating the `OTLP_ENDPOINT` constant.

## Adding Tracing to Code

### Basic Logging

```rust
use tracing::{info, debug, warn, error, trace};

pub fn my_function() {
    trace!("Very detailed debug information");
    debug!("Debug information for developers");
    info!("General information about program execution");
    warn!("Something unexpected happened");
    error!("An error occurred: {}", error_message);
}
```

### Structured Logging

**Important**: When logging to journald (systemd's logging service), key-value pairs in event logs are not properly captured. Instead, include structured data using string formatting within the message itself.

```rust
use tracing::{info, instrument};

// ❌ Don't use key-value pairs in event logs (won't work with journald)
info!(
    user_id = %user.id,
    email = %user.email,
    "Updating user profile"
);

// ✅ Use string formatting instead (works with journald)
info!(
    "Updating user profile user_id={} email={}",
    user.id,
    user.email
);

// ✅ Spans can still use structured fields
#[instrument(fields(user_id = %user.id, operation = "update"))]
pub fn update_user(user: &User) {
    info!(
        "Updating user profile user_id={} email={}",
        user.id,
        user.email
    );
}
```

### Custom Spans

```rust
use tracing::{Span, instrument};

pub fn complex_operation() {
    let span = tracing::info_span!(
        "complex_operation",
        operation_id = 123,
        stage = "initialization"
    );
    let _enter = span.enter();

    info!("Starting complex operation");
    
    // Create child span
    let child_span = tracing::debug_span!("database_query");
    let _child_enter = child_span.enter();
    
    debug!("Executing database query");
}
```

### Error Handling

```rust
use tracing::{error, warn};
use anyhow::Result;

pub fn operation_with_error_handling() -> Result<()> {
    match risky_operation() {
        Ok(result) => {
            info!("Operation completed successfully result={:?}", result);
            Ok(())
        }
        Err(e) => {
            error!("Operation failed error={}", e);
            Err(e)
        }
    }
}
```

## Docker Compose Example

Complete observability stack with Jaeger:

```yaml
version: '3.8'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "14268:14268"
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  control-server:
    build: .
    environment:
      - RUST_LOG=info
      - OTEL_EXPORTER=jaeger
      - OTEL_EXPORTER_JAEGER_ENDPOINT=http://jaeger:14268/api/traces
      - OTEL_SERVICE_NAME=qitech-control-server
    depends_on:
      - jaeger
    # Use journald for system logs
    logging:
      driver: journald
```

## Performance Considerations

- **Fmt logging**: Minimal overhead, suitable for development
- **Journald logging**: Low overhead, optimized for production systems
- **OpenTelemetry**: Moderate overhead due to span collection and channel-based export; the custom implementation minimizes impact by using a dedicated thread for OTLP communication

## Troubleshooting

### No log output
- Check `RUST_LOG` environment variable
- Verify the correct features are enabled
- Ensure the logging initialization is called in `main()`

### OpenTelemetry traces not appearing in Jaeger
- Verify Jaeger is running with OTLP support on port 4317
- Check that the `tracing-otel` feature is enabled
- Ensure Jaeger OTLP collector is enabled: `COLLECTOR_OTLP_ENABLED=true`
- Look for error messages in the application logs related to OTLP export
- Verify the service name "qitech-control-server" appears in Jaeger's service dropdown

### Service name shows as "unknown_service:server" in Jaeger
- This is a known issue where the resource configuration may not be applied correctly
- The traces will still work correctly, but the service name display may be incorrect
- Check the span metadata for the correct service information

### Journald logs not appearing
- Ensure systemd is running
- Check systemd service configuration
- Verify the `tracing-journald` feature is enabled

## NixOS Configuration

The QiTech control server is automatically configured for optimal logging on NixOS systems:

### Package Configuration

The server package is built with the `tracing-journald` feature enabled:

```nix
# In nixos/packages/server.nix
buildFeatures = [ "tracing-journald" ];
buildNoDefaultFeatures = true;
```

### Service Configuration

The systemd service is configured to use journald logging:

```nix
# In nixos/modules/qitech.nix
systemd.services.qitech-control-server = {
  serviceConfig = {
    StandardOutput = "journal";
    StandardError = "journal";
    SyslogIdentifier = "qitech-control-server";
  };
  
  environment = {
    RUST_LOG = "info,tower_http=debug,axum=debug";
    QITECH_OS = "1";  # Legacy compatibility
  };
};
```

This configuration ensures that all logs are properly structured and integrated with the systemd journal.

For more specific configuration options, see the individual module documentation in the source code.
