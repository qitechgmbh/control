![](./docs/assets/github-banner.png)

# QiTech Control

QiTech Control is an open-source framework designed to bring modern software development practices to certified industrial hardware.

It frees developers from proprietary, license-heavy PLC ecosystems and rigid "point-and-click" workflows that are no longer adequate for today's complex automation challenges.

QiTech Control combines the modularity and reliability of standard EtherCAT terminals (e.g., WAGO, Beckhoff) with the power of a modern Rust & React stack.

## Documentation

**[View Full Documentation Wiki](https://github.com/qitechgmbh/control/wiki)**

- [Getting Started](https://github.com/qitechgmbh/control/wiki/Getting-Started)
- [Architecture Overview](https://github.com/qitechgmbh/control/wiki/Architecture-Overview)
- [EtherCAT Basics](https://github.com/qitechgmbh/control/wiki/Ethercat-Basics)
- [Device Examples](https://github.com/qitechgmbh/control/wiki/Hardware-Examples)
- [REST API Reference](https://github.com/qitechgmbh/control/wiki/Rest-Api)

## Videos

[![](https://img.youtube.com/vi/KI3YeBwfV-s/maxresdefault.jpg)](https://www.youtube.com/watch?v=KI3YeBwfV-s)
*Software demo*

[![](https://img.youtube.com/vi/55egCAkQgyM/maxresdefault.jpg)](https://youtu.be/55egCAkQgyM) 
*Full explainer*

## Repository Structure

- **`/electron`** - React + Electron frontend
- **`/server`** - Rust backend implementing machine logic
- **`/ethercat-hal`** - Hardware abstraction layer for EtherCAT devices
- **`/control-core`** - Core control logic
- **`/nixos`** - Custom Linux OS with realtime kernel

## Technology Stack

**Backend:** Rust with [Ethercrab](https://github.com/ethercrab-rs/ethercrab) for EtherCAT, [SocketIO](https://socket.io/) for real-time communication, [axum](https://docs.rs/axum/latest/axum/) for REST API

**Frontend:** [Electron](https://www.electronjs.org/) + [React](https://react.dev/) with [Shadcn](https://ui.shadcn.com/) components and [Tailwind](https://tailwindcss.com/) styling


## Hardware Examples

QiTech Control supports a wide range of EtherCAT hardware from WAGO and Beckhoff. We provide complete step-by-step tutorials with wiring diagrams and software configuration for many common modules.

**[View all hardware examples and tutorials on the wiki](https://github.com/qitechgmbh/control/wiki/Hardware-Examples)**

Quick links to popular examples:
- [EL2004 - LED Control](https://github.com/qitechgmbh/control/wiki/Minimal-Example-El2004) (simplest setup for beginners)
- [EL3021 - Analog Input](https://github.com/qitechgmbh/control/wiki/Minimal-Example-El3021)
- [EL7031 - Stepper Motor Control](https://github.com/qitechgmbh/control/wiki/Minimal-Example-El7031-Motor)
- [WAGO 750-402 - Digital Input](https://github.com/qitechgmbh/control/wiki/Minimal-Example-Wago750-402)
- [WAGO 750-455 - Analog Input](https://github.com/qitechgmbh/control/wiki/Minimal-Example-Wago-750-455)

## QiTech Machines

QiTech Control powers 10+ production machines for filament extrusion, winding, measurement, and material processing.

**[View complete machine catalog on the wiki](https://github.com/qitechgmbh/control/wiki/QiTech-Machines)**


## Contributing

This is an open-source project. Contributions are welcome! Please see the [wiki](https://github.com/qitechgmbh/control/wiki) for development guidelines.

## License

See [LICENSE](LICENSE) file for details.
