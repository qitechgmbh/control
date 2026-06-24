# Getting Started

This guide sets up a development environment, runs QiTech Control with and without hardware, and outlines deploying it to a device.

## Prerequisites

The only prerequisite is [Nix](https://nixos.org/) with flakes enabled. Everything else — the right Rust and Node versions, Electron, and the required system libraries — is provided by the project's development shell, so you do not install them yourself.

## Get the code

```bash
git clone https://github.com/qitechgmbh/control.git
cd control
nix develop
```

`nix develop` drops you into a shell with the full toolchain (Rust, Node 22, Electron, and the system libraries the build needs). Run it once in each terminal you work in.

## Run it

The backend and the frontend are two separate processes, so start them in **two separate terminals** — each inside `nix develop`.

**Backend — mock mode (no hardware).** A `mock` feature runs the backend with no machine connected, which is all you need to develop the interface and the machine logic:

```bash
cargo run -p qitech_control --features mock
```

**Frontend.** In the `electron/` directory:

```bash
cd electron
npm install
npm start
```

`npm start` builds and launches the Electron application, which connects to the running backend.

## Running against real hardware

Talking to EtherCAT hardware requires elevated network capabilities on the host. On a deployed device these are granted automatically by the QiTech NixOS module (see below), so the system runs against real terminals out of the box.

## Install on a device

In production, QiTech Control runs as a NixOS system defined under `nixos/`. The QiTech NixOS module configures the real-time kernel, the runtime, and the network capabilities the EtherCAT stack needs. An installer image is built with `nixos-build-iso.sh`, and a device is provisioned with `nixos-install.sh` and `nixos-setup.sh`.

For a step-by-step walkthrough of installing on a device, watch the video guide:

[![Install walkthrough](https://img.youtube.com/vi/7b6CcfBpeEk/maxresdefault.jpg)](https://www.youtube.com/watch?v=7b6CcfBpeEk&t=644s)
