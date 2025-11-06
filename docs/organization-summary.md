# Repository Organization Summary

This document provides a quick reference for understanding the QiTech Control repository structure.

## Quick Navigation

- **[Repository Organization](./repository-organization.md)** - Complete guide to the monorepo structure
- **[Component Interfaces](./component-interfaces.md)** - How components interact
- **[Migration Guide](./migration-guide-splitting.md)** - How to split into multiple repos (if needed)
- **[Architecture Overview](./architecture-overview.md)** - System architecture diagrams

## At a Glance

### Framework Layer (Reusable)

These components form a generic control framework that could be used by other projects:

```
ethercat-hal + ethercat-hal-derive
    ↓
control-core + control-core-derive
```

- **Purpose**: Provide abstractions for building industrial control systems
- **Audience**: Could be published to crates.io for external use
- **Dependencies**: Only on external crates and each other

### Application Layer (QiTech-Specific)

```
server/
├── machines/      # Machine implementations (winder, extruder, etc.)
├── ethercat/      # EtherCAT setup
├── serial/        # Serial device handling
├── rest/          # REST API handlers
└── socketio/      # SocketIO handlers
```

- **Purpose**: QiTech-specific machine implementations
- **Audience**: QiTech internal use
- **Dependencies**: Uses framework + external crates

### User Interface

```
electron/
├── src/machines/  # Machine-specific UIs
├── components/    # Reusable UI components
├── routes/        # Application routing
└── ...
```

- **Purpose**: Control panel user interface
- **Technology**: Electron, React, TypeScript
- **Communication**: REST + SocketIO APIs

## Key Principles

1. **Clean Layering**: Framework doesn't depend on application code
2. **Type Safety**: Rust's type system ensures safe hardware access
3. **Real-time Capable**: Designed for deterministic control loops
4. **API-First**: UI communicates through well-defined APIs
5. **Testable**: Each layer can be tested independently

## When to Use What

### Working on Hardware Support
- Edit: `ethercat-hal/`
- Purpose: Adding new EtherCAT devices or improving HAL

### Working on Framework Features
- Edit: `control-core/`
- Purpose: Adding actors, improving control logic abstractions

### Working on Machines
- Edit: `server/src/machines/`
- Purpose: Implementing or modifying machine behavior

### Working on UI
- Edit: `electron/`
- Purpose: UI changes, new screens, UX improvements

### Working on System Configuration
- Edit: `nixos/`
- Purpose: OS configuration, deployment setup

## Common Questions

### Q: Can I depend on `server` from `control-core`?
**A: No.** Framework crates should never depend on application code. This maintains reusability.

### Q: Where should I put reusable control logic?
**A: In `control-core`.** If it's generic and could be used by multiple machines, it belongs in the framework.

### Q: Where should I put QiTech-specific logic?
**A: In `server`.** Machine-specific implementations go in `server/src/machines/`.

### Q: Should we split into multiple repositories?
**A: Not yet.** The current monorepo structure works well for the current development phase. See the [Migration Guide](./migration-guide-splitting.md) for when and how to split if needed.

### Q: How do I ensure my changes maintain clean boundaries?
**A: Follow these guidelines:**
1. Framework code should be generic (no QiTech-specific assumptions)
2. Document public APIs as if for external users
3. Keep machine code isolated in appropriate directories
4. Write tests that verify isolation

## Quick Reference: Dependency Rules

```
✅ Allowed:
- server → control-core → ethercat-hal
- control-core → ethercat-hal
- electron → server (via APIs only)

❌ Not Allowed:
- ethercat-hal → control-core
- control-core → server
- ethercat-hal → server
- server → electron
```

## Getting Started

New to the project? Start here:

1. Read [Getting Started](./developer-docs/getting-started.md)
2. Review [Architecture Overview](./architecture-overview.md)
3. Read [Repository Organization](./repository-organization.md)
4. Look at [Component Interfaces](./component-interfaces.md) for detailed API information

## Contributing

When making changes:

1. **Understand the layer** you're working in
2. **Follow the dependencies** - only depend on lower layers
3. **Document public APIs** - especially in framework crates
4. **Write tests** - unit tests for each layer, integration tests across layers
5. **Keep it clean** - maintain separation of concerns

## Future Considerations

The repository is structured to allow future splitting if needed:

- **Short term**: Continue with monorepo, maintain clean boundaries
- **Medium term**: Prepare framework crates for publication (add docs, examples)
- **Long term**: Consider splitting when framework is stable and there are external users

See [Migration Guide](./migration-guide-splitting.md) for the full plan.

---

For more detailed information, see the complete documentation:
- [Repository Organization](./repository-organization.md)
- [Component Interfaces](./component-interfaces.md)
- [Migration Guide](./migration-guide-splitting.md)
