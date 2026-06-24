# Other transports

EtherCAT is the system's real-time backbone and carries the cyclic process data — see **Hardware & EtherCAT**. Additionally  QiTech Control supports both Modbus RTU & TCP via QiTech Lib.

A device on a serial line (for example a USB serial port) is attached to a machine as part of its hardware; the backend builds the machine's hardware from the serial port (`generate_machine_hardware_from_serial`). A serial line also carries Modbus RTU (below).
