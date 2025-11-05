# Component Interfaces and Boundaries

This document describes the interfaces between major components in the QiTech Control system and how they interact with each other.

## Component Layering

The system follows a clear layered architecture:

```
┌─────────────────────────────────────────────────────┐
│                   User Interface                     │
│                    (Electron)                        │
└────────────────────┬────────────────────────────────┘
                     │ REST API / SocketIO
                     ▼
┌─────────────────────────────────────────────────────┐
│              Machine Implementations                 │
│                    (server)                          │
└────────────────────┬────────────────────────────────┘
                     │ Trait implementations
                     ▼
┌─────────────────────────────────────────────────────┐
│               Control Framework                      │
│                 (control-core)                       │
└────────────────────┬────────────────────────────────┘
                     │ Device traits
                     ▼
┌─────────────────────────────────────────────────────┐
│          Hardware Abstraction Layer                  │
│                 (ethercat-hal)                       │
└─────────────────────────────────────────────────────┘
```

## 1. Hardware Abstraction Layer (ethercat-hal)

### Public Interface

The HAL provides traits and types for working with EtherCAT devices:

**Core Traits:**
- `Device`: Base trait for all EtherCAT devices
- `Group`: Trait for grouping related devices
- `PDO`: Process Data Object handling
- `CoE`: CANopen over EtherCAT configuration

**Key Types:**
- Input/Output primitives (digital, analog)
- Device configurations
- Error types

**Dependencies:**
- External: `ethercrab`, `smol`, `uom`, `bitvec`
- Internal: None (lowest layer)

**Usage Example:**
```rust
use ethercat_hal::{Device, Group};

// Implement Device trait for a custom EtherCAT device
impl Device for MyCustomDevice {
    // Implementation details
}
```

### Design Principles

- **Generic**: No knowledge of specific machines or control logic
- **Stateless**: Devices are primarily data structures for I/O
- **Type-safe**: Uses Rust's type system for safe I/O operations
- **Zero-cost abstractions**: Minimal runtime overhead

## 2. Control Framework (control-core)

### Public Interface

The control framework provides abstractions for building control systems:

**Core Traits:**
- `Actor`: Trait for control logic components that run in the control loop
- `Machine`: Trait for machine implementations
- `MachineValidation`: Trait for validating machine configurations

**Key Features:**
- Actor system for modular control logic
- SocketIO namespace abstractions
- REST API helpers
- Device identification and grouping
- Caching mechanisms

**Dependencies:**
- Internal: `ethercat-hal` (for device types)
- External: `socketioxide`, `axum`, `serde`, `smol`

**Usage Example:**
```rust
use control_core::{Actor, Machine, Group};

// Implement Actor for a control component
struct PIDController {
    // ... fields
}

impl Actor for PIDController {
    fn act(&mut self, group: &Group) -> anyhow::Result<()> {
        // Control logic that runs each cycle
        Ok(())
    }
}

// Implement Machine for a specific machine type
struct MyMachine {
    actors: Vec<Box<dyn Actor>>,
}

impl Machine for MyMachine {
    // Implementation details
}
```

### Design Principles

- **Generic**: Machine-agnostic, works for any industrial control application
- **Composable**: Actors can be combined to build complex control logic
- **Real-time capable**: Designed for deterministic execution
- **API-ready**: Built-in SocketIO and REST abstractions

## 3. Machine Implementations (server)

### Responsibilities

The server crate contains QiTech-specific implementations:

1. **Machine Implementations** (`src/machines/`)
   - Winder, Extruder, Laser, etc.
   - Each machine implements the `Machine` trait
   - Contains machine-specific actors and logic

2. **EtherCAT Setup** (`src/ethercat/`)
   - Interface discovery
   - Device enumeration
   - Group creation

3. **Serial Devices** (`src/serial/`)
   - Non-EtherCAT devices (e.g., laser sensors)
   - Serial protocol implementations

4. **API Handlers** (`src/rest/`, `src/socketio/`)
   - Machine mutation endpoints
   - Real-time status updates
   - Device identification management

5. **Control Loop** (`src/loop.rs`)
   - Main control loop execution
   - Thread management
   - Actor coordination

**Dependencies:**
- Internal: `control-core`, `ethercat-hal`
- External: `axum`, `socketioxide`, `tokio`, `smol`

**Example Machine Implementation:**
```rust
use control_core::{Machine, Actor, Group};
use ethercat_hal::Device;

pub struct WinderV2 {
    // Actors
    puller: Puller,
    traverse: Traverse,
    // ... other components
}

impl Machine for WinderV2 {
    fn act(&mut self, group: &Group) -> anyhow::Result<()> {
        // Coordinate all actors
        self.puller.act(group)?;
        self.traverse.act(group)?;
        Ok(())
    }
}
```

### Design Principles

- **Machine-specific**: Contains QiTech business logic
- **Uses framework**: Builds on `control-core` and `ethercat-hal`
- **API implementation**: Provides REST and SocketIO endpoints
- **Integration point**: Ties together hardware, framework, and UI

## 4. User Interface (electron)

### Interface with Backend

The Electron UI communicates with the server through two channels:

#### REST API

Used for request/response operations:

- **Machine mutations**: Start, stop, change parameters
- **Device identification**: Read/write device IDs
- **Configuration**: System settings

**Example:**
```typescript
// POST /machine/mutation
const response = await fetch('http://localhost:3001/machine/mutation', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    mutation: 'Start',
    parameters: { /* ... */ }
  })
});
```

#### SocketIO

Used for real-time updates:

- **Main namespace** (`/`): System-wide events (machine discovery, errors)
- **Machine namespaces** (`/machine/<vendor>/<id>`): Machine-specific state updates

**Example:**
```typescript
import { io } from 'socket.io-client';

const socket = io('http://localhost:3001/machine/1/2');

socket.on('state', (machineState) => {
  // Update UI with new machine state
  updateDisplay(machineState);
});
```

### Design Principles

- **Decoupled**: Communicates via APIs, not tight coupling
- **Reactive**: Uses SocketIO for real-time updates
- **Machine-aware**: Contains UI for each machine type
- **Reusable**: Built with standard web technologies

## 5. Operating System (nixos)

### Role

Provides the runtime environment:

- Real-time kernel for deterministic control loop execution
- System services for the server
- Touchscreen configuration
- Package management

### Integration

- Builds and packages the `server` binary
- Configures systemd service for automatic startup
- Provides system dependencies (libraries, drivers)

## Interface Guidelines

### For Framework Development (ethercat-hal, control-core)

When working on framework crates:

1. **No QiTech-specific code**: Keep implementations generic
2. **Document public APIs**: Assume external users
3. **Semantic versioning**: Changes should follow semver
4. **Minimal dependencies**: Only add well-maintained, stable crates
5. **Examples**: Provide usage examples in documentation
6. **Testing**: Write tests that don't depend on specific machines

### For Machine Development (server)

When implementing machines:

1. **Use framework traits**: Implement `Machine` and `Actor` traits
2. **Isolate machine code**: Keep machine-specific code in `src/machines/<machine_name>/`
3. **API design**: Design REST/SocketIO APIs with UI in mind
4. **Error handling**: Use `anyhow::Result` for error propagation
5. **Documentation**: Document machine-specific behavior

### For UI Development (electron)

When working on the UI:

1. **API contracts**: Respect REST/SocketIO API contracts
2. **Type safety**: Use TypeScript types for API responses
3. **Machine components**: Keep machine-specific UI in `src/machines/<machine_name>/`
4. **Reusable components**: Extract common patterns to shared components
5. **Error handling**: Handle network errors gracefully

## Cross-Cutting Concerns

### Error Handling

- **HAL**: Returns `anyhow::Result` for device operations
- **Framework**: Uses `anyhow::Result` with context
- **Server**: Propagates errors, logs with `tracing`
- **UI**: Displays user-friendly error messages

### Logging and Tracing

- **Backend**: Uses `tracing` crate with structured logging
- **UI**: Uses console logging and optional telemetry

### Testing

- **Unit tests**: Each crate has unit tests for internal logic
- **Integration tests**: Server has tests that span multiple components
- **E2E tests**: Electron has Playwright tests for UI workflows

### Configuration

- **Server**: TOML configuration file (`config.toml`)
- **UI**: Electron store for user preferences
- **NixOS**: System configuration in Nix files

## Future Considerations

### If Splitting into Multiple Repositories

1. **Version Pinning**: Server would depend on specific versions of framework crates
2. **Release Coordination**: Framework releases need stability guarantees
3. **Breaking Changes**: Would require careful migration planning
4. **Documentation**: Each repository needs comprehensive docs
5. **CI/CD**: Separate pipelines for each repository

### Maintaining Current Structure

1. **Clean Boundaries**: Continue enforcing layer separation
2. **Interface Stability**: Treat framework interfaces as if they were versioned APIs
3. **Documentation**: Keep this document updated
4. **Testing**: Ensure framework crates can be tested independently

## Summary

The QiTech Control system has a well-defined layered architecture:

- **ethercat-hal**: Provides hardware abstraction (reusable)
- **control-core**: Provides control framework (reusable)
- **server**: Implements QiTech machines (specific)
- **electron**: Provides user interface (adaptable)
- **nixos**: Provides runtime environment (adaptable)

Each layer depends only on layers below it, creating clear separation of concerns and enabling potential future modularization.
