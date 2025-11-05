# Repository Organization

This document explains the current repository structure and provides guidance on how the codebase is organized for maintainability and potential future modularization.

## Overview

The QiTech Control repository is currently a monorepo that contains multiple logical components:

1. **Hardware Abstraction Layer (HAL)** - Low-level EtherCAT device interfaces
2. **Control Framework** - Generic control logic and abstractions
3. **Machine Implementations** - QiTech-specific machine logic
4. **User Interface** - Electron-based control panel UI
5. **Operating System** - NixOS configuration for deployment

## Current Structure

### 1. Hardware Abstraction Layer

**Crates:** `ethercat-hal`, `ethercat-hal-derive`

**Purpose:** Provides low-level abstractions for interfacing with Beckhoff and other EtherCAT devices.

**Key Features:**
- Device trait definitions
- PDO (Process Data Object) handling
- CoE (CANopen over EtherCAT) configuration
- I/O primitives
- Built on top of [ethercrab](https://github.com/ethercrab-rs/ethercrab)

**Dependencies:** External crates only (ethercrab, smol, uom, etc.)

**Reusability:** Highly reusable for any EtherCAT-based control system

### 2. Control Framework

**Crates:** `control-core`, `control-core-derive`

**Purpose:** Provides generic control logic abstractions that can be used across different machine types.

**Key Features:**
- Actor system for control logic components
- SocketIO and REST API abstractions
- Machine trait definitions
- Device identification and validation
- Group management
- Video streaming support (optional)

**Dependencies:** `ethercat-hal`, external crates

**Reusability:** Reusable for building various industrial control systems

### 3. Machine Implementations (QiTech-Specific)

**Crate:** `server`

**Purpose:** Contains QiTech-specific machine implementations and glue code that ties everything together.

**Key Components:**
- **Machine implementations** (`src/machines/`):
  - Winder V2
  - Extruder V1/V2
  - Aquapath (Waterway) V1
  - Laser V1
  - Buffer V1
  - Mock machines for testing
- **EtherCAT setup** (`src/ethercat/`): Interface discovery, configuration
- **Serial device handling** (`src/serial/`): Non-EtherCAT devices
- **REST API handlers** (`src/rest/`): Machine mutations, device identification
- **SocketIO handlers** (`src/socketio/`): Real-time communication
- **Control loop** (`src/loop.rs`): Main control loop execution

**Dependencies:** `control-core`, `ethercat-hal`, external crates

**Reusability:** QiTech-specific, but demonstrates how to use the framework

### 4. User Interface

**Directory:** `electron/`

**Purpose:** Electron-based user interface for controlling and monitoring machines.

**Key Features:**
- React-based UI with TypeScript
- Real-time updates via SocketIO
- REST API client for machine control
- Machine-specific UI components (`src/machines/`)
- Shadcn/UI component library
- TanStack Router for navigation
- Theming and internationalization support

**Dependencies:** Node.js, Electron, React ecosystem

**Reusability:** Could be reused with modifications for similar control systems

### 5. Operating System Configuration

**Directory:** `nixos/`

**Purpose:** NixOS configuration for deployment on control panels.

**Key Features:**
- Real-time kernel configuration
- Touchscreen support
- System service definitions
- Package management

**Reusability:** Reusable for similar embedded Linux deployments

### 6. Utility Crates

**Crate:** `ethercat-eeprom-dump`

**Purpose:** Command-line utility for dumping EtherCAT device EEPROM data.

**Dependencies:** `ethercat-hal`, `control-core`

## Dependency Graph

```
┌─────────────────────┐
│   ethercat-hal      │◄─── Low-level HAL
│ + derive macros     │
└──────────┬──────────┘
           │
           │ depends on
           ▼
┌─────────────────────┐
│   control-core      │◄─── Generic framework
│ + derive macros     │
└──────────┬──────────┘
           │
           │ depends on
           ▼
┌─────────────────────┐
│      server         │◄─── QiTech-specific
│  (machine impls)    │     implementations
└─────────────────────┘

┌─────────────────────┐
│      electron       │◄─── UI (communicates via
│        (UI)         │     REST/SocketIO APIs)
└─────────────────────┘

┌─────────────────────┐
│       nixos         │◄─── OS configuration
│   (deployment)      │
└─────────────────────┘
```

## Conceptual Separation

If this repository were to be split into separate repositories, a logical separation would be:

### Option A: Three Repository Split

1. **`control-framework`** (or `industrial-control-rs`)
   - Contains: `ethercat-hal`, `ethercat-hal-derive`, `control-core`, `control-core-derive`
   - Purpose: Reusable Rust framework for building industrial control systems
   - Audience: Other companies/projects building EtherCAT control systems
   - Published to: crates.io

2. **`control-ui`** (or `control-electron-ui`)
   - Contains: `electron/` directory
   - Purpose: Reusable Electron-based UI for control systems
   - Audience: Projects that want a pre-built control panel UI
   - Could be: npm package or standalone repository

3. **`control-qitech`**
   - Contains: `server/` (machine implementations), `nixos/` (deployment config)
   - Dependencies: Uses `control-framework` crates, includes `control-ui` as git submodule
   - Purpose: QiTech-specific machine implementations and deployment
   - Audience: QiTech internal use

### Option B: Two Repository Split (Simpler)

1. **`control-framework`**
   - Contains: `ethercat-hal`, `control-core`, derive macros, `electron/`, `nixos/`
   - Purpose: Complete reusable control system framework (backend + UI + OS)
   - Published: Core crates to crates.io, UI could be npm package

2. **`control-qitech`**
   - Contains: `server/` machine implementations
   - Dependencies: Uses `control-framework` crates and UI
   - Purpose: QiTech-specific implementations

## Advantages of Current Monorepo Structure

1. **Faster Development**: No need to coordinate versions across repositories
2. **Atomic Changes**: Changes that span multiple components can be done in a single commit
3. **Simplified CI/CD**: Single build pipeline for all components
4. **Easier Testing**: Integration tests can span all components
5. **No Submodule Complexity**: Git submodules can be challenging to work with
6. **Unified Documentation**: All docs in one place
7. **Consistent Versioning**: Everything versioned together

## Advantages of Multi-Repository Structure

1. **Clear Boundaries**: Enforces separation of concerns at repository level
2. **Independent Versioning**: Framework can have stable releases while QiTech-specific code evolves
3. **Access Control**: Different teams can have different permissions
4. **Selective Cloning**: Users only clone what they need
5. **Public/Private Split**: Framework could be open-source while keeping QiTech-specific code private
6. **Clearer Licensing**: Different licenses for different components
7. **Smaller Repositories**: Faster clone times, less clutter

## Recommendations

### For Current Development (Monorepo)

The monorepo structure is appropriate for the current stage of development because:

1. The framework is still evolving rapidly alongside machine implementations
2. The team is small and benefits from unified development workflow
3. Breaking changes across layers can be handled atomically
4. There's no immediate need to share the framework externally

**Best Practices to Maintain:**
- Keep dependencies clean (framework crates should not depend on `server`)
- Document public APIs in framework crates as if they were standalone
- Write framework code generically without QiTech-specific assumptions
- Keep machine-specific code isolated in `server/src/machines/`

### For Future (Multi-Repository)

Consider splitting when:

1. **Framework Stabilization**: When `ethercat-hal` and `control-core` APIs are stable enough for versioned releases
2. **External Users**: When other companies/projects want to use the framework
3. **Team Growth**: When separate teams are working on framework vs. machines
4. **Open Source Plans**: If planning to open-source the framework while keeping machine implementations private

**Migration Path:**
1. Extract `ethercat-hal` and `control-core` to `control-framework` repository
2. Publish to crates.io with semantic versioning
3. Update `server/Cargo.toml` to use published crates instead of path dependencies
4. Optionally extract `electron/` as a separate UI package
5. Keep `server/` as QiTech-specific implementation repository

## Current Approach: Virtual Separation

To get benefits of both approaches:

1. **Maintain Clean Boundaries**: Continue enforcing that framework crates don't depend on `server`
2. **Documentation**: Document framework crates as standalone libraries
3. **Testing**: Write tests for framework crates that don't depend on machine implementations
4. **Cargo Workspaces**: Use workspace organization to group related crates
5. **Future-Ready**: Write code assuming eventual split, making future extraction easier

## Workspace Organization

The workspace is organized to reflect logical grouping:

```toml
[workspace]
members = [
    # Framework crates (HAL layer)
    "ethercat-hal",
    "ethercat-hal-derive",
    
    # Framework crates (Core layer)
    "control-core",
    "control-core-derive",
    
    # Machine implementations
    "server",
    
    # Utilities
    "ethercat-eeprom-dump",
]
```

This organization makes it clear which crates are part of the reusable framework and which are application-specific.

## Conclusion

The current monorepo structure is well-suited for QiTech's needs at this stage. The codebase already has good logical separation between:
- Reusable framework components (`ethercat-hal`, `control-core`)
- Machine-specific implementations (`server`)
- User interface (`electron`)
- Deployment configuration (`nixos`)

The key is to maintain clean boundaries and follow best practices that would make a future split straightforward if needed. This gives QiTech the flexibility to evolve the architecture as requirements change without prematurely optimizing for a multi-repository structure.
