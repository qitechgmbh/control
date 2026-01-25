# Minimal Example — Analog Input Wago 750-455

## Table of Contents
1. [Introduction](#1-introduction)
2. [Requirements](#2-requirements)
3. [Hardware Setup](#3-hardware-setup)
4. [Software Setup](#4-software-setup)
5. [Demo](#5-demo)
6. [References](#6-references)

## 1. Introduction
The Wago 750-455 Analog Input example demonstrates how to measure analog current on a **Wago 750-455 EtherCAT module** using the QiTech machine framework. This module provides 4 independent analog input channels that can read 4-20mA signals.

## 2. Requirements

### Hardware
- **Wago 750-354 EtherCAT Coupler**
- **Wago 750-602 Power Supply** (24V VDC)
- **Wago 750-455 EtherCAT Module** (4-channel analog input, 4-20mA)
- **Wago 750-600 End Module** 
- **24 V DC power supply** for the Wago system
- **Adjustable DC power supply** (or any other means of producing 4-20mA DC @ <10V)
- Jumper / bridge wires (0.5–1.5 mm² recommended)
- A **Linux PC** (Ubuntu/Debian recommended)
- Standard Ethernet cable
- Flat screwdriver

### Software
*(Installation steps in Section 4)*
- Rust toolchain
- Node.js + npm
- Git
- QiTech Control repository
- EtherCAT HAL (included inside repo)

## 3. Hardware Setup

### 3.1 Overview
The Wago 750-455 is a 4-channel analog input module that measures current signals in the 4-20mA range. Each channel has its own input terminals and can operate independently.

#### ⚠️ Safety Warning
Always disconnect power before wiring.
Working on live EtherCAT modules can cause serious damage or electrical shock.

### 3.2 Wiring Procedure

#### 3.2.1 Safe Wiring (Wago Spring Clamp)
1. Insert a screwdriver **straight** into the square release hole.
2. Insert the stripped wire into the round opening.
3. Remove the screwdriver — the spring clamp locks the wire.

![](../assets/wiring.png)

---

### 3.3 Wago 750-455 Module Wiring

The Wago 750-455 has 4 analog input channels. Each channel requires two terminals:

**Channel 1 (AI1):**
- Terminal 1: ⊕ (Positive input)
- Terminal 2: ⊖ (Negative input/Ground)

**Channel 2 (AI2):**
- Terminal 3: ⊕ (Positive input)
- Terminal 4: ⊖ (Negative input/Ground)

**Channel 3 (AI3):**
- Terminal 5: ⊕ (Positive input)
- Terminal 6: ⊖ (Negative input/Ground)

**Channel 4 (AI4):**
- Terminal 7: ⊕ (Positive input)
- Terminal 8: ⊖ (Negative input/Ground)

For this minimal example, we'll connect at least one channel to an adjustable power supply:

1. Wire adjustable power supply ⊕ to **Terminal 1** (Channel 1 positive)
2. Wire adjustable power supply ⊖ to **Terminal 2** (Channel 1 negative)

*Repeat for additional channels as needed.*

---

### 3.4 Wago I/O System Integration

The Wago 750-455 module plugs into a Wago fieldbus system. Consult your specific Wago controller/coupler documentation for:
- Power supply requirements
- Network configuration
- Module placement in the I/O rack

The module automatically connects to the backplane for EtherCAT communication and power.

---

### 3.5 Power & Ethernet

#### Power
Ensure your Wago system has adequate 24V DC power connected per manufacturer specifications.

#### Ethernet
Connect your PC to the Wago controller/coupler using a standard Ethernet cable.

---

## 4. Software Setup

### 4.1 Installing on Ubuntu/Debian

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

### 4.2 Running the Backend

```bash
./cargo_run_linux.sh
```

This script:
- Builds the backend
- Grants required system capabilities (raw sockets)
- Starts EtherCAT communication

Ensure the Wago controller is connected and powered.

### 4.3 Running the Frontend

```bash
cd electron
npm run start
```

This launches the QiTech Control dashboard.

---

## 5. Demo

### 5.1 Assigning Devices in the Dashboard

Once both backend and frontend are running:

1. Navigate to **Setup → EtherCAT** page
2. You should see your Wago controller and the 750-455 module listed
3. Select the Device called 750-354 and assign the Wago AI Test V1 Machine, set the Serial to 1 and set the device role to 0 (Wago Bus Coupler)
4. Press write thn restart the backend to load the changes

### 5.2 Testing Analog Input

1. Set your adjustable power supply to output a current in the 4-20mA range
2. In the dashboard, navigate to the monitoring view
3. Observe the analog input value for the connected channel(s)

### 5.3 Channel Specifications

The Wago 750-455 provides:
- **Input Range:** 4-20mA
- **Wiring Error Detection:** Built-in detection for open/short circuit conditions
- **4 Independent Channels:** Each can be read separately

### 5.4 Error Detection

The module includes wiring error detection. If you see a wiring error flag:
- Check that your current source is within the 4-20mA range
- Verify all wire connections are secure
- Ensure the current loop is properly closed

---

## 6. References

### Wago Documentation
- [Wago 750-455 Product Page](https://www.wago.com/de-en/io-systems/4-channel-analog-input/p/750-455)
- Wago I/O System manual
- EtherCAT configuration guide for Wago systems

### QiTech Control
- [Architecture Overview](../architecture-overview.md)
- [EtherCAT Basics](../ethercat-basics.md)
- [Device Documentation](../devices.md)
- [Getting Started Guide](getting-started.md)

### Related Examples
- [Minimal Example — EL3021](minimal-example-el3021.md) (Single channel analog input)
- [Minimal Example — EL2004](minimal-example-el2004.md) (Digital output)