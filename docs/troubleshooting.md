# Ethercat Tab shows no Ethercat Terminals!

When the Ethercat page does not show any terminals, this can be due to multiple reasons

## 1: Incorrect Connection of EK1100

Check that the computers ethernet cable is properly connected to the EK1100 IN port

Check the cabling if more than one EK1100 is used.
When you have multiple EK the connection should be like this:

Master (Your pc) -> Ethernet cable -> IN EK1100 (A)

OUT EK1100 (A) -> Ethernet cable -> IN EK1100 (B)

If you connected any part of this incorrectly the ethercat software will try to communicate but will fail, because the protocol works only if all devices are connect in series.

## 2: Defective EtherCAT Terminal

If a Terminal is somehow defective it can disrupt the communication flow expected by EtherCAT thus causing no devices to show up, as ethercat requires all terminals to work properly.

To identify the defective terminal start unplugging terminals starting from the End terminal. Working your way backwards. Every time you unplug a terminal restart the backend process.

Youll know that you have removed the defective terminal when the ethercat tab suddenly shows devices.

Replace that terminal with an appropriate one and repeat this if the error persists.

# Motor does not start!

If the Motor does not start after trying to talk to the inverter, the most common reason is that the inverter's serial connection settings (baudrate,encoding ...) are incorrect.

The correct settings can be found in mitsubishi_inverter.md under `All Settings needed for the Inverter Communication/Operation with Extruder`
