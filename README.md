![](./docs/assets/github-banner.png)

# QiTech Control

QiTech Control is an open-source framework designed to bring modern software paradigms to certified industrial hardware.

It frees developers from proprietary, license-heavy PLC ecosystems and rigid "point-and-click" workflows that are no longer adequate for today's complex automation challenges.

QiTech Control combines the modularity and reliability of standard EtherCAT terminals (e.g., WAGO, Beckhoff) with the power of a modern Rust & React stack.


[![](https://img.youtube.com/vi/KI3YeBwfV-s/maxresdefault.jpg)](https://www.youtube.com/watch?v=KI3YeBwfV-s)
*Click here to watch a video demo of our software.*

[![](https://img.youtube.com/vi/55egCAkQgyM/maxresdefault.jpg)](https://youtu.be/55egCAkQgyM) 
*Click here to watch a full explainer Video of our Software.*

# Repo Structure

**Frontend**

- `/electron`: Frontend code for the control software built with React and Electron.

**Backend**

- `/server`: Glue between Beckhoff and Electron. Implements machine logic.
- `/ethercat-hal`: Hardware abstraction layer for Beckhoff (and possibly other EtherCat) devices and protocols.
- `/ethercat-hal-derive`: Macros for `ethercat-hal`
- `/control-core`: Core control logic for the server.

**Operating System**

- `/nixos`: Custom Linux with realtime kernel & preconfigured for touchscreens.

**Other**

- `/docs`: Documentation for the project.

# Technology Choices

## Backend

To interface with Beckhoff and other EtherCAT devices we need an EtherCAT master software. Possibilities are [PySOEM](https://github.com/bnjmnp/pysoem) (Python), [SOEM](https://github.com/OpenEtherCATsociety/SOEM) (C) and [Ethercrab](https://github.com/ethercrab-rs/ethercrab) (Rust). For realtime operation only C and Rust are suitable. We chose Rust because of safety and confidence in the written code.

[SocketIO](https://socket.io/) was chosen for performant event driven communication from the backend to the server. But we still use REST with [axum](https://docs.rs/axum/latest/axum/) for the communication thet benefits from the request/response model.

We use [Smol](https://github.com/smol-rs/smol) for EtherCAT IO in the control loop for it's performance and [Tokio](https://tokio.rs/) for server IO because of it's ecosystem and maturity.

## Frontend

We could combine the code of the frontend and backend using [Doxius](https://dioxuslabs.com/) but it lacks good Linux support. We chose [Electron](https://www.electronjs.org/) with [React](https://react.dev/) for it's maturity and ecosystem. For the UI we use [Shadcn](https://ui.shadcn.com/) components and [Tailwind](https://tailwindcss.com/) for styling. For routing we use [TanStack Router](https://tanstack.com/router/v1).

# Dev Setup

[Developer Documentation](./docs/developer-docs/)

[API Documentation](https://github.com/qitechgmbh/control/blob/master/docs/rest-api.md)

-> Backend
- Rust stable 1.86^ toolchain (install via [rustup](https://rustup.rs/))
- `rust-analyzer` extension for VSCode
- Set your interface in `server/src/ethercat/init.rs` like `en10`
- Connect a Beckhoff EK1100 to your interface
- run `cd server && cargo run` to start the server (localhost:3001)

-> Frontend
- nodejs and npm installed
- run `cd electron && npm i && npm run start` to start the frontend


## Minimal Hardware Examples

For complete step-by-step tutorials on setting up your first hardware, including wiring diagrams and software configuration, see the [Getting Started Guide](./docs/developer-docs/getting-started.md#minimal-hardware-examples).

| Example            | Vendor   | Hardware      | Description                             | How-To Video | Docs                                                          |
| :----------------- | :------- | :------------ | :-------------------------------------- | :----------- | :------------------------------------------------------------ |
| **Digital Input**  | WAGO     | 750-402       | 4-Channel Digital Input                 | [ ]          | [Docs](./docs/developer-docs/minimal-example-wago750-402.md) |
| **Digital Output** | WAGO     | 750-753       | 8-Channel Digital Output                | [ ]          | -                                                             |
| **Analog Input**   | WAGO     | 750-455       | 4-Channel Analog Input                  | [ ]          | [Docs](./docs/developer-docs/minimal-example-wago-750-455.md)|
| **Analog Output**  | WAGO     | 750-553       | 4-Channel Analog Output                 | [ ]          | -                                                             |
| **Serial Comms**   | WAGO     | 750-652       | RS-485 Modbus Module                    | [ ]          | -                                                             |
| **Stepper Drive**  | WAGO     | 750-67x       | Stepper Controller                      | [ ]          | -                                                             |
| **Power Supply**   | WAGO     | 2787-214      | Power Supply Unit                       | [ ]          | -                                                             |
| **LED Control**    | Beckhoff | EL2004        | Digital output control (simplest setup) | [ ]          | [Docs](./docs/developer-docs/minimal-example-el2004.md)       |
| **Analog Input**   | Beckhoff | EL3021        | Reading analog current measurements     | [ ]          | [Docs](./docs/developer-docs/minimal-example-el3021.md)       |

# QiTech Machines

| Machine Name      | Description                 | Machine ID | Implemented | Video                                                           | Docs                                  |
| :---------------- | :-------------------------- | :--------- | :---------- | :-------------------------------------------------------------- | :------------------------------------ |
| Winder V1         | Winding Filaments & Similar | 1          | Legacy      | [Video](https://youtu.be/4aE4cFhioKA?si=Xdj_LVnFrAYWnLm6)       | -                                     |
| Winder V2         | Winding Filaments & Similar | 2          | Yes         | [Video](https://youtu.be/f2kzh6kpQWE?si=HEYcLOaC9gWSp2Wo)       | -                                     |
| XL Winder V1      | Large Scale Winder          | 3          | Yes         | [Video](https://youtu.be/ynI6ioWIQQY?si=4EDVKXbgIqrvGGtY)       | -                                     |
| Buffer V1         | Filament buffering system   | 4          | In Progress | [Video](https://youtu.be/VR5mEdZDPA0?si=eip7dQXU4zaXK7Ev)       | -                                     |
| Extruder V1       | Single Screw Extruder       | 5          | Legacy      | [Video](https://youtu.be/gchoG-yGczI?si=bEM4daf5eOVaU5_2)       | -                                     |
| Extruder V2       | Single Screw Extruder       | 6          | Yes         | [Video](https://youtu.be/mexRYGDNWa4?si=h7JBz_XKMwKLPenG)       | -                                     |
| Waterway V1       | Filament Water Cooling      | 7          | In Progress | [Video](https://youtu.be/_T5z1J8bl_k?si=uOC1hIQ1EAP0cHIF)       | -                                     |
| Laser V1          | Diameter Measuring Laser    | 8          | Yes         | [Video](https://youtu.be/WDM34lj4afM?si=MZzUKkHCrzH4P0aA)       | [Docs](./docs/machines/laser-DRE.md)  |
| 2-Axis-Laser V1   | Dual Axis Laser             | 9          | Yes         | [Video](https://youtu.be/WDM34lj4afM?si=MZzUKkHCrzH4P0aA)       | -                                     |
| Mock              | Mock Machine for Testing    | 10         | Yes         |                                                                 | -                                     |
| Extruder V3       | Single Screw Extruder       | 11         | Yes         | [Video](https://youtu.be/ipHHuPzCvn4?si=hkZ1b93rVuwDxhFD)       | -                                     |
| Mini Schredder V1 | mini plastic crusher        | 12         | Legacy      | [Video](https://youtu.be/m8NplNqdu2Q?si=x6zjDktJtpbSMSmu)       | -                                     |
| Pro Schredder V1  | large plastic crusher       | 13         | Legacy      | [Video](https://youtu.be/pSaVMqp06pU?si=y61enutRpxKscBm4)       | -                                     |
| Dryer V1          | polymer dryer               | 14         | Reserved    | [Video](https://youtu.be/6hdmUUAdZp0?si=eKOW1WlgkqTpnfdS)       | -                                     |
| Pelletizier V1    | Filament chopper            | 15         | Reserved    |                                                                 | -                                     |

# Current Restructuring Efforts

[![](https://img.youtube.com/vi/UlbnSVIhfLI/maxresdefault.jpg)](https://www.youtube.com/live/UlbnSVIhfLI?si=ZTotC5B7gd87tUim)
*Video: Live stream of software dev meeting*
