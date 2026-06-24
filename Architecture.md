# Architecture

QiTech Control is built from three parts that run independently and exchange data only by passing messages: the **EtherCAT loop**, the **machine loop**, and the **async runtime**. The machine loop sits in the middle, it exchanges the process image with the EtherCAT loop below it, and exchanges commands and live values with the async runtime above it. The EtherCAT loop and the async runtime never talk to each other directly. Keeping the three separate is what lets the hardware run on a fixed cycle while the operator interface and the network run freely.

## The EtherCAT loop

`qitech_lib`'s EtherCAT application keeps the bus cycling. Every cycle it sends and receives the process image and holds all devices in lockstep with **distributed clocks**, a 1 ms cycle, where `sync0_period` is the cycle time and `sync0_shift` is half of it. It runs on **two reserved CPU cores at real-time priority** (the bus loop on core 2, IO on core 3, priority 99), so the bus timing is never disturbed by the rest of the system (`qitech_control/src/main.rs`). It exchanges data with the rest of the system only through the shared process image that the machine loop reads and writes.

## The machine loop

The machine loop is the meeting point of the three parts. Each pass it (`qitech_control/src/machine_loop.rs`):

1. **Reads inputs**, takes the latest EtherCAT process image and distributes it to the devices.
2. **Runs the machines** (`run_machines`), each machine applies any commands waiting in its mailbox, updates its logic, and drives its devices' outputs.
3. **Writes outputs**, collects the devices' outputs and sends them back out over EtherCAT.

This is where commands and live values cross between the hardware and the outside world: the machines pull commands from their mailboxes (filled by the async runtime) and publish their state back out (consumed by the async runtime). If a machine reports a recoverable or an irrecoverable failure, the loop removes it rather than letting one machine stall the others.

## The async runtime

The async runtime is a [tokio](https://tokio.rs/) runtime that serves the non-real-time work: the REST API, the Socket.IO live-data feed, and device discovery. It communicates only with the machines in the machine loop, it delivers commands into each machine's mailbox and receives the machines' live values to stream out to clients. It never touches the EtherCAT loop's data directly, which is what keeps the real-time loops free to run on time.

## How a machine fits in

Each machine is an isolated unit with its own mailbox. Commands arrive as messages and are applied on the next pass of the machine loop, nothing calls a machine directly across the boundary. Every machine implements a common trait, defined in `qitech_control`, the hardware library this repository depends on, so the machine loop can step them all uniformly without knowing their concrete types. Each machine's state and command surface (its API) lives in `machine_implementations`.

## The layers

The backend is one Cargo workspace, layered so each crate depends only on those beneath it:

| Crate | Responsibility |
|---|---|
| `qitech_control` | Runtime and composition: startup, the EtherCAT and machine loops, the REST and Socket.IO APIs, device discovery |
| `machine_implementations` | The concrete machines, each built to a common pattern |
| `control-core` | Shared mechanisms: controllers, converters |
| `control-core-derive` | Procedural macros for `control-core` |

The hardware-abstraction layer (the EtherCAT device drivers, the typed IO layer) and the core machine traits come from a separate dependency, `qitech_lib`.

## Transports

EtherCAT is the real-time backbone, reached through the `ethercrab` master and the device layer in `qitech_lib`. The system also communicates over serial and Modbus (`control-core`), see **Other transports**.
