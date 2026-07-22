# Laser Manual

The **Laser DRE** is a precision measurement device designed for real-time diameter measurement of rotating objects. It communicates over a **USB Serial interface** using the **Modbus RTU** protocol.

Key features include:

- High-speed diameter measurement
- Single- or dual-axis capability
- USB Serial communication
- Modbus RTU protocol support

## Technical Specifications

- **Communication Interface:** USB Serial
- **Protocol:** Modbus RTU
- **Axes:** 1-axis or 2-axis measurement (depending on model)
- **Polling Rate:** Every 16 ms
- **Measurement Rate:** 320 measurements per second (manufacturer specification)
- **Measurement Type:** Diameter measurement | X-Axis width (1-axis) | X/Y-Axis width + Roundness (2-axis)

## Operation

### Polling

The device **polls the diameter every 16 milliseconds**, ensuring near real-time data acquisition for precise monitoring.

### Measurement

According to the manufacturer, the Laser DRE can measure the diameter **320 times per second**, providing high-resolution data for process control and analysis.

### Axes

Depending on the model, the device measures across **one or two axes**. The 2-axis variant enables comprehensive profiling of both the diameter and the cross-sectional shape of the object.

## Communication Protocol

The device uses **Modbus RTU** over the USB Serial connection.

Key points:

- **Master/Slave Model:** The Laser DRE acts as a Modbus slave device.
- **Function Codes:** Supports standard Modbus RTU function codes for reading measurements.
- **Data Format:** Diameter measurements are transmitted in the format specified by the manufacturer (consult device manual for register addresses and scaling).

## Settings

The device includes a small display that allows changing several operational settings:

### Units

The laser can operate in **millimeters (mm)** or **inches (in)**.
The QiTech Control software uses **millimeters**.

### Alarm

The laser can emit an audible alarm if the measured diameter is out of tolerance.
**Note:** The tolerance settings on the laser may differ from the tolerance configured in our software.

### Communication

Several communication parameters can be configured:

- **Set UART:** Configure **parity (8N1)** and **baud rate (38400)**.
- **Set Protocol:** Choose the communication protocol. _(We use Modbus RTU.)_
- **Set Slave Address:** The slave address of our laser is **1**.

### Calibration

The laser can be calibrated using **4 mm** and **8 mm** reference rods.
