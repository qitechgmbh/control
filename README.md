![](./docs/github-banner.png)
# QiTech Control
QiTech Control is an industrial control panel software for the next generation of QiTech recycling machines built on top of Beckhoff Automation hardware.

# Repo Structure

Frontend
- `/electron`: Frontend code for the control software build with React and Electron.

Backend
- `/server`: Glue between Backhoff and Electron. Implements machine logic.
- `/stepper-driver`: Generic stepper driver for absolute, relative & speed movements with PID controllers.
- `/ethercat-hal`: Hardware abstraction layer for Beckhoff (and possibly other EtherCat) devices and protocols.
- `/ethercat-hal-derive`: Macros for `ethercat-hal`
- `/ethercrab`: Fork of `ethercrab` for ad-hoch changes.

Operating System
- `/nixos`: Custom Linux with realtime kernel & preconfigured for touchscreens.

Other
- `/docs`: Documentation for the project.


# Technology Choices

# Dev Setup

# Machines

| Machine Type | Version | Release Date | Description                 | Change to Previous Version                             | Vendor ID                  | Macine ID | Implemented | Docs                            |
| ------------ | ------- | ------------ | --------------------------- | ------------------------------------------------------ | -------------------------- | --------- | ----------- | ------------------------------- |
| Winder       | V1      | ???          | Winding Filaments & Similar | -                                                      | 1 (Qitech Industries GmbH) | 1         | Reserved    | -                               |
| Winder       | V2      | 2025         | Winding Filaments & Similar | Reengineered Traverse                                  | 1 (Qitech Industries GmbH) | 2         | In Progress | [](./docs/machines/winder-1.md) |
| Extruder     | V1      | ???          | Single Screw Extruder       | -                                                      | 1 (Qitech Industries GmbH) | 3         | Reserved    | -                               |
| Extruder     | V2      | 2025         | Single Screw Extruder       | PT100 Thermometers, Optional Additional Heating Zone 4 | 1 (Qitech Industries GmbH) | 4         | Not Yet     | [                               |
| Waterway     | V1      | 2025         | Filament Water Cooling      | -                                                      | 1 (Qitech Industries GmbH) | 5         | Not Yet     |                                 |

# More Docs

- [X] [Architecture & Data Flow](./docs/architecture-overview.md)
    - [X] Example Winder V2

- [ ] Machines
  - [ ] Table
  - [ ] Winder V2
    - [ ] Devices
    - [ ] Functionality
    - [ ] Logic
  - [ ] Extruder V2

- [ ] Electron
  - [ ] Folder Structure 
  - [ ] Routing with TanStack Router
  - [ ] Seign with Tailwind & Shadcn 

- [ ] Interfacing with Electron/Server
    - [ ] SocketIO
      - [ ] Rooms & Cacheing
        - [ ] Joining leaving rooms
        - [ ] Our rooms vs native socketIo rooms
        - [ ] Cacheing
          - [ ] Serverside Cacheing
          - [ ] Clientside Cacheing
      - [ ] Machine Room
      - [ ] Main Room
    - [ ] REST
      - [ ] Machine Mutations
      - [ ] Write Device Identification

- [ ] Server
  - [ ] Control Loop Setup
    - [ ] Control Loop Thread
      - [ ] Realtime
    - [ ] Maindevice
    - [ ] Group
    - [ ] Extracting Device Identifications
    - [ ] Identifying Groups
    - [ ] Validating Machines
    - [ ] Run Control Loop
  - [ ] Control Loop
    - [ ] Ethercat TX/RX
    - [ ] Reading Inputs
    - [ ] Calling Actors
    - [ ] Writing Outputs
  - [ ] Machine/Device Identification
    - [ ] SubDevice Identity
    - [ ] Identification design choices
    - [ ] Machine/Device Identification Values
    - [ ] Device Identification with EEPROM
    - [ ] Grouping Devices
    - [ ] Validating Device Groups to Machines
  - [ ] Machines
    - [ ] When to create a new Machine?
    - [ ] Creatine/Validating a Machine
      - [ ] Validation
      - [ ] Configuration

- [ ] Ethercat HAL
  - [ ] Devices/PDO
    - [ ] CoE Theory
      - [ ] Parameters
      - [ ] Pdo Assignment
    - [ ] PDO Theory
      - [ ] Pdo Objects
      - [ ] Tx / Rx Meaning
      - [ ] Presets
      - [ ] Mapping
    - [ ] How To: Implementing an EL2521
  - [ ] IO
    - [ ] How to: Implementing a Pulse Train Output

- [ ] Ethercat Basics
  - [ ] State Machine
  - [ ] Network Topology Packets
  - [ ] Master / Slave
  - [ ] Terms of Communication
    - [ ] SDO 
    - [ ] PDO 
    - [ ] Mailbox
    - [ ] CoE 
    - [ ] Commands
  - [ ] EEPROM

- [ ] Ethercat HAL
- [ ] Ethercat Devices
  - [ ] Buscoupler
  - [ ] Digital Input
    - [ ] EL2000er
  - [ ] Digital Output
    - [ ] EL1000er
  - [ ] Analog Input
    - [ ] EL3000er
    - [ ] Temperature Input

- [ ] Operating System
  - [ ] Why
  - [ ] How