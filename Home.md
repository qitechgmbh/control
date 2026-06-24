# QiTech Control

QiTech Control is a real-time control system for industrial machines. A Rust backend drives the hardware directly over EtherCAT on a fixed 1 ms cycle, while an Electron + React interface lets the operator run the machine. One codebase runs a range of machines, each built to a common pattern, so new machine types can be added without changing the core.

## Overview

The backend is split into two sides that never block each other. A real-time side steps every machine once per cycle and exchanges process data with the hardware. A non-real-time side serves the operator interface, the live-data feed, and device discovery. The two are connected only by message-passing channels, so the real-time cycle is never stalled by the interface or the network.

The real-time work runs on two isolated CPU cores at real-time priority, one core runs the control loop and the machines, the other handles EtherCAT IO - which keeps the 1 ms cycle stable.

The hardware-abstraction layer (the EtherCAT device drivers and the typed IO layer) and the core machine traits are provided by a separate library, `qitech_lib`, which this repository depends on. This wiki documents the `control` repository.

→ See **Architecture** for the full picture.

## FAQ

**What does it run on?**
A Linux system with a real-time kernel. It is deployed as a NixOS system that ships the kernel and the device setup. See **Getting Started**.

**What hardware does it support?**
Standard EtherCAT terminals (for example from Beckhoff and WAGO), reached over the EtherCAT bus. Serial and Modbus transports are also available, see **Other transports**.

**Can I run it without hardware?**
Yes. A mock mode runs the backend with no machine connected, so the interface and the machine logic can be developed on any computer. See **Getting Started**.

**How do I control a machine programmatically?**
Through the REST API for commands and the Socket.IO interface for live data. See **Communication & API**.

**How do I add a new machine?**
Start from the boilerplate template in `machine_implementations` and fill in the marked steps. See **Extending**.

**Where does the hardware layer live?**
The EtherCAT device drivers, the typed IO layer, and the core machine traits are in `qitech_lib` — a separate library this repository depends on, not in `control`.
