# EtherCAT Basics

## State Machine
![State Machine](./ethercat-state-machine.png)
The EtherCAT state machine is a fundamental concept on setting up an EtherCAT network.

Devices in an EtherCAT network can be in one of the following states:
| State  | Configuration | Process Data | Electrical Output | Description                               |
| ------ | ------------- | ------------ | ----------------- | ----------------------------------------- |
| INIT   | No            | No           | No                | Device can be configured.                 |
| PREOP  | Yes           | No           | No                | Device can be configured.                 |
| SAFEOP | No            | Yes          | No                | Like production but no electrical output. |
| OP     | Yes           | Yes          | Yes               | Production Mode.                          |
| BOOT   | Yes           | No           | No                |                                           |

1. When Ethercrab initializes the network it transfers the devices from INIT to PREOP state.
2. We can then identify machines and send the configurations to the devices accordingly.
3. Once the devices are configured we can move them indirectly to OP state.

Now we can read and write process data from the devices.

## Network Topology
EtherCAT supports a variety of network topologies like:
- Line
- Ring
- Star
- Tree
- ...

In our case we don't need to worry about the topology as the EtherCAT master (Ethercrab) will take care of it.
Most likely we will use a line topology as just daisy chaining the devices together.

## Master / Slave
EtherCAT is a master/slave architecture. The EtherCAT is a device as well as a software. Slaves are also devices and need a slave software. The slave software is the Beckhoff firmware which we can't change. The master can be Beckhoff's TwinCAT, SOEM (C++), PySEOM (Python) or in our case Ethercrab (Rust). The master is responsible for the network topology, device discovery, distributed clock synchronization, the state machine and the communication with the slaves. Ethercrab gives us the tools to send **SDO** requests and access to the **PDO** data for each individual device.

Only the master can initiate communication. The slaves can only modify the data in the frame.

Buscouplers like EK1100 are not EtherCAT masters but also slaves.

## EtherCAT Frame
EtherCAT builds on top of Ethernet. Packets are always broadcasted and traverse the whole network. The frame will eventually return to the master. The devices can read and write to the same frame so often the sent data will return similarly. A "working count" keeps track of the number of devices that have processed the frame.

Each frame can contain multiple EtherCAT datagrams. Each datagram can contain multiple EtherCAT commands. The commands specify the method of addressing and the kind of reading and writing. All this logic is handled by the EtherCAT master (Ethercrab).

## Methods of Communication
- **Process Data** / **PDU**: The process data is a frame that is frequently sent to the devices. It contains an array of binary data. The schema of the data is configured beforehand. The EtherCAT Master (Ethercrab) negotiates with the devices how many bytes they need at what position. The amount of bytes depends on the **PDO Assignment** which can sometimes be configured but has always a default
- **Service Data** / **SDO**
  -  **Mailbox**
     -  **CoE**: The CoE (CANopen over EtherCAT) is a protocol for a key-value store inside the device. The CoE is reset to default when the device is powered on. It's not available on all devices. It can be used to change parameters of the devices like different operating modes, limits, etc. It can't be changed on the fly in OP state. It's also used to set the **PDO Assignment**.
       -  (And more protocols that we don't use)
  
## EEPROM
Meaning: Electrically Erasable Programmable Read-Only Memory.

Although not every device supports CoE all of them have an internal EEPROM which holds the identity of the device like product ID, vendor and serial number (All devices I encountered have a serial of `0`). When the device is powered on the values are transferred into RAM and can be read by reading it with special Commands. The EEPROM is not meant to be read directly but it can be done via the SII interface. TwinCAT also supports a HEX dump of the EEPROM.

The EEPROM values are 16bit not 8bit so it's addressed by "words" not by "bytes" 

**Further Reading:** [ESC Access](https://infosys.beckhoff.com/english.php?content=../content/1033/tc3_io_intro/1358008331.html&id=)

## Modes of Addressing
There are mainly three different addressing modes but they have multiple names:

| Names                                                                  | How to Change                                     | Description                  |
| ---------------------------------------------------------------------- | ------------------------------------------------- | ---------------------------- |
| **Configured Address** / **Explicit Address** / **Configured Address** | Automatically by master on network initialization | Default way of addressing    |
| **Alias Address** / **SecondSlaveAddress** / **SSA**                   | Via SDO, persisted in EEPROM.                     | Alternative to configured    |
| **InputWord** / **IdentificationValue** / **Data Word**                | Changed by hardware.                              | Not available on all devices |

**Further Reading**: [Addressing](https://infosys.beckhoff.com/english.php?content=../content/1033/ethercatsystem/2469080715.html&id=)

## Device Configuration (CoE)
Some device support can be configured before operational. We can configure the process data schema, metedata about connected sensors, operation modes, limits, and more.
[Read how to configure devices with Ethercat HAL](./docs/coe.md)