# Identification

Every machine the backend controls is identified by three numbers: a **vendor**, a **machine** type, and a **serial**. Together they tell the backend *what* a piece of hardware is and *which* individual unit it is.

## Type and instance

Identification is split into two parts:

- **`MachineIdentification { vendor, machine }`** — the machine *type*: which vendor built it and which kind of machine it is. The QiTech vendor id is `0x0001`.
- **Serial** — added on top of the type, it pins down one specific unit.

The full, unique identity is written as `vendor/machine/serial`. An identity counts as valid only when all three values are non-zero.

## The identity lives on the hardware

An identity is not assigned in software — it is read from the device itself. When devices are detected on the EtherCAT bus, the backend reads each device's stored identification (vendor, machine, serial) and a **role**.

The role is what lets several devices form a single machine: a machine's inputs and outputs can be spread across multiple terminals, and each terminal carries the same `vendor/machine/serial` but a different role, so the backend knows how they fit together.

## From an identity to a running machine

Machine types are collected in a **registry**. Each type registers itself against one or more identifications it is able to drive. When hardware is discovered, its identity is looked up in the registry, and the matching machine type is instantiated with that hardware.

If an identity is not registered — or is not valid — no machine is built for it.

## Slugs and the API

Each identity also maps to a short **slug**. The slug together with the serial is how a specific machine is addressed over the REST and Socket.IO APIs — see [Communication & API](Communication-API).
