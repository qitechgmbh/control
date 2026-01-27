# Environment Setup

Working on this repository requires you to use cargo and npm. In the following document the installation process will be shown

## Ubuntu/Debian Installation

```bash
    # Just press enter when prompted
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    apt install npm
    apt install nodejs
    # assuming you dont have git already, Git setup not included in here
    apt install git
    git clone git@github.com:qitechgmbh/control.git
    cd control
    cd electron
    npm install
```

# Running Backend and Frontend Linux

## Backend

To Compile backend code and run it on Linux:

```bash
./cargo_run_linux.sh
```

The script sets capabilities on the compiled binary like raw socket access.
Here it is required that you are connected to atleast one machine that is communicated with over ethercat or usb/serial.
The Most minimal working setup would be to connect over ethernet to an ek1100 beckhoff terminal.

## Mock-Machine

The Mock-Machine can be used to test code that does not require an actual machine connection.
Like Frontend Code for example.

Mock machines can be used if the compile feature mock-machine is enabled.

```bash
# in root git folder
./cargo_run_linux.sh mock-machine
```

## Frontend

To run the Frontend Code:

```bash
# need to be in the electron folder and an additional terminal
cd electron
npm run start
```

# Contributing

Generally you contribute to the codebase by:

1. Opening or Choosing an existing issue (Bug,Feature,Task etc)
2. Work on it locally
3. Commit your changes locally
4. Before pushing changes, if there were changes on the master branch, rebase your branch like so:

```bash
git fetch
git rebase origin/master
```

5. Push changes

```bash
# IF you needed to rebase
git push --force-with-lease
# ELSE you just push
git push
```

6. Open a Pull Request on Github and link the issue by writing fix #issue_number, after the pull request is merged into master the branch is closed automatically
7. Request a Review and hope for the best :)

# Recommended Editor Setup

We recommend you to use an Editor with rust-analyzer support like VSCode to speed up development and detect errors before compiling.

# Minimal Hardware Examples

To get started with actual hardware, check out these step-by-step tutorials:

- **[LED Control with EL2004](./minimal-example-el2004.md)** - Digital output control, the simplest possible hardware setup
- **[Analog Input with EL3021](./minimal-example-el3021.md)** - Reading analog current measurements
- **[Stepper Motor Control with EL7031-0030](./minimal-example-el7031-motor.md)** - Complete motor control integration with velocity control

These examples provide complete hardware wiring diagrams and software setup instructions.


# More Docs 

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
