<p align="center">
  <img width="1280" height="537" alt="QiTech Control" src="https://github.com/user-attachments/assets/7fa67de4-772e-4528-bd6d-72b56380cad7" />
</p>

# QiTech Control

QiTech Control is a real-time control system for industrial machines. A Rust backend drives the hardware directly over EtherCAT on a fixed cycle, while an Electron + React interface lets the operator run the machine. One codebase runs a range of machines, each built to a common pattern, so new machine types can be added without changing the core.

## Videos

| Software Demo | Full Explainer |
|---|---|
| [![Software Demo](https://img.youtube.com/vi/KI3YeBwfV-s/maxresdefault.jpg)](https://www.youtube.com/watch?v=KI3YeBwfV-s)<br/>*A demo of the control software in action* | [![Full Explainer](https://img.youtube.com/vi/55egCAkQgyM/maxresdefault.jpg)](https://youtu.be/55egCAkQgyM)<br/>*A complete overview of QiTech Control* |

## Documentation

The full documentation lives in the [Wiki](https://github.com/qitechgmbh/control/wiki):

- **Getting Started** — set up the development environment, run with and without hardware, install on a device
- **Architecture** — the real-time loop, the machine model, registry & identification, logging
- **Communication & API** — the command API and the live-data Socket.IO interface
- **Frontend** — the Electron + React application
- **Extending** — how to add a new machine
- **Reference & examples** — the example machines

## Repository structure

- `qitech_control/` — Runtime and composition: startup, the real-time loop, the REST and Socket.IO APIs, device discovery
- `machine_implementations/` — The concrete machines, each built to a common pattern
- `control-core/` — Shared mechanisms: controllers, converters, Socket.IO, Modbus, serial
- `control-core-derive/` — Procedural macros for `control-core`
- `electron/` — React + Electron frontend
- `nixos/` — NixOS system definition, including the real-time kernel and the device installer

The hardware-abstraction layer (EtherCAT device drivers and the typed IO layer) is provided by a separate library, `qitech_lib`, which this repository depends on.

## Technology

**Backend:** Rust, with [ethercrab](https://github.com/ethercrab-rs/ethercrab) for EtherCAT, [axum](https://docs.rs/axum) for the REST API, and [Socket.IO](https://socket.io/) for live data.

**Frontend:** [Electron](https://www.electronjs.org/) + [React](https://react.dev/), with [Radix](https://www.radix-ui.com/) components and [Tailwind](https://tailwindcss.com/) styling.

## Getting started

The project uses [Nix](https://nixos.org/) for a reproducible toolchain. Clone the repository and enter the development shell, which provides the correct Rust and Node versions and all system libraries:

```bash
git clone https://github.com/qitechgmbh/control.git
cd control
nix develop
```

From there you can run the backend (a `mock` feature lets you run without hardware) and the Electron frontend. See the **Getting Started** wiki page for the exact run commands.

## License

See [LICENSE](LICENSE).
