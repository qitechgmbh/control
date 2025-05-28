# Identification

As explained in the [EtherCAT Basics](./ethercat-basics.md), there are different ways to address and identify devices in EtherCAT.
However, those are not suitable for use because we build the one software to rule them all.

## Identification Values

Requirements:
- Order Agnostic (We should be able to plug into the network at any point, or machines are in different order, devices are plugged in in different order)
- Multiple same devices in one machine (like 2x EL2002)
- Multiple machines (Like Winder + Extruder)
- Multiple same machines (Like 2x Winder)

We need a method of recognizing what device belongs to what machine:
- Each device needs a **Machine ID**

To avoid machine ID collisions:
- Each machine needs a **Vendor ID**

We need to be able to tell multiple machines of the same type apart:
- Each machine needs a **Serial Number**

We need to know what the purpose of each device in a machine is:
- Each device needs a **Device Role**

## Persistence

How and where do we save the identification values? A machine does not have a central storage medium. Saving the network configuration in the application is bad when changing the EtherCAT master device. Additionally, the serial number in the EEPROM is always `0`. Not all devices have the CoE key value store.

The solution: Persisting the values in empty spaces inside each device's EEPROM. As seen in the [ESC Access](https://infosys.beckhoff.com/english.php?content=../content/1033/tc3_io_intro/1358008331.html&id=) documentation, there are reserved spaces. With a hex dump of the EEPROM, we can confirm that these spaces are likely not used. The EEPROM is very small, so we can fit the values as four `u16` (Big Endian) values. In case we need to customize the position of the values, we customize the addresses for each device.

The default word (not byte) addresses are:
- Vendor: `0x0028`
- Machine: `0x0029`
- Serial: `0x002A`
- Role: `0x002B`

## Writing values
1. For each new device, we need to identify free space in the EEPROM and add them to `get_identification_addresses` in `control-core/src/identification/mod.rs`.
2. We can configure the values for connected devices in the frontend or the REST endpoint `/api/v1/write_machine_device_identification`.
3. Then we need to power restart the network.

In the frontend, we have `machinePreset` which stores what EtherCAT devices can have what roles in what machines, simplifying the UI.

## Identifying machines
When setting up the network, we read the identification values for all devices. We then merge them into groups by the vendor ID, machine ID, and serial number. Devices without these values are *unidentified* devices and can't yet be included in a machine.

We then give the identified device groups to the respective machine constructor. It parses the EtherCAT identity (vendor, product ID, revision) again and ensures all correct devices are present. It rejects duplicate roles and fails when a mandatory device is missing. If the machine validation process fails, it marks the machine with a readable error which can be seen in the frontend. If the machine is valid, it is included as an actor in the control loop.

## Referencing devices internally
The device identification we proposed is of a semantic nature. While identifying machines, we still need to refer to devices in the network. We simply use the `subdevice_index`, which is the index/order the device was initialized by the EtherCAT master. This is likely the same as the physical order.