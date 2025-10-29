# Laser DRE Device Documentation

## Overview
The **Laser DRE** is a precision measurement device designed for real-time diameter measurement of rotating objects. It communicates over a **USB Serial interface** using the **Modbus RTU** protocol.

Key features include:

- High-speed diameter measurement
- Dual-axis capability
- USB Serial communication
- Modbus RTU protocol support

---

## Technical Specifications

- **Communication Interface:** USB Serial
- **Protocol:** Modbus RTU
- **Axes:** 2-axis measurement
- **Polling Rate:** Every 16 ms
- **Measurement Rate:** 320 measurements per second (manufacturer specification)
- **Measurement Type:** Diameter measurement | X/Y Axis width

---

## Operation

### Polling
The device **polls the diameter every 16 milliseconds**, ensuring near real-time data acquisition for precise monitoring.

### Measurement
According to the manufacturer, the Laser DRE can measure the diameter **320 times per second**, providing high-resolution data for process control and analysis.

### Axes
The device is capable of measuring across **two axes**, allowing for comprehensive profiling of the object’s diameter and shape.

---

## Communication Protocol

The device uses **Modbus RTU** over the USB Serial connection. Key points for communication:

- **Master/Slave Model:** The Laser DRE acts as a Modbus slave device.
- **Function Codes:** Supports standard Modbus RTU function codes for reading measurements.
- **Data Format:** Diameter measurements are transmitted in the format specified by the manufacturer (consult the device manual for exact register addresses and scaling).
