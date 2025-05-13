# Serial Devices
Serial communication interface through which information transfers in or out sequentially one bit at a time.

Current Serial Devices:
- DRE (Laser Diameter Measuring Instrument) with VID `0x0403` and PID `0x6001`, connected through RTU Modbus (USB).

## Current Serial Detection 
Serial Detection contains 3 main components:
- `Serial Registry`, the interface that is used as buffer for all registered Serial Devices, and responsible for start of registered Serial Devices and returns link to started Serial Device.
- `Connected Serial USB`, the hashmap that saves the path to USB port with the link to started Serial Device under this port path.
- `Ports`, previously detected USB ports with pathes and usb port information (such as Vendor & Product ID, Serial Number)

The cycle of Serial Detection is consisted from following steps:

`1)` Detection of all current Serial Connections

`2)` Extraction of USB Port connections (only this type of connection contains information about vid and pid of modbus itself)

`3)` Comparison of previously detected USB ports with currently detected USB ports (from the step 2)

`4)` Resave previously detected USB ports with current ports

`5)` Start connection of newly detected ports with Serial Registry and save the path to the port with the link to started Serial Device to `Connected Serial USB`

`6)` Remove serial ports from `Connected Serial USB` that was not detected (in the step 2)

`7)` The last step is to check whether the connection with the serial devices is still active. Using the `Connected Devices` parameters or functions, we can verify if the serial device can still fulfill its functionalities. If not, the link to the connected device should be resaved with a disconnection message. 
The next step is to check if the devices that were previously disconnected or unable to establish a connection can now connect to the serial device during this iteration. If they can, save the link to the device in Connected Serial USB.
