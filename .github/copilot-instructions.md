This is a machine control software project.

## Repository structure

### Frontend

`/electron`: Frontend code for the control software built with React and Electron.

### Backend

`/server`: Implements machine setup & loop. Exposes a REST & Socket.io API for the electron frontend.
`/ethercat-hal`: Hardware abstraction layer for Beckhoff (and possibly other EtherCat) devices and protocols.
`/ethercat-hal-derive`: Macros for ethercat-hal
`/control-core`: Generic logic for the server.

### HMI Panel Operating System

`/nixos`: Custom Linux with realtime kernel & preconfigured for touchscreens.

### Other

/docs: Documentation for the project.

## Key Guidelines

1. Follow Rust & Typescript best practices and idiomatic patterns
2. Maintain existing code structure and organization
3. Write unit tests for new functionality where applicable.
4. Document public APIs and complex logic in code. Suggest changes to the `docs/` folder when appropriate
