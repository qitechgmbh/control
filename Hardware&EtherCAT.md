# Hardware & EtherCAT

EtherCAT is the real-time fieldbus that connects the controller to the IO terminals. This page covers how the `control` repository uses EtherCAT. The device drivers themselves — the typed IO layer and the per-device PDO/CoE handling — live in `qitech_lib` and are out of scope for this wiki.

## The EtherCAT master

`control` talks to the bus through an EtherCAT master (the [ethercrab](https://github.com/ethercrab-rs/ethercrab) library, depended on via `control-core`). At startup it selects a network interface, brings up the master, and the master enumerates the devices present on the bus.

## Real-time IO

EtherCAT IO runs on its own reserved CPU core at real-time priority (core 3, priority 99), separate from the control loop (core 2). The transport uses `io_uring` for low-latency send and receive. The bus is held in lockstep with **distributed clocks**: a sync pulse every cycle, shifted by half the period (`sync0_period` is the cycle time, `sync0_shift` is half of it). See **Architecture** for how the loop reads inputs, runs the machines, and writes outputs each cycle.

## Interface discovery

The controller selects the EtherCAT network interface automatically: it lists the host's Ethernet interfaces and tests them for a link. `control-core` also marks the chosen interface as unmanaged, so the operating system's network manager leaves it alone before the master takes it over.

## The device layer

The actual device support — the hardware-abstraction layer, the typed IO mapping for each terminal, and the PDO/CoE handling — lives in the separate `qitech_lib` library that this repository depends on. `control` orchestrates those devices every cycle (reading their inputs, driving their outputs) but does not define them. For the device drivers themselves, refer to `qitech_lib`.
