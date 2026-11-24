<img width="1280" height="537" alt="grafik" src="https://github.com/user-attachments/assets/7fa67de4-772e-4528-bd6d-72b56380cad7" />


# QiTech Control

QiTech Control is an open-source framework designed to bring modern software development practices to certified industrial hardware.

It frees developers from proprietary, license-heavy PLC ecosystems and rigid "point-and-click" workflows that are no longer adequate for today's complex automation challenges.

QiTech Control combines the modularity and reliability of standard EtherCAT terminals (e.g., WAGO, Beckhoff) with the power of a modern Rust & React stack.

Control is the top of a three-layer stack: the QiTech machines, API and UI live here, the machine runtime in [qitech_framework](https://github.com/qitechgmbh/qitech_framework), and the hardware layer (EtherCAT, terminal drivers, Modbus) in [qitech_lib](https://github.com/qitechgmbh/qitech_lib).

## Documentation

**[View Full Documentation Wiki](https://github.com/qitechgmbh/control/wiki)**

- [Getting Started](https://github.com/qitechgmbh/control/wiki/Getting-Started)
- [Architecture](https://github.com/qitechgmbh/control/wiki/Architecture)
- [API Reference](https://github.com/qitechgmbh/control/wiki/API)
- [Adding a Machine](https://github.com/qitechgmbh/control/wiki/Adding-A-Machine)

## Videos

| Software Demo | Full Explainer |
|---------------|----------------|
| [![](https://img.youtube.com/vi/KI3YeBwfV-s/maxresdefault.jpg)](https://www.youtube.com/watch?v=KI3YeBwfV-s)<br/>*Watch a demo of the control software in action* | [![](https://img.youtube.com/vi/KwC9g2Vn_Lc/maxresdefault.jpg)](https://www.youtube.com/watch?v=KwC9g2Vn_Lc)<br/>*Watch a complete overview of QiTech Control* |

## Repository Structure

- **`/qitech_control`** - Rust backend: machines, schemas and the REST/Socket.IO API
- **`/qitech_control_core`** - Controllers used by the machines (PID, heating, motion)
- **`/electron`** - React + Electron frontend
- **`/nixos`** - Custom Linux OS with realtime kernel

## Technology Stack

**Backend:** Rust on [qitech_framework](https://github.com/qitechgmbh/qitech_framework) and [qitech_lib](https://github.com/qitechgmbh/qitech_lib), [axum](https://docs.rs/axum/latest/axum/) for REST API, [SocketIO](https://socket.io/) for real-time communication

**Frontend:** [Electron](https://www.electronjs.org/) + [React](https://react.dev/) with [Shadcn](https://ui.shadcn.com/) components and [Tailwind](https://tailwindcss.com/) styling

## Hardware

QiTech Control uses standard EtherCAT terminals from WAGO, Beckhoff and others, plus Modbus RTU devices. Terminal drivers live in qitech_lib ([EtherCAT HAL](https://github.com/qitechgmbh/qitech_lib/wiki/EtherCAT-HAL)), minimal hardware examples in qitech_framework ([digital output](https://github.com/qitechgmbh/qitech_framework/wiki/Minimal-Example-Digital-Output-Video-Script), [WAGO digital I/O](https://github.com/qitechgmbh/qitech_framework/wiki/Minimal-Example-Wago-Digital-IO)).

## QiTech Machines

QiTech Control powers 10+ production machines for filament extrusion, winding, measurement, and material processing.

**[View complete machine catalog on the wiki](https://github.com/qitechgmbh/control/wiki/QiTech-Machines)**


## Contributing

This is an open-source project. Contributions are welcome! Please see [Contributing](https://github.com/qitechgmbh/control/wiki/Contributing) for development guidelines.



## Additional Videos

| Episode Highlights |  |
|--------------------|--------------------|
| [![](https://img.youtube.com/vi/U2wNN6sF4to/maxresdefault.jpg)](https://www.youtube.com/watch?v=U2wNN6sF4to&t=163s)<br/>**Day in the Software Team**<br/>*Behind the scenes with Robin and how the team ships control software.* | [![](https://img.youtube.com/vi/R5WGAY3WRWA/maxresdefault.jpg)](https://www.youtube.com/watch?v=R5WGAY3WRWA&t=1s)<br/>**Software Stack Day**<br/>*A fast tour through the tools, stack, and daily workflow.* |
| [![](https://img.youtube.com/vi/xyaHwxXguw4/maxresdefault.jpg)](https://www.youtube.com/watch?v=xyaHwxXguw4&t=32s)<br/>**First Client**<br/>*A milestone week building the first real client project.* | [![](https://img.youtube.com/vi/moI7z2PVdF4/maxresdefault.jpg)](https://www.youtube.com/watch?v=moI7z2PVdF4)<br/>**Steelworks Week**<br/>*A deep-dive week on a demanding steelworks order.* |

## License

See [LICENSE](LICENSE) file for details.
