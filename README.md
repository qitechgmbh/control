![](./docs/assets/github-banner.png)

# QiTech Control

QiTech Control is an industrial control panel software for the next generation of QiTech recycling machines built on top of Beckhoff Automation hardware.

[![](https://img.youtube.com/vi/KI3YeBwfV-s/maxresdefault.jpg)](https://www.youtube.com/watch?v=KI3YeBwfV-s)
*Click here to watch a video demo of our software.*

[![](https://img.youtube.com/vi/55egCAkQgyM/maxresdefault.jpg)](https://youtu.be/55egCAkQgyM) 
*Click here to watch a full explainer Video of our Software.*

# Repo Structure

Frontend

- `/electron`: Frontend code for the control software built with React and Electron.

Backend

- `/server`: Glue between Beckhoff and Electron. Implements machine logic.
- `/ethercat-hal`: Hardware abstraction layer for Beckhoff (and possibly other EtherCat) devices and protocols.
- `/ethercat-hal-derive`: Macros for `ethercat-hal`
- `/control-core`: Core control logic for the server.

Operating System

- `/nixos`: Custom Linux with realtime kernel & preconfigured for touchscreens.

Other

- `/docs`: Documentation for the project.

# Technology Choices

## Backend

To interface with Beckhoff and other EtherCAT devices we need an EtherCAT master stoftware. Possibilities are [PySOEM](https://github.com/bnjmnp/pysoem) (Python), [SOEM](https://github.com/OpenEtherCATsociety/SOEM) (C) and [Ethercrab](https://github.com/ethercrab-rs/ethercrab) (Rust). For realtime operation only C and Rust are suitable. We chose Rust because of safety and confidence in the written code.

[SocketIO](https://socket.io/) was chosen for performant event driven communication from the backend to the server. But we still use REST with [axum](https://docs.rs/axum/latest/axum/) for the communication thet benefits from the request/response model.

We use [Smol](https://github.com/smol-rs/smol) for EtherCAT IO in the control loop for it's performance and [Tokio](https://tokio.rs/) for server IO because of it's ecosystem and maturity.

## Frontend

We could combine the code of the frontend and backend using [Doxius](https://dioxuslabs.com/) but it lacks good Linux support. We chose [Electron](https://www.electronjs.org/) with [React](https://react.dev/) for it's maturity and ecosystem. For the UI we use [Shadcn](https://ui.shadcn.com/) components and [Tailwind](https://tailwindcss.com/) for styling. For routing we use [TanStack Router](https://tanstack.com/router/v1).

# Dev Setup

[Developer Documentation](./docs/developer-docs/)

## Backend

- Rust stable 1.86^ toolchain (install via [rustup](https://rustup.rs/))
- `rust-analyzer` extension for VSCode
- Set your interface in `server/src/ethercat/init.rs` like `en10`
- Connect a Beckhoff EK1100 to your interface
- run `cd server && cargo run` to start the server (localhost:3001)

## Frontend

- nodejs and npm installed
- run `cd electron && npm i && npm run start` to start the frontend

# Machines

| Machine Type | Version | Release Date | Description                 | Change to Previous Version                             | Vendor ID                  | Machine ID | Implemented | Docs                            |
| ------------ | ------- | ------------ | --------------------------- | ------------------------------------------------------ | -------------------------- | ---------- | ----------- | ------------------------------- |
| Winder       | V1      | ???          | Winding Filaments & Similar | -                                                      | 1 (Qitech Industries GmbH) | 1          | Reserved    | -                               |
| Winder       | V2      | 2025         | Winding Filaments & Similar | Reengineered Traverse                                  | 1 (Qitech Industries GmbH) | 2          | Yes         | [](./docs/machines/winder-1.md) |
| Extruder     | V1      | ???          | Single Screw Extruder       | -                                                      | 1 (Qitech Industries GmbH) | 3          | Reserved    | -                               |
| Extruder     | V2      | 2025         | Single Screw Extruder       | PT100 Thermometers, Optional Additional Heating Zone 4 | 1 (Qitech Industries GmbH) | 4          | Yes         |                                 |
| Waterway     | V1      | 2025         | Filament Water Cooling      | -                                                      | 1 (Qitech Industries GmbH) | 5          | In Progress     |                                 |
| Laser        | V1      | ???          | Diameter Measuring Laser    | -                                                      | 1 (Qitech Industries GmbH) | 6          | Yes         |                                 |
| Mock         | -       | ???          | Mock Machine for Testing    | -                                                      | 1 (Qitech Industries GmbH) | 7          | Yes         | -                               |

# More Docs    docs

/assets

- [x] [Architecture & Data Flow](./docs/architecture-overview.md)

  - [x] Example Winder V2

- [ ] Electron

  - Folder Structure
  - Routing with TanStack Router
  - Design with Tailwind & Shadcn
  - ...

- [ ] Interfacing with Electron/Server

  - [ ] SocketIO
    - Machine Namespace
    - Main Namespace
  - [ ] REST
    - Machine Mutations
    - Write Device Identification

- Server

  - [x] [Threading](./docs/control-loop.md)
  - [x] [Logging](./docs/logging.md)
  - [ ] Control Loop Setup
    - Control Loop Thread
      - [ ] realtime
    - Maindevice
    - Group
    - Extracting Device Identifications
    - Identifying Groups
    - Validating Machines
    - Run Control Loop
  - [x] [Control Loop](./docs/control-loop.md)
  - [x] [Machine/Device Identification](./docs/identification.md)
  - [ ] Machines
    - When to create a new Machine?
      - Versioning
      - Code sharing
    - Creating/Validating a Machine
      - Validation
      - Configuration
  - [ ] Machine Implementation Guide
    - Link: How to create a Device
    - Link: How to create an Actor
    - Link: How to create a Machine
      - API (SocketIO + REST)
      - Creation/Validation Logic
        - Optional/Mandatory Devices
        - Validate Devices
      - Business Logic
    - Link: How to create Machine Abstraction (Like Traverse/Puller/...)
    - Forward `act` in winder.

- [ ] Control Core

  - [x] [Actors](./docs/actors.md)
  - [ ] SocketIO
    - Namespaces & Caching
    - Joining leaving namespaces
    - NamespaceId
    - Caching
      - Serverside Caching
      - Clientside Caching
  - [ ] REST

- [x] Ethercat HAL

  - [x] [Devices](./docs/devices.md)
  - [x] [Configuration (CoE)](./docs/coe.md)
  - [x] [IO](./docs/io.md)
  - [x] [PDO](./docs/pdo.md)

- [x] [Ethercat Basics](./docs/ethercat-basics.md)

- [x] [NixOS Operating System](./docs/nixos/README.md)


## Minimal Example — LED Control on the EL2004  
A complete hardware + software walkthrough

---

## Table of Contents
1. [Introduction](#1-introduction)  
2. [Requirements](#2-requirements)  
3. [Hardware Setup](#3-hardware-setup)  
4. [Software Setup](#4-software-setup)  
5. [Demo](#5-demo)  
6. [References](#6-references)  
7. [Acknowledgements](#7-acknowledgements)

---

# 1. Introduction

The EL2004 LED Toggle Example is a minimal demonstration showing how to control digital outputs on a **Beckhoff EL2004 EtherCAT terminal** using the QiTech machine framework.  
It represents the simplest possible hardware interaction in the system:  
**toggling LED outputs using the QiTech Control Dashboard.**

---

# 2. Requirements

## Hardware
- Beckhoff **EL2004 EtherCAT Terminal** (4-channel digital output)  
- Beckhoff **EK1100 EtherCAT Coupler**  
- **24 V DC power supply** (AC/DC adapter + DC hollow plug)  
- Jumper / bridge wires (0.5–1.5 mm² recommended)  
- A **Linux PC** (Ubuntu/Debian recommended)  
- Standard Ethernet cable  
- Flat screwdriver  

## Software  
*(Installation steps in Section 4)*  
- Rust toolchain  
- Node.js + npm  
- Git  
- QiTech Control repository  
- EtherCAT HAL (included inside repo)

---

# 3. Hardware Setup

## 3.1 Schematic

> Replace this path with your actual asset path  
> Example: `./docs/assets/schematic.png`

![](./docs/assets/schematic.png)

---

## 3.2 EK1100 Wiring

This wiring configuration powers the EL2004 and prepares it for LED control.  
It is not the only possible wiring but is the **simplest functional setup**.

### ⚠️ Safety Warning  
Always disconnect power before wiring.  
Working on live EtherCAT terminals can cause serious damage or electrical shock.

---

## 3.2.1 Safe Wiring Procedure (Beckhoff Recommended)

1. Insert a screwdriver **straight** into the square release hole.  
2. Insert the stripped wire into the round opening.  
3. Remove the screwdriver — the spring clamp locks the wire.

![](./docs/assets/wiring.png)

---

## 3.2.2 Wiring Steps (Used in This Example)

We supply power using a **DC hollow-plug adapter**, like this one:  
https://www.amazon.de/dp/B093FTFZ8Q

Perform the following wiring on the EK1100:

1. Red wire **(+24 V)** → Terminal **2**  
2. Black wire **(0 V)** → Terminal **3**  
3. Jumper wire from **Terminal 1 → Terminal 6**  
4. Jumper wire from **Terminal 5 → Terminal 7**  

After wiring, your module should look like **Figure 1**.

---

### **Figure 1 — EK1100 Minimal Wiring**
![](./docs/assets/ek1100.jpeg)

---

## 3.3 EL2004 Integration

Slide the EL2004 onto the **right side of the EK1100** until it locks.  
The EtherCAT E-Bus and power contacts connect automatically — **no wiring required**.

### **Figure 2 — EL2004 Terminal**
![](./docs/assets/el2004.jpeg)

---

## 3.4 Final Assembled Setup

### **Figure 3 — EK1100 + EL2004 Connected**
![](./docs/assets/complete.jpeg)

---

## 3.5 Power & Ethernet

### Power  
Connect the 24 V adapter to the hollow plug used earlier.
**Example AC/DC Adapter (Figure 4):**

![](./docs/assets/adapter.jpeg)
### Ethernet  
Use a standard LAN cable to connect your PC → EK1100.
The final powered up and connected setup should look like this:

![](./docs/assets/power.jpeg)

---

# 4. Software Setup

## 4.1 Installing on Ubuntu/Debian

Paste this into your terminal:

```bash
# Press Enter when prompted
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

sudo apt update
sudo apt install -y npm nodejs git

git clone git@github.com:qitechgmbh/control.git
cd control/electron
npm install
```
## 4.2 Running the Backend
```bash
./cargo_run_linux.sh
```
This script:

- Builds the backend

- Grants required system capabilities (raw sockets)

- Starts EtherCAT communication

Ensure the EK1100 is connected.

## 4.3 Running the Frontend
```bash
cd electron
npm run start
```
This launches the QiTech Control dashboard.
# 5. Demo

## 5.1 Assigning Devices in the Dashboard

Once the backend + frontend are running, you should see:

    EK1100 Coupler

    EL2004 Digital Output Terminal
![](./docs/assets/discovery.png)

Steps:

1.Click Assign on the EK1100

2.Select TestMachine V1
![](./docs/assets/setmachine.png)

3.Enter a serial number (use the same for EK1100 + EL2004)
![](./docs/assets/setserial.png)

4.Click Write

5.Repeat for the EL2004

## 5.2 Testing LED Control
Navigate to:

Machines → TestMachine
![](./docs/assets/machinedetected.png)
You will see this interface:
![](./docs/assets/machinecontrol.png)
You can now toggle the four digital outputs of the EL2004.
# 6. References

This guide incorporates information from official Beckhoff documentation.
All diagrams, product names, and figures belong to Beckhoff Automation GmbH & Co. KG and are used here solely for educational purposes.

Referenced Manuals

EK1100 / EK1501 EtherCAT Coupler Documentation
[Beckhoff EK1100 Documentation](https://download.beckhoff.com/download/Document/io/ethercat-terminals/ek110x_ek15xx_en.pdf)

EL2004 Digital Output Terminal Documentation
[Beckhoff EL2004 Documentation](https://download.beckhoff.com/download/Document/io/ethercat-terminals/el20xx_el2124_de.pdf)

# 7.Acknowledgements

This tutorial is inspired by the clarity and educational quality of Beckhoff manuals.
All wiring illustrations and hardware descriptions in this guide are provided for demonstration purposes only and do not replace official Beckhoff installation guidelines.

Special thanks to the QiTech engineering team for providing the backend architecture, EtherCAT HAL abstraction, and the TestMachine framework that makes this example possible.
