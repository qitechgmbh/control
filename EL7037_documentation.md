Configuration by means of the TwinCAT System Manager 

## **7 Configuration by means of the TwinCAT System Manager** 

## **7.1 EL7037 - Object description and parameterization** 

## **EtherCAT XML Device Description** 

The display matches that of the CoE objects from the EtherCAT XML Device Description. We recommend downloading the latest XML file from the download area of the Beckhoff website and installing it according to installation instructions. 

## **Parameterization via the CoE list (CAN over EtherCAT)** 

The terminal is parameterized via the CoE - Online tab (double-click on the respective object) or via the Process Data tab (allocation of PDOs). Please note the following general CoE information [} 37] when using/manipulating the CoE parameters: 

- Keep a startup list if components have to be replaced 

- Differentiation between online/offline dictionary, existence of current XML description 

- use "CoE reload" for resetting changes 

## _**NOTICE**_ 

## **Risk of damage to the device!** 

We strongly advise not to change settings in the CoE objects while the axis is active, since this could impair the control. 

## **Introduction** 

The CoE overview contains objects for different intended applications: 

## **Object overview** 

- Restore object [} 195] 

- Configuration data [} 196] 

- Command object [} 200] 

- Input data [} 201] 

- Output data [} 202] 

- Information / diagnostic data (channel specific) [} 207] 

- Manufacturer configuration data (device-specific) [} 208] 

- Information / diagnostic data (device-specific) [} 209] 

- Standard objects [} 209] 

## **7.1.1 Restore object** 

**Index 1011 Restore default parameters** 

**Index Name Meaning Data type Flags Default (hex)** 1011:0 Restore default Restore default parameters UINT8 RO 0x01 (1dec) parameters 1011:01 SubIndex 001 If this object is set to **"0x64616F6C"** in the set value UINT32 RW 0x00000000 (0dec) ~~as~~ dialog, all backup objects are reset to their delivery state. 

EL70x7 

Version: 2.2.0 

195 

Configuration by means of the TwinCAT System Manager 

## **7.1.2 Configuration data** 

## **Index 8000 ENC Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8000:0|ENC Settings<br>Ch.1|Maximum subindex|UINT8|RO|0x0E (14dec)|
|8000:08|Disable filter|Deactivates the input filters.|BOOLEAN|RW|0x00(0dec)|
|8000:0A|Enable micro<br>increments|The lower 8 bits of the counter value are extrapolated.|BOOLEAN|RW|0x00 (0dec)|
|8000:0E|Reversion of<br>rotation|Activates reversion of rotation of the encoder.|BOOLEAN|RW|0x00 (0dec)|



## **Index 8010 STM Motor Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8010:0|STM Motor<br>Settings Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|8010:01|Maximal current|Maximum permanent motor coil current<br>**Unit**: 1 mA|UINT16|RW|0x1388 (5000dec)|
|8010:02|Reduced current|Reduced coil current<br>**Unit**: 1 mA|UINT16|RW|0x09C4 (2500dec)|
|8010:03|Nominal voltage|Nominal voltage (supply voltage) of the motor<br>**Unit**: 10 mV|UINT16|RW|0x1388 (5000dec)|
|8010:04|Motor coil<br>resistance|Internal resistance of the motor<br>**Unit**: 10 mOhm|UINT16|RW|0x0064 (100dec)|
|8010:05|Motor EMF|Countervoltage of the motor<br>**Unit**: 1 mV /(rad/s)|UINT16|RW|0x0000 (0dec)|
|8010:06|Motor fullsteps|Number of full motor steps|UINT16|RW|0x00C8(200dec)|
|8010:07|Encoder<br>increments (4-<br>fold)|Number of encoder increments per revolution with<br>quadruple evaluation|UINT16|RW|0x1000 (4096dec)|
|8010:09|Start velocity|Minimum starting velocity of the motor<br>**Unit**:10000 corresponds to 100% [<br>}<br>167<br>]|UINT16|RW|0x0000 (0dec)|
|8010:0A|Motor coil<br>inductance|Inductance of the motor<br>**Unit**: 0.01 mH|UINT16|RW|0x0000 (0dec)|
|8010:10|Drive on delay<br>time|Delay between activation of driver stage and<br>„ready= 1“|UINT16|RW|0x0064 (100dec)|
|8010:11|Drive off delay<br>time|Delay between deactivation of driver stage and<br>„ready= 0“|UINT16|RW|0x0096 (150dec)|



## **Index 8011 STM Controller Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8011:0|STM Controller<br>Settings Ch.1|Maximum subindex|UINT8|RO|0x02 (2dec)|
|8011:01|Kpfactor(curr.)|Kpcontrol factor of the current controller|UINT16|RW|0x0096(150dec)|
|8011:02|Ki factor(curr.)|Ki control factor of the current controller|UINT16|RW|0x000A(10dec)|



196 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 8012 STM Features Ch.1 (part 1)** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8012:0|STM Features<br>Ch.1|Maximum subindex|UINT8|RO|0x3A (58dec)|
|8012:01|Operation mode|permitted values:|BIT4|RW|0x00 (0dec)|
|||0: Automatic||||
|||1: Velocitydirect||||
|||3: Position controller||||
|||4: Ext. Velocitymode||||
|||5: Ext. Position mode||||
|||6: Velocitysensorless||||
|8012:05|Speed range|permitted values:|BIT3|RW|0x01 (1dec)|
|||0: 1000 Fullsteps/sec||||
|||1: 2000 Fullsteps/sec||||
|||2: 4000 Fullsteps/sec||||
|||3: 8000 Fullsteps/sec||||
|||4: 16000 Fullsteps/sec||||
|||5: 32000 Fullsteps/sec||||
|8012:08|Feedback type|permitted values:|BIT1|RW|0x01 (1dec)|
|||0: Encoder||||
|||1: Internal counter||||
|8012:09|Invert motor<br>polarity|Invert the direction of rotation of the motor|BOOLEAN|RW|0x00 (0dec)|
|8012:0A|Error on steplost|Error on loss of step|BOOLEAN|RW|0x00(0dec)|
|8012:0B|Fan cartridge<br>present|Fan cartridge present|BOOLEAN|RW|0x00 (0dec)|
|8012:11|Select info data 1|permitted values:|UINT8|RW|0x0B (11dec)|
|||0: Status word||||
|||7: Motor velocity||||
|||11: Motor load||||
|||13: Motor dc current||||
|||101: Internal temperature||||
|||103: Control voltage||||
|||104: Motor supplyvoltage||||
|||150: Drive - Status word||||
|||151: Drive – State||||
|||152: Drive - Position lag (low word)||||
|||153: Drive - Position lag (high word)||||



EL70x7 

Version: 2.2.0 

197 

Configuration by means of the TwinCAT System Manager 

## **Index 8012 STM Features Ch.1 (part 2)** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8012:19|Select info data 2|permitted values:|UINT8|RW|0x0D (13dec)|
|||0: Status word||||
|||7: Motor velocity||||
|||11: Motor load||||
|||13: Motor dc current||||
|||101: Internal temperature||||
|||103: Control voltage||||
|||104: Motor supplyvoltage||||
|||150: Drive - Status word||||
|||151: Drive - State||||
|||152: Drive - Position lag (low word)||||
|||153: Drive - Position lag (high word)||||
|8012:30|Invert digital input<br>1|Invert digital input|BOOLEAN|RW|0x00 (0dec)|
|8012:31|Invert digital input<br>2|Invert digital input|BOOLEAN|RW|0x00 (0dec)|
|8012:32|Function for input<br>1|permitted values:|BIT4|RW|0x00 (0dec)|
|||0: Normal input||||
|||1: Hardware enable||||
|||2: PLC cam||||
|8012:36|Function for input<br>2|permitted values:|BIT4|RW|0x00 (0dec)|
|||0: Normal input||||
|||1: Hardware enable||||
|||2: PLC cam||||
|8012:3A|Function for<br>output 1|permitted values:|BIT4|RW|0x0F (15dec)|
|||0: Normal output||||
|||1: Break(linked with driver enable)||||
|||15: Disabled||||



**Index 8014 STM Controller Settings 3 Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8014:0|STM Controller<br>Settings 3 Ch.1|Maximum subindex|UINT8|RO|0x09 (9dec)|
|8014:01|Feed forward<br>(pos.)|Pilot control of the position controller|UINT32|RW|0x000186A0<br>(100000dec)|
|8014:02|Kpfactor(pos.)|Kpcontrol factor of theposition controller|UINT16|RW|0x01F4(500dec)|
|8014:03|Kp factor (velo.)|Kp control factor of the velocity controller<br>**Unit**: 0.1 mA /(rad/s)|UINT32|RW|0x00000032<br>(50dec)|
|8014:04|Tn (velo.)|Time constant Tn of the velocity controller<br>**Unit**: 0.01 ms|UINT16|RW|0xC350 (50000dec)|
|8014:05|Sensorless param<br>1|First parameter (sensorless control)|UINT16|RW|0x0000 (0dec)|
|8014:06|Sensorless param<br>2|Second parameter (sensorless control)|UINT16|RW|0x0000 (0dec)|
|8014:07|Cross over<br>velocity1|First velocity transition (sensorless control)<br>**Unit**: 0.1 rad/s|UINT16|RW|0x0000 (0dec)|
|8014:08|Cross over<br>velocity2|Second velocity transition (sensorless control)<br>**Unit**: 0.1 rad/s|UINT16|RW|0x0000 (0dec)|
|8014:09|Cross over<br>velocity3|Third velocity transition (sensorless control)<br>**Unit**: 0.1 rad/s|UINT16|RW|0x0000 (0dec)|



198 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 8020 POS Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8020:0|POS Settings<br>Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|8020:01|Velocity min.|Minimum set velocity<br>(range: 0-10000)|INT16|RW|0x0064 (100dec)|
|8020:02|Velocity max.|Maximum set velocity<br>(range: 0-10000)|INT16|RW|0x2710 (10000dec)|
|8020:03|Acceleration pos.|Acceleration in positive direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:04|Acceleration neg.|Acceleration in negative direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:05|Deceleration pos.|Deceleration in positive direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:06|Deceleration neg.|Deceleration in negative direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:07|Emergency<br>deceleration|Emergency deceleration (both directions of rotation)<br>**Unit**: 1 ms|UINT16|RW|0x0064 (100dec)|
|8020:08|Calibration<br>position|Calibration position|UINT32|RW|0x00000000 (0dec)|
|8020:09|Calibration<br>velocity (towards<br>plc cam)|Calibration velocity towards the cam<br>(range: 0-10000)|INT16|RW|0x0064 (100dec)|
|8020:0A|Calibration<br>Velocity (off plc<br>cam)|Calibration velocity away from the cam<br>(range: 0-10000)|INT16|RW|0x000A (10dec)|
|8020:0B|Target window|Target window|UINT16|RW|0x000A(10dec)|
|8020:0C|In-Target timeout|Target position timeout<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:0D|Dead time<br>compensation|Dead time compensation<br>**Unit**: 1µs|INT16|RW|0x0032 (50dec)|
|8020:0E|Modulo factor|Modulo factor/position|UINT32|RW|0x00000000(0dec)|
|8020:0F|Modulo tolerance<br>window|Tolerance window for modulo positioning|UINT32|RW|0x00000000 (0dec)|
|8020:10|Position lagmax.|Maximum allowable steperror|UINT16|RW|0x0000(0dec)|
|8020:11|Calibration<br>acceleration<br>(aroundplc cam)|Acceleration and braking ramps for homing runs|UINT16|RW|0x0000 (0dec)|



EL70x7 

Version: 2.2.0 

199 

Configuration by means of the TwinCAT System Manager 

## **Index 8021 POS Features Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8021:0|POS Features<br>Ch.1|Maximum subindex|UINT8|RO|0x16 (22dec)|
|8021:01|Start type|permitted values:|UINT16|RW|0x0001 (1dec)|
|||0: Idle||||
|||1: Absolute||||
|||2: Relative||||
|||3: Endlessplus||||
|||4: Endless minus||||
|||6: Additive||||
|||24832: Calibration(Hardware sync)||||
|||24576: Calibration(Plc cam)||||
|||28416: Calibration(Clear manual)||||
|||28160: Calibration(Set manual)||||
|||28161: Calibration(Set manual auto)||||
|||1029: Modulo current||||
|||773: Modulo minus||||
|||517: Moduloplus||||
|||261: Modulo short||||
|8021:11|Time information|permitted values:|BIT2|RW|0x00 (0dec)|
|||0: Elapsed time<br>current drive time since start of the travel command||||
|8021:13|Invert calibration<br>cam search<br>direction|Inversion of the direction of rotation towards the cam|BOOLEAN|RW|0x01 (1dec)|
|8021:14|Invert sync<br>impulse search<br>direction|Inversion of the direction of rotation away from the cam|BOOLEAN|RW|0x00 (0dec)|
|8021:15|Emergency stop<br>on position lag<br>error|Triggers an emergency stop if the maximum following<br>error is exceeded|BOOLEAN|RW|0x00 (0dec)|
|8021:16|Enhanced diag<br>history|Provides detailed messages about the status of the<br>positioninginterface in the diaghistory|BOOLEAN|RW|0x00 (0dec)|



## **7.1.3 Command object** 

## **Index FB00 STM Command** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|FB00:0|STM Command|Maximum subindex|UINT8|RO|0x03(3dec)|
|FB00:01|Request|Requesting a command<br>0x8000: Software reset|OCTET-<br>STRING[2]|RW|{0}|
|FB00:02|Status|Status of the command|UINT8|RO|0x00 (0dec)|
|||0: No error, without return value||||
|||1: No error, with return value||||
|||2: With error, without return value||||
|||3: With error, with return value||||
|||... reserved||||
|||255: Command execution active||||
|FB00:03|Response|Return value of the executed command|OCTET-<br>STRING[4]|RO|{0}|



200 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **7.1.4 Input data** 

## **Index 6000 ENC Inputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|6000:0|ENC Inputs Ch.1|Maximum subindex|UINT8|RO|0x16(22dec)|
|6000:01|Latch C valid|The counter value was latched with the C track.|BOOLEAN|RO|0x00(0dec)|
|6000:02|Latch extern valid|The counter value was stored via the external latch.|BOOLEAN|RO|0x00(0dec)|
|6000:03|Set counter done|The counter was set.|BOOLEAN|RO|0x00(0dec)|
|6000:04|Counter underflow|Counter underflow|BOOLEAN|RO|0x00(0dec)|
|6000:05|Counter overflow|Counter overflow|BOOLEAN|RO|0x00(0dec)|
|6000:08|Extrapolation stall|The extrapolatedpart of the counter is invalid.|BOOLEAN|RO|0x00(0dec)|
|6000:09|Status of input A|Status of the A-input|BOOLEAN|RO|0x00(0dec)|
|6000:0A|Status of input B|Status of the B-input|BOOLEAN|RO|0x00(0dec)|
|6000:0B|Status of input C|Status of the C-input|BOOLEAN|RO|0x00(0dec)|
|6000:0D|Status of extern<br>latch|Status of the ext. latch input|BOOLEAN|RO|0x00 (0dec)|
|6000:0E|Sync error|The Sync error bit is only required for DC mode. It<br>indicates whether a synchronization error has occurred<br>duringtheprevious cycle.|BOOLEAN|RO|0x00 (0dec)|
|6000:10|TxPDO Toggle|The TxPDO toggle is toggled by the slave when the data<br>of the associated TxPDO is updated.|BOOLEAN|RO|0x00 (0dec)|
|6000:11|Counter value|The counter value|UINT32|RO|0x00000000(0dec)|
|6000:12|Latch value|The latch value|UINT32|RO|0x00000000(0dec)|
|6000:16|Timestamp|Time stampof the last counter change|UINT32|RO|0x00000000(0dec)|



## **Index 6010 STM Inputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**||**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|---|
|6010:0|STM Inputs Ch.1|Maximum subindex|||UINT8|RO|0x15(21dec)|
|6010:01|Readyto enable|Driver stage is readyfor enabling.|||BOOLEAN|RO|0x00(0dec)|
|6010:02|Ready|Driver stage is readyfor operation.|||BOOLEAN|RO|0x00(0dec)|
|6010:03|Warning|A warninghas occurred(see index||0xA010 [<br>}<br>208<br>]<br>).|BOOLEAN|RO|0x00 (0dec)|
|6010:04|Error|An error has occurred(see index|0xA010 [<br>}<br>208<br>]<br>).||BOOLEAN|RO|0x00 (0dec)|
|6010:05|Moving positive|Driver stage is activated inpositive direction.|||BOOLEAN|RO|0x00(0dec)|
|6010:06|Movingnegative|Driver stage is activated in negative direction.|||BOOLEAN|RO|0x00(0dec)|
|6010:07|Torque reduced|Reduced torque is active.|||BOOLEAN|RO|0x00(0dec)|
|6010:08|Motor stall|A loss of stephas occurred.|||BOOLEAN|RO|0x00(0dec)|
|6010:0C|Digital input 1|Digital input 1|||BOOLEAN|RO|0x00(0dec)|
|6010:0D|Digital input 2|Digital input 2|||BOOLEAN|RO|0x00(0dec)|
|6010:0E|Sync error|The Sync error bit is only required for DC mode. It<br>indicates whether a synchronization error has occurred<br>duringtheprevious cycle.|||BOOLEAN|RO|0x00 (0dec)|
|6010:10|TxPDO Toggle|The TxPDO toggle is toggled by the slave when the data<br>of the associated TxPDO is updated.|||BOOLEAN|RO|0x00 (0dec)|
|6010:11|Info data 1|Synchronous information (selection via subindex<br>0x8012:11 [<br>}<br>197<br>]<br>)|||UINT16|RO|0x0000 (0dec)|
|6010:12|Info data 2|Synchronous information (selection via subindex<br>0x8012:19 [<br>}<br>197<br>]<br>)|||UINT16|RO|0x0000 (0dec)|
|6010:13|Motor load|Current motor load<br>**Unit:**0.01°|||INT16|RO|0x0000 (0dec)|
|6010:14|Internalposition|Internal microstep position|||UINT32|RO|0x00000000(0dec)|
|6010:15|Externalposition|Encoderposition|||UINT32|RO|0x00000000(0dec)|



EL70x7 

Version: 2.2.0 

201 

Configuration by means of the TwinCAT System Manager 

## **Index 6020 POS Inputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|6020:0|POS Inputs Ch.1|Maximum subindex|UINT8|RO|0x23(35dec)|
|6020:01|Busy|A current travel command is active.|BOOLEAN|RO|0x00(0dec)|
|6020:02|In-Target|Motor has arrived at target.|BOOLEAN|RO|0x00(0dec)|
|6020:03|Warning|A warninghas occurred.|BOOLEAN|RO|0x00(0dec)|
|6020:04|Error|An error has occurred.|BOOLEAN|RO|0x00(0dec)|
|6020:05|Calibrated|Motor is calibrated.|BOOLEAN|RO|0x00(0dec)|
|6020:06|Accelerate|Motor is in the accelerationphase.|BOOLEAN|RO|0x00(0dec)|
|6020:07|Decelerate|Motor is in the decelerationphase.|BOOLEAN|RO|0x00(0dec)|
|6020:11|Actualposition|Current targetposition of the travel commandgenerator|UINT32|RO|0x00000000(0dec)|
|6020:21|Actual velocity|Current set velocityof the travel commandgenerator|INT16|RO|0x0000(0dec)|
|6020:22|Actual drive time|Travel command time information (see subindex<br>0x8021:11 [<br>}<br>200<br>]<br>)|UINT32|RO|0x00000000 (0dec)|
|6020:23|Actualposition lag|Lagofposition|UINT32|RO|0x00000000(0dec)|



## **7.1.5 Output data** 

## **Index 7000 ENC Outputs (compact) Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|7000:0|ENC Outputs<br>Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|7000:01|Enable latch C|Activate latchingvia the C-track.|BOOLEAN|RO|0x00(0dec)|
|7000:02|Enable latch<br>extern on positive<br>edge|Activate external latch with positive edge.|BOOLEAN|RO|0x00 (0dec)|
|7000:03|Set counter|Set the counter value.|BOOLEAN|RO|0x00(0dec)|
|7000:04|Enable latch<br>extern on<br>negative edge|Activate external latch with negative edge.|BOOLEAN|RO|0x00 (0dec)|
|7000:11|Set counter value|This is the counter value to be set via "Set counter".|UINT16|RO|0x0000(0dec)|



## **Index 7000 ENC Outputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|7000:0|ENC Outputs<br>Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|7000:01|Enable latch C|Activate latchingvia the C-track.|BOOLEAN|RO|0x00(0dec)|
|7000:02|Enable latch<br>extern on positive<br>edge|Activate external latch with positive edge.|BOOLEAN|RO|0x00 (0dec)|
|7000:03|Set counter|Set the counter value.|BOOLEAN|RO|0x00(0dec)|
|7000:04|Enable latch<br>extern on<br>negative edge|Activate external latch with negative edge.|BOOLEAN|RO|0x00 (0dec)|
|7000:11|Set counter value|This is the counter value to be set via "Set counter".|UINT32|RO|0x00000000(0dec)|



202 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 7010 STM Outputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|7010:0|STM Outputs<br>Ch.1|Maximum subindex|UINT8|RO|0x21 (33dec)|
|7010:01|Enable|activates the output stage|BOOLEAN|RO|0x00(0dec)|
|7010:02|Reset|All errors that may have occurred are reset by setting this<br>bit(risingedge).|BOOLEAN|RO|0x00 (0dec)|
|7010:03|Reduce torque|Reduced torque (coil current) is active (see subindex<br>0x8010:02 [<br>}<br>196<br>]<br>).|BOOLEAN|RO|0x00 (0dec)|
|7010:0C|Digital output 1|Digital output 1|BOOLEAN|RO|0x00(0dec)|
|7010:11|Position|Set position specification<br>**Unit:**Increments [<br>}<br>170<br>]|UINT32|RO|0x00000000 (0dec)|
|7010:21|Velocity|Set velocity specification<br>**Unit:**+/-32767 corresponds to +/-100% [<br>}<br>167<br>]|INT16|RO|0x0000 (0dec)|



EL70x7 

Version: 2.2.0 

203 

Configuration by means of the TwinCAT System Manager 

## **Index 7020 POS Outputs Ch.1** 

204 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|7020:0|POS Outputs<br>Ch.1|Maximum subindex||UINT8|RO|0x24 (36dec)|
|7020:01|Execute|Start travel command (rising edge), or prematurely abort<br>travel command(fallingedge)||BOOLEAN|RO|0x00 (0dec)|
|7020:02|Emergency Stop|Prematurely abort travel command with an emergency<br>ramp (risingedge)||BOOLEAN|RO|0x00 (0dec)|
|7020:11|Targetposition|Specification of the targetposition||UINT32|RO|0x00000000(0dec)|
|7020:21|Velocity|Specification of the maximum set velocity||INT16|RO|0x0000(0dec)|
|7020:22|Start type|0x0000<br>Idle|No travel command is being<br>executed|UINT16|RO|0x0000 (0dec)|
|||0x0001<br>Absolute|Absolute target position||||
|||0x1001<br>Absolute(Change)|Change during an active travel<br>command||||
|||0x0002<br>Relative|Target position relative to the<br>currentposition||||
|||0x1002<br>Relative(Change)|Change during an active travel<br>command||||
|||0x0003<br>Endlessplus|Endless driving in positive<br>direction of rotation||||
|||0x0004<br>Endless minus|Endless driving in negative<br>direction of rotation||||
|||0x0105<br>Modulo short|Shortest distance to the next<br>moduloposition||||
|||0x0115<br>Modulo short extended|Shortest distance to the next<br>modulo position (without<br>modulo window)||||
|||0x0205<br>Modulo plus|Drive in positive direction of<br>rotation to the next modulo<br>position||||
|||0x0215<br>Modulo plus extended|Drive in positive direction of<br>rotation to the next modulo<br>position (without modulo<br>window)||||
|||0x0305<br>Modulo minus|Drive in negative direction of<br>rotation to the next modulo<br>position||||
|||0x0315<br>Modulo minus extended|Drive in negative direction of<br>rotation to the next modulo<br>position (without modulo<br>window)||||
|||0x0405<br>Modulo current|Drive in the last implemented<br>direction of rotation to the next<br>moduloposition||||
|||0x0415<br>Modulo current<br>extended|Drive in the last implemented<br>direction of rotation to the next<br>modulo position (without<br>modulo window)||||
|||0x0006<br>Additive|New target position relative/<br>additive to the last target<br>position||||
|||0x1006<br>Additive(Change)|Change during an active travel<br>command||||
|||0x6000<br>Calibration, PLC cam|Calibration with cam||||
|||0x6100<br>Calibration, HW sync|Calibration with cam and C-<br>track||||
|||0x6E00<br>Calibration, set manual|Set calibration manually||||
|||0x6E01<br>Calibration, set manual<br>auto|Set automatic calibration, for<br>"Enable = 1"||||
|||0x6F00<br>Calibration, clear<br>manual|Clear calibration manually||||
|7020:23|Acceleration|Acceleration specification||UINT16|RO|0x0000(0dec)|
|7020:24|Deceleration|Deceleration specification||UINT16|RO|0x0000(0dec)|



EL70x7 

Version: 2.2.0 

205 

Configuration by means of the TwinCAT System Manager 

## **Index 7021 POS Outputs 2 Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|7021:0|POS Outputs<br>Ch.1|Maximum subindex||UINT8|RO|0x24 (36dec)|
|7021:03|Enable auto start|Enable auto start||BOOLEAN|RO|0x00(0dec)|
|7021:11|Targetposition|Specification of the targetposition||UINT32|RO|0x00000000(0dec)|
|7021:21|Velocity|Specification of the maximum set velocity||INT16|RO|0x0000(0dec)|
|7021:22|Start type|0x0000<br>Idle|No travel command is being<br>executed|UINT16|RO|0x0000 (0dec)|
|||0x0001<br>Absolute|Absolute target position||||
|||0x1001<br>Absolute(Change)|Change during an active travel<br>command||||
|||0x0002<br>Relative|Target position relative to the<br>currentposition||||
|||0x1002<br>Relative(Change)|Change during an active travel<br>command||||
|||0x0003<br>Endlessplus|Endless driving in positive<br>direction of rotation||||
|||0x0004<br>Endless minus|Endless driving in negative<br>direction of rotation||||
|||0x0105<br>Modulo short|Shortest distance to the next<br>moduloposition||||
|||0x0115<br>Modulo short<br>extended|Shortest distance to the next<br>modulo position (without modulo<br>window)||||
|||0x0205<br>Modulo plus|Drive in positive direction of<br>rotation to the next modulo<br>position||||
|||0x0215<br>Modulo plus<br>extended|Drive in positive direction of<br>rotation to the next modulo<br>position(without modulo window)||||
|||0x0305<br>Modulo minus|Drive in negative direction of<br>rotation to the next modulo<br>position||||
|||0x0315<br>Modulo minus<br>extended|Drive in negative direction of<br>rotation to the next modulo<br>position(without modulo window)||||
|||0x0405<br>Modulo current|Drive in the last implemented<br>direction of rotation to the next<br>moduloposition||||
|||0x0415<br>Modulo current<br>extended|Drive in the last implemented<br>direction of rotation to the next<br>modulo position (without modulo<br>window)||||
|||0x0006<br>Additive|New target position relative/<br>additive to the last targetposition||||
|||0x1006<br>Additive(Change)|Change during an active travel<br>command||||
|||0x6000<br>Calibration, PLC cam|Calibration with cam||||
|||0x6100<br>Calibration, HW sync|Calibration with cam and C-track||||
|||0x6E00<br>Calibration, set<br>manual|Set calibration manually||||
|||0x6E01<br>Calibration, set<br>manual auto|Set automatic calibration, for<br>"Enable = 1"||||
|||0x6F00<br>Calibration, clear<br>manual|Clear calibration manually||||
|7021:23|Acceleration|Acceleration specification||UINT16|RO|0x0000(0dec)|
|7021:24|Deceleration|Deceleration specification||UINT16|RO|0x0000(0dec)|



206 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **7.1.6 Information / diagnostic data (channel specific)** 

## **Index 9010 STM Info data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**||**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|9010:0|STM Info data<br>Ch.1|Maximum subindex||UINT8|RO|0x13 (19dec)|
|9010:01|Status word|Status word(see index|0xA010 [<br>}<br>208<br>]<br>)|UINT16|RO|0x0000 (0dec)|
|9010:08|Motor velocity|Current motor velocity||INT16|RO|0x0000(0dec)|
|9010:09|Internalposition|Internalposition(micro|increments)|UINT32|RO|0x00000000(0dec)|
|9010:0B|Motor load|Current motor load<br>**Unit**: 0.01°||INT16|RO|0x0000 (0dec)|
|9010:0D|Motor dc current|Current motor current (DC vector)<br>**Unit**: 1 mA||INT16|RO|0x0000 (0dec)|
|9010:0E|Tn (curr.)|Internally calculated time constant of the current<br>controller<br>**Unit**: 0.01 ms||UINT16|RO|0x0000 (0dec)|
|9010:13|Externalposition|Externalposition(connected encoder)||UINT32|RO|0x00000000(0dec)|



## **Index 9020 POS Info data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|9020:0|POS Info data<br>Ch.1|Maximum subindex|UINT8|RO|0x04 (4dec)|
|9020:01|Status word|Status word|UINT16|RO|0x0000(0dec)|
|9020:03|State (drive<br>controller)|permitted values:|UINT16|RO|0x0000 (0dec)|
|||0: Init||||
|||1: Idle||||
|||272: Go cam||||
|||273: On cam||||
|||16: Start||||
|||17: Acceleration||||
|||18: Constant||||
|||19: Deceleration||||
|||288: Go sync impulse||||
|||289: Leave cam||||
|||4096: Pre target||||
|||4097: In target||||
|||32: EmergencyStop||||
|||33: Normal stop||||
|||304: Calibration stop||||
|||8192: Drive end||||
|||8193: Wait for init||||
|||320: Is calibrated||||
|||321: Not calibrated||||
|||16384: Drive warning||||
|||32768: Error||||
|||65535: Undefined||||
|||256: Calibration start||||
|9020:04|Actualposition lag|Current steperror|INT32|RO|0x00000000(0dec)|



EL70x7 

Version: 2.2.0 

207 

Configuration by means of the TwinCAT System Manager 

**Index A010 STM Diag data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|A010:0|STM Diag data<br>Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|A010:01|Saturated|Driver stage operates with maximum dutycycle|BOOLEAN|RO|0x00(0dec)|
|A010:02|Over temperature|Internal terminal temperature isgreater than 80 °C|BOOLEAN|RO|0x00(0dec)|
|A010:03|Torque overload|Dutycycle output at 100 %|BOOLEAN|RO|0x00(0dec)|
|A010:04|Under voltage|Supplyvoltage less than 7 V|BOOLEAN|RO|0x00(0dec)|
|A010:05|Over voltage|Supply voltage 10 % higher than the nominal voltage<br>(see 0x8010:03 [<br>}<br>196<br>]<br>)|BOOLEAN|RO|0x00 (0dec)|
|A010:06|Short circuit|Short circuit of motor coil|BOOLEAN|RO|0x00(0dec)|
|A010:08|No controlpower|Nopower supplyto driver stage|BOOLEAN|RO|0x00(0dec)|
|A010:09|Misc error|•<br>Initialization failed or<br>•<br>Internal terminal temperature is higher than 100 °C<br>(see 0xF80F:05 [<br>}<br>208<br>]<br>)|BOOLEAN|RO|0x00 (0dec)|
|A010:0A|Configuration|CoE change has not yet been adopted into the current<br>configuration|BOOLEAN|RO|0x00 (0dec)|
|A010:0B|Motor stall|A loss of stephas occurred|BOOLEAN|RO|0x00(0dec)|
|A010:11|Actual operation<br>mode|permitted values:|BIT4|RO|0x00 (0dec)|
|||0: Automatic||||
|||1: Velocitydirect||||
|||2: Velocitycontroller||||
|||3: Position controller||||
|||4: Ext. Velocitymode||||
|||5: Ext. Position mode||||
|||6: Velocitysensorless||||



**Index A020 POS Diag data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|A020:0|POS Diag data<br>Ch.1|Maximum subindex|UINT8|RO|0x06 (6dec)|
|A020:01|Command<br>rejected|Travel command was rejected|BOOLEAN|RO|0x00 (0dec)|
|A020:02|Command<br>aborted|Travel command was aborted|BOOLEAN|RO|0x00 (0dec)|
|A020:03|Target overrun|Targetposition was overrun in the opposite direction|BOOLEAN|RO|0x00(0dec)|
|A020:04|Target timeout|The target window was not reached within the in-target<br>timeout|BOOLEAN|RO|0x00 (0dec)|
|A020:05|Position lag|The maximum followingerror was exceeded|BOOLEAN|RO|0x00(0dec)|
|A020:06|EmergencyStop|An emergencystopwas triggered(automatic or manual)|BOOLEAN|RO|0x00(0dec)|



## **7.1.7 Vendor configuration data (device specific)** 

## **Index F80F STM Vendor data** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|F80F:0|STM Vendor data|Maximum subindex|UINT8|RO|0x05(5dec)|
|F80F:04|Warning<br>temperature|Temperature warning threshold<br>**Unit**: 1 °C|INT8|RW|0x50 (80dec)|
|F80F:05|Switch off<br>temperature|Switch-off temperature<br>**Unit**: 1 °C|INT8|RW|0x64 (100dec)|



208 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

**Index F010 Module list** 

## **7.1.8 Information / diagnostic data (device specific)** 

|**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F010:0<br>Module list<br>Maximum subindex<br>UINT8<br>RW<br>0x03(3dec)<br>F010:01<br>SubIndex 001<br>Encoder profile number<br>UINT32<br>RW<br>0x000001FF<br>(511dec)<br>F010:02<br>SubIndex 002<br>Stepper motor profile number<br>UINT32<br>RW<br>0x000002BF<br>(703dec)<br>F010:03<br>SubIndex 003<br>Positioning interface profile number<br>UINT32<br>RW<br>0x000002C0<br>(704dec)<br>~~——_———~~|**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F010:0<br>Module list<br>Maximum subindex<br>UINT8<br>RW<br>0x03(3dec)<br>F010:01<br>SubIndex 001<br>Encoder profile number<br>UINT32<br>RW<br>0x000001FF<br>(511dec)<br>F010:02<br>SubIndex 002<br>Stepper motor profile number<br>UINT32<br>RW<br>0x000002BF<br>(703dec)<br>F010:03<br>SubIndex 003<br>Positioning interface profile number<br>UINT32<br>RW<br>0x000002C0<br>(704dec)<br>~~——_———~~|
|---|---|
|**Index F081 Download revision**||
|**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F081:0<br>Download revision<br>Maximum subindex<br>UINT8<br>RO<br>0x01(1dec)<br>F081:01<br>Revision number<br>Revision number<br>UINT32<br>RW<br>0x00000000(0dec)<br>~~or~~||
|**Index F900 STM Info data**<br>**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F900:0<br>STM Info data<br>Maximum subindex<br>UINT8<br>RO<br>0x06(6dec)<br>F900:01<br>Software version<br>(driver)<br>Software version of the output driver<br>STRING<br>RO<br>F900:02<br>Internal<br>temperature<br>Internal terminal temperature<br>**Unit**: 1 °C<br>INT8<br>RO<br>0x00 (0dec)<br>F900:04<br>Control voltage<br>Control voltage<br>**Unit**: 1 mV, 10 mV with field-oriented control<br>UINT16<br>RO<br>0x0000 (0dec)<br>F900:05<br>Motor supply<br>voltage<br>Motor supply voltage<br>**Unit**: 1 mV, 10 mV with field-oriented control<br>UINT16<br>RO<br>0x0000 (0dec)<br>F900:06<br>Cycle time<br>Current EtherCAT cycle time<br>**Unit**: 1µs<br>UINT16<br>RO<br>0x0000 (0dec)<br>~~===~~||
|**Index FB40 Memory interface**<br>**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>FB40:0<br>Memoryinterface<br>Maximum subindex<br>UINT8<br>RO<br>0x03(3dec)<br>FB40:01<br>Address<br>reserved<br>UINT32<br>RW<br>0x00000000(0dec)<br>FB40:02<br>Length<br>reserved<br>UINT16<br>RW<br>0x0000(0dec)<br>FB40:03<br>Data<br>reserved<br>OCTET-<br>STRING[8]<br>RW<br>{0}<br>**7.1.9**<br>**Standard objects**<br>**EtherCAT XML Device Description**<br>~~ee~~||
|The display matches that of the CoE objects from the EtherCATXML<br>Device Description. We||
|recommend downloading the latest XML file from the download area of the Beckhoff website and||
|installing it according to installation instructions.||



## **Standard objects (0x1000-0x1FFF)** 

**Index 1000 Device type Index Name Meaning Data type Flags Default (hex)** 1000:0 Device type Device type of the EtherCAT slave: the Lo-Word contains UINT32 RO 0x00001389 the CoE profile used (5001). The Hi-Word contains the (5001dec) ~~ee~~ module profile according to the modular device profile. 

EL70x7 

Version: 2.2.0 

209 

Configuration by means of the TwinCAT System Manager 

## **Index 1008 Device name** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1008:0|Device name|Device name of the EtherCAT slave|STRING|RO|EL7037|
|**Index 1009 Hardware version**||||||
|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|1009:0|Hardware version|Hardware version of the EtherCAT slave|STRING|RO||



## **Index 100A Software version** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|100A:0|Software version|Firmware version of the EtherCAT slave|STRING|RO|01|



## **Index 1018 Identity** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1018:0|Identity|Information for identifyingthe slave|UINT8|RO|0x04(4dec)|
|1018:01|Vendor ID|Vendor ID of the EtherCAT slave|UINT32|RO|0x00000002(2dec)|
|1018:02|Product code|Product code of the EtherCAT slave|UINT32|RO|0x1B873052<br>(461844562dec)|
|1018:03|Revision|Revision number of the EtherCAT slave; the low word (bit<br>0-15) indicates the special terminal number, the high<br>word(bit 16-31)refers to the device description|UINT32|RO|0x00000000 (0dec)|
|1018:04|Serial number|Serial number of the EtherCAT slave; the low byte (bit<br>0-7) of the low word contains the year of production, the<br>high byte (bit 8-15) of the low word contains the week of<br>production, the high word(bit 16-31)is 0|UINT32|RO|0x00000000 (0dec)|



## **Index 10F0 Backup parameter handling** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|10F0:0|Backup parameter<br>handling|Information for standardized loading and saving of<br>backupentries|UINT8|RO|0x01 (1dec)|
|10F0:01|Checksum|Checksum across all backup entries of the EtherCAT<br>slave|UINT32|RO|0x00000000 (0dec)|



## **Index 10F3 Diagnosis History** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|10F3:0|Diagnosis History|Maximum subindex|UINT8|RO|0x37(55dec)|
|10F3:01|Maximum<br>Messages|Maximum number of stored messages. A maximum of 50<br>messages can be stored|UINT8|RO|0x00 (0dec)|
|10F3:02|Newest Message|Subindex of the latest message|UINT8|RO|0x00(0dec)|
|10F3:03|Newest<br>Acknowledged<br>Message|Subindex of the last confirmed message|UINT8|RW|0x00 (0dec)|
|10F3:04|New Messages<br>Available|Indicates that a new message is available|BOOLEAN|RO|0x00 (0dec)|
|10F3:05|Flags|not used|UINT16|RW|0x0000(0dec)|
|10F3:06|Diagnosis<br>Message 001|Message 1|OCTET-<br>STRING[28]|RO|{0}|
|...|...|...|...|...|...|
|10F3:37|Diagnosis<br>Message 050|Message 50|OCTET-<br>STRING[28]|RO|{0}|



210 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 10F8 Actual Time Stamp** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|10F8:0|Actual Time<br>Stamp|Timestamp|UINT64|RO||
|**Index 1400 ENC RxPDO-Par Control compact**||||||
|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|1400:0|ENC RxPDO-Par<br>Control compact|PDO Parameter RxPDO 1|UINT8|RO|0x06 (6dec)|
|1400:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 1|OCTET-<br>STRING[6]|RO|01 16 00 00 00 00|



## **Index 1401 ENC RxPDO-Par Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1401:0|ENC RxPDO-Par<br>Control|PDO Parameter RxPDO 2|UINT8|RO|0x06 (6dec)|
|1401:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 2|OCTET-<br>STRING[6]|RO|00 16 00 00 00 00|



## **Index 1403 STM RxPDO-Par Position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1403:0|STM RxPDO-Par<br>Position|PDO Parameter RxPDO 4|UINT8|RO|0x06 (6dec)|
|1403:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 4|OCTET-<br>STRING[6]|RO|04 16 05 16 06 16|



## **Index 1404 STM RxPDO-Par Velocity** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1404:0|STM RxPDO-Par<br>Velocity|PDO Parameter RxPDO 5|UINT8|RO|0x06 (6dec)|
|1404:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 5|OCTET-<br>STRING[6]|RO|03 16 05 16 06 16|



## **Index 1405 POS RxPDO-Par Control compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1405:0|POS RxPDO-Par<br>Control compact|PDO Parameter RxPDO 6|UINT8|RO|0x06 (6dec)|
|1405:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 6|OCTET-<br>STRING[6]|RO|03 16 04 16 06 16|



## **Index 1406 POS RxPDO-Par Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1406:0|POS RxPDO-Par<br>Control|PDO Parameter RxPDO 7|UINT8|RO|0x06 (6dec)|
|1406:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 7|OCTET-<br>STRING[6]|RO|03 16 04 16 05 16|



EL70x7 

Version: 2.2.0 

211 

Configuration by means of the TwinCAT System Manager 

## **Index 1407 POS RxPDO-Par Control 2** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1407:0|POS RxPDO-Par<br>Control 2|PDO Parameter RxPDO 8|UINT8|RO|0x06 (6dec)|
|1407:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 8|OCTET-<br>STRING[6]|RO|03 16 04 16 05 16|



## **Index 1600 ENC RxPDO-Map Control compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1600:0|ENC RxPDO-Map<br>Control compact|PDO Mapping RxPDO 1|UINT8|RO|0x06 (6dec)|
|1600:01|SubIndex 001|1. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x01(Enable latch C))|UINT32|RO|0x7000:01, 1|
|1600:02|SubIndex 002|2. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x02(Enable latch extern onpositive edge))|UINT32|RO|0x7000:02, 1|
|1600:03|SubIndex 003|3. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x03(Set counter))|UINT32|RO|0x7000:03, 1|
|1600:04|SubIndex 004|4. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry 0x04 (Enable latch extern on negative<br>edge))|UINT32|RO|0x7000:04, 1|
|1600:05|SubIndex 005|5. PDO Mappingentry (12 bits align)|UINT32|RO|0x0000:00, 12|
|1600:06|SubIndex 006|6. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x11(Set counter value))|UINT32|RO|0x7000:11, 16|



## **Index 1601 ENC RxPDO-Map Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1601:0|ENC RxPDO-Map<br>Control|PDO Mapping RxPDO 2|UINT8|RO|0x06 (6dec)|
|1601:01|SubIndex 001|1. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x01(Enable latch C))|UINT32|RO|0x7000:01, 1|
|1601:02|SubIndex 002|2. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x02(Enable latch extern onpositive edge))|UINT32|RO|0x7000:02, 1|
|1601:03|SubIndex 003|3. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x03(Set counter))|UINT32|RO|0x7000:03, 1|
|1601:04|SubIndex 004|4. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry 0x04 (Enable latch extern on negative<br>edge))|UINT32|RO|0x7000:04, 1|
|1601:05|SubIndex 005|5. PDO Mappingentry (12 bits align)|UINT32|RO|0x0000:00, 12|
|1601:06|SubIndex 006|6. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x11(Set counter value))|UINT32|RO|0x7000:11, 32|



## **Index 1602 STM RxPDO-Map Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1602:0|STM RxPDO-Map<br>Control|PDO Mapping RxPDO 3|UINT8|RO|0x06 (6dec)|
|1602:01|SubIndex 001|1. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x01(Enable))|UINT32|RO|0x7010:01, 1|
|1602:02|SubIndex 002|2. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x02(Reset))|UINT32|RO|0x7010:02, 1|
|1602:03|SubIndex 003|3. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x03(Reduce torque))|UINT32|RO|0x7010:03, 1|
|1602:04|SubIndex 004|4. PDO Mappingentry (13 bits align)|UINT32|RO|0x0000:00, 8|
|1602:05|SubIndex 005|5. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x0C(Digital output 1))|UINT32|RO|0x7010:0C, 1|
|1602:06|SubIndex 006|6. PDO Mappingentry (4 bits align)|UINT32|RO|0x0000:00, 4|



212 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1603 STM RxPDO-Map Position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1603:0|STM RxPDO-Map<br>Position|PDO Mapping RxPDO 4|UINT8|RO|0x01 (1dec)|
|1603:01|SubIndex 001|1. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x11(Position))|UINT32|RO|0x7010:11, 32|



## **Index 1604 STM RxPDO-Map Velocity** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1604:0|STM RxPDO-Map<br>Velocity|PDO Mapping RxPDO 5|UINT8|RO|0x01 (1dec)|
|1604:01|SubIndex 001|1. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x21(Velocity))|UINT32|RO|0x7010:21, 16|



## **Index 1605 POS RxPDO-Map Control compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1605:0|POS RxPDO-Map<br>Control compact|PDO Mapping RxPDO 6|UINT8|RO|0x04 (4dec)|
|1605:01|SubIndex 001|1. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x01(Execute))|UINT32|RO|0x7020:01, 1|
|1605:02|SubIndex 002|2. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x02(Emergencystop))|UINT32|RO|0x7020:02, 1|
|1605:03|SubIndex 003|3. PDO Mappingentry (14 bits align)|UINT32|RO|0x0000:00, 14|
|1605:04|SubIndex 004|4. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x11(Targetposition))|UINT32|RO|0x7020:11, 32|



## **Index 1606 POS RxPDO-Map Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1606:0|POS RxPDO-Map<br>Control|PDO Mapping RxPDO 7|UINT8|RO|0x08 (8dec)|
|1606:01|SubIndex 001|1. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x01(Execute))|UINT32|RO|0x7020:01, 1|
|1606:02|SubIndex 002|2. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x02(Emergencystop))|UINT32|RO|0x7020:02, 1|
|1606:03|SubIndex 003|3. PDO Mappingentry (14 bits align)|UINT32|RO|0x0000:00, 14|
|1606:04|SubIndex 004|4. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x11(Targetposition))|UINT32|RO|0x7020:11, 32|
|1606:05|SubIndex 005|5. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x21(Velocity))|UINT32|RO|0x7020:21, 16|
|1606:06|SubIndex 006|6. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x22(Start type))|UINT32|RO|0x7020:22, 16|
|1606:07|SubIndex 007|7. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x23(Acceleration))|UINT32|RO|0x7020:23, 16|
|1606:08|SubIndex 008|8. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x24(Deceleration))|UINT32|RO|0x7020:24, 16|



EL70x7 

Version: 2.2.0 

213 

Configuration by means of the TwinCAT System Manager 

## **Index 1607 POS RxPDO-Map Control 2** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1606:0|POS RxPDO-Map<br>Control|PDO Mapping RxPDO 7|UINT8|RO|0x08 (8dec)|
|1607:01|SubIndex 001|1. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00,2|
|1607:02|SubIndex 002|2. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x03(Enable auto start))|UINT32|RO|0x7021:03, 1|
|1607:03|SubIndex 003|3. PDO Mappingentry (13 bits align)|UINT32|RO|0x0000:00, 13|
|1607:04|SubIndex 004|4. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x11(Targetposition))|UINT32|RO|0x7021:11, 32|
|1607:05|SubIndex 005|5. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x21(Velocity))|UINT32|RO|0x7021:21, 16|
|1607:06|SubIndex 006|6. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x22(Start type))|UINT32|RO|0x7021:22, 16|
|1607:07|SubIndex 007|7. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x23(Acceleration))|UINT32|RO|0x7021:23, 16|
|1607:08|SubIndex 008|8. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x24(Deceleration))|UINT32|RO|0x7021:24, 16|



## **Index 1800 ENC TxPDO-Par Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1800:0|ENC TxPDO-Par<br>Status compact|PDO parameter TxPDO 1|UINT8|RO|0x06 (6dec)|
|1800:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 1|OCTET-<br>STRING[2]|RO|01 1A|



## **Index 1801 ENC TxPDO-Par Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1801:0|ENC TxPDO-Par<br>Status|PDO parameter TxPDO 2|UINT8|RO|0x06 (6dec)|
|1801:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 2|OCTET-<br>STRING[2]|RO|00 1A|



## **Index 1806 POS TxPDO-Par Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1806:0|POS TxPDO-Par<br>Status compact|PDO parameter TxPDO 7|UINT8|RO|0x06 (6dec)|
|1806:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 7|OCTET-<br>STRING[2]|RO|07 1A|



## **Index 1807 POS TxPDO-Par Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1807:0|POS TxPDO-Par<br>Status|PDO parameter TxPDO 8|UINT8|RO|0x06 (6dec)|
|1807:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 8|OCTET-<br>STRING[2]|RO|06 1A|



214 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1A00 ENC TxPDO-Map Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A00:0|ENC TxPDO-Map<br>Status compact|PDO Mapping TxPDO 1|UINT8|RO|0x11 (17dec)|
|1A00:01|SubIndex 001|1. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x01(Latch C valid))|UINT32|RO|0x6000:01, 1|
|1A00:02|SubIndex 002|2. PDO Mappingentry (1 bits align)|UINT32|RO|0x6000:02, 1|
|1A00:03|SubIndex 003|3. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x03(Set counter done))|UINT32|RO|0x6000:03, 1|
|1A00:04|SubIndex 004|4. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x04(Counter underflow))|UINT32|RO|0x6000:04, 1|
|1A00:05|SubIndex 005|5. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x05(Counter overflow))|UINT32|RO|0x6000:05, 1|
|1A00:06|SubIndex 006|6. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00, 2|
|1A00:07|SubIndex 007|7. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x08(Extrapolation stall))|UINT32|RO|0x6000:08, 1|
|1A00:08|SubIndex 008|8. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x09(Status of input A))|UINT32|RO|0x6000:09, 1|
|1A00:09|SubIndex 009|9. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0A(Status of input B))|UINT32|RO|0x6000:0A, 1|
|1A00:0A|SubIndex 010|10. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0B(Status of input C))|UINT32|RO|0x6000:0B, 1|
|1A00:0B|SubIndex 011|11. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00, 1|
|1A00:0C|SubIndex 012|12. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0E(Sync error))|UINT32|RO|0x6000:0D, 1|
|1A00:0D|SubIndex 013|13. PDO Mappingentry (1 bits align)|UINT32|RO|0x6000:0E, 1|
|1A00:0E|SubIndex 014|14. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x0000:00, 1|
|1A00:0F|SubIndex 015|15. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x11(Counter value))|UINT32|RO|0x6000:10, 1|
|1A00:10|SubIndex 016|16. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x12(Latch value))|UINT32|RO|0x6000:11, 16|
|1A00:11|SubIndex 017|17. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x12(Latch value))|UINT32|RO|0x6000:12, 16|



EL70x7 

Version: 2.2.0 

215 

Configuration by means of the TwinCAT System Manager 

## **Index 1A01 ENC TxPDO-Map Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A01:0|ENC TxPDO-Map<br>Status|PDO Mapping TxPDO 2|UINT8|RO|0x11 (17dec)|
|1A01:01|SubIndex 001|1. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x01(Latch C valid))|UINT32|RO|0x6000:01, 1|
|1A01:02|SubIndex 002|2. PDO Mappingentry (1 bits align)|UINT32|RO|0x6000:02, 1|
|1A01:03|SubIndex 003|3. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x03(Set counter done))|UINT32|RO|0x6000:03, 1|
|1A01:04|SubIndex 004|4. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x04(Counter underflow))|UINT32|RO|0x6000:04, 1|
|1A01:05|SubIndex 005|5. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x05(Counter overflow))|UINT32|RO|0x6000:05, 1|
|1A01:06|SubIndex 006|6. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00, 2|
|1A01:07|SubIndex 007|7. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x08(Extrapolation stall))|UINT32|RO|0x6000:08, 1|
|1A01:08|SubIndex 008|8. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x09(Status of input A))|UINT32|RO|0x6000:09, 1|
|1A01:09|SubIndex 009|9. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0A(Status of input B))|UINT32|RO|0x6000:0A, 1|
|1A01:0A|SubIndex 010|10. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0B(Status of input C))|UINT32|RO|0x6000:0B, 1|
|1A01:0B|SubIndex 011|11. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00, 1|
|1A01:0C|SubIndex 012|12. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0E(Sync error))|UINT32|RO|0x6000:0D, 1|
|1A01:0D|SubIndex 013|13. PDO Mappingentry (1 bits align)|UINT32|RO|0x6000:0E, 1|
|1A01:0E|SubIndex 014|14. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x0000:00, 1|
|1A01:0F|SubIndex 015|15. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x11(Counter value))|UINT32|RO|0x6000:10, 1|
|1A01:10|SubIndex 016|16. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x12(Latch value))|UINT32|RO|0x6000:11, 32|
|1A01:11|SubIndex 017|17. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x12(Latch value))|UINT32|RO|0x6000:12, 32|



**Index 1A02 ENC TxPDO-Map Timest. compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A02:0|ENC TxPDO-Map<br>Timest. compact|PDO Mapping TxPDO 3|UINT8|RO|0x01 (1dec)|
|1A02:01|SubIndex 001|1. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x16(Timestamp))|UINT32|RO|0x6000:16, 32|



216 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1A03 STM TxPDO-Map Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A03:0|STM TxPDO-Map<br>Status|PDO Mapping TxPDO 4|UINT8|RO|0x0E (14dec)|
|1A03:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x01(Readyto enable))|UINT32|RO|0x6010:01, 1|
|1A03:02|SubIndex 002|2. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x02(Ready))|UINT32|RO|0x6010:02, 1|
|1A03:03|SubIndex 003|3. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x03(Warning))|UINT32|RO|0x6010:03, 1|
|1A03:04|SubIndex 004|4. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x04(Error))|UINT32|RO|0x6010:04, 1|
|1A03:05|SubIndex 005|5. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x05(Moving positive))|UINT32|RO|0x6010:05, 1|
|1A03:06|SubIndex 006|6. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x06(Movingnegative))|UINT32|RO|0x6010:06, 1|
|1A03:07|SubIndex 007|7. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x07(Torque reduced))|UINT32|RO|0x6010:07, 1|
|1A03:08|SubIndex 008|8. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x08(Motor stall))|UINT32|RO|0x6010:08, 1|
|1A03:09|SubIndex 009|9. PDO Mappingentry (5 bits align)|UINT32|RO|0x0000:00, 3|
|1A03:0A|SubIndex 010|10. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x0E(Sync error))|UINT32|RO|0x6010:0C, 1|
|1A03:0B|SubIndex 011|11. PDO Mappingentry (1 bits align)|UINT32|RO|0x6010:0D, 1|
|1A03:0C|SubIndex 012|12. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x6010:0E, 1|
|1A03:0D|SubIndex 013|13. PDO Mappingentry (1 bits align)|UINT32|RO|0x0000:00, 1|
|1A03:0E|SubIndex 014|14. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x6010:10, 1|



**Index 1A04 STM TxPDO-Map Synchron info data** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A04:0|STM TxPDO-Map<br>Synchron info<br>data|PDO Mapping TxPDO 5|UINT8|RO|0x02 (2dec)|
|1A04:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x11(Info data 1))|UINT32|RO|0x6010:11, 16|
|1A04:02|SubIndex 002|2. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x12(Info data 2))|UINT32|RO|0x6010:12, 16|



## **Index 1A05 STM TxPDO-Map Motor load** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A05:0|STM TxPDO-Map<br>Motor load|PDO Mapping TxPDO 6|UINT8|RO|0x01 (1dec)|
|1A05:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x13(Motor load))|UINT32|RO|0x6010:13, 16|



EL70x7 

Version: 2.2.0 

217 

Configuration by means of the TwinCAT System Manager 

**Index 1A06 POS TxPDO-Map Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A06:0|POS TxPDO-Map<br>Status compact|PDO Mapping TxPDO 7|UINT8|RO|0x09 (9dec)|
|1A06:01|SubIndex 001|1. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x01(Busy))|UINT32|RO|0x6020:01, 1|
|1A06:02|SubIndex 002|2. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x02(In-Target))|UINT32|RO|0x6020:02, 1|
|1A06:03|SubIndex 003|3. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x03(Warning))|UINT32|RO|0x6020:03, 1|
|1A06:04|SubIndex 004|4. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x04(Error))|UINT32|RO|0x6020:04, 1|
|1A06:05|SubIndex 005|5. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x05(Calibrated))|UINT32|RO|0x6020:05, 1|
|1A06:06|SubIndex 006|6. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x06(Accelerate))|UINT32|RO|0x6020:06, 1|
|1A06:07|SubIndex 007|7. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x07(Decelerate))|UINT32|RO|0x6020:07, 1|
|1A06:08|SubIndex 008|8. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x08(Readyto execute))|UINT32|RO|0x6020:08, 1|
|1A06:09|SubIndex 009|9. PDO Mappingentry (8 bits align)|UINT32|RO|0x0000:00, 8|



**Index 1A07 POS TxPDO-Map Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A07:0|POS TxPDO-Map<br>Status|PDO Mapping TxPDO 8|UINT8|RO|0x0C (12dec)|
|1A07:01|SubIndex 001|1. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x01(Busy))|UINT32|RO|0x6020:01, 1|
|1A07:02|SubIndex 002|2. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x02(In-Target))|UINT32|RO|0x6020:02, 1|
|1A07:03|SubIndex 003|3. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x03(Warning))|UINT32|RO|0x6020:03, 1|
|1A07:04|SubIndex 004|4. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x04(Error))|UINT32|RO|0x6020:04, 1|
|1A07:05|SubIndex 005|5. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x05(Calibrated))|UINT32|RO|0x6020:05, 1|
|1A07:06|SubIndex 006|6. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x06(Accelerate))|UINT32|RO|0x6020:06, 1|
|1A07:07|SubIndex 007|7. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x07(Decelerate))|UINT32|RO|0x6020:07, 1|
|1A07:08|SubIndex 008|8. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x08(Readyto execute))|UINT32|RO|0x6020:08, 1|
|1A07:09|SubIndex 009|9. PDO Mappingentry (8 bits align)|UINT32|RO|0x0000:00, 8|
|1A07:0A|SubIndex 010|10. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x11(Actualposition))|UINT32|RO|0x6020:11, 32|
|1A07:0B|SubIndex 011|11. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x21(Actual velocity))|UINT32|RO|0x6020:21, 16|
|1A07:0C|SubIndex 012|12. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x22(Actual drive time))|UINT32|RO|0x6020:22, 32|



**Index 1A08 STM TxPDO-Map Internal position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A08:0|STM TxPDO-Map<br>Internalposition|PDO Mapping TxPDO 9|UINT8|RO|0x01 (1dec)|
|1A08:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x14(Internalposition))|UINT32|RO|0x6010:14, 32|



218 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1A09 STM TxPDO-Map External position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A09:0|STM TxPDO-Map<br>Externalposition|PDO Mapping TxPDO 10|UINT8|RO|0x01 (1dec)|
|1A09:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x15(Externalposition))|UINT32|RO|0x6010:15, 32|



## **Index 1A0A POS TxPDO-Map Actual position lag** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A0A:0|POS TxPDO-Map<br>Actualposition lag|PDO Mapping TxPDO 11|UINT8|RO|0x01 (1dec)|
|1A0A:01|SubIndex 001|1. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x23(Actualposition lag))|UINT32|RO|0x6020:23, 32|



## **Index 1C00 Sync manager type** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C00:0|Sync manager<br>type|Using the sync managers|UINT8|RO|0x04 (4dec)|
|1C00:01|SubIndex 001|Sync-Manager Type Channel 1: Mailbox Write|UINT8|RO|0x01(1dec)|
|1C00:02|SubIndex 002|Sync-Manager Type Channel 2: Mailbox Read|UINT8|RO|0x02(2dec)|
|1C00:03|SubIndex 003|Sync-Manager Type Channel 3: Process Data Write<br>(Outputs)|UINT8|RO|0x03 (3dec)|
|1C00:04|SubIndex 004|Sync-Manager Type Channel 4: Process Data Read<br>(Inputs)|UINT8|RO|0x04 (4dec)|



## **Index 1C12 RxPDO assign** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C12:0|RxPDO assign|PDO Assign Outputs|UINT8|RW|0x03(3dec)|
|1C12:01|Subindex 001|1. allocated RxPDO (contains the index of the associated<br>RxPDO mappingobject)|UINT16|RW|0x1600 (5632dec)|
|1C12:02|Subindex 002|2. allocated RxPDO (contains the index of the associated<br>RxPDO mappingobject)|UINT16|RW|0x1602 (5634dec)|
|1C12:03|Subindex 003|3. allocated RxPDO (contains the index of the associated<br>RxPDO mappingobject)|UINT16|RW|0x1604 (5636dec)|



## **Index 1C13 TxPDO assign** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C13:0|TxPDO assign|PDO Assign Inputs|UINT8|RW|0x02(2dec)|
|1C13:01|Subindex 001|1. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x1A00 (6656dec)|
|1C13:02|Subindex 002|2. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x1A03 (6659dec)|
|1C13:03|Subindex 003|3. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C13:04|Subindex 004|4. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C13:05|Subindex 005|5. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C32:06|Subindex 006|6. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C32:07|Subindex 007|7. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C32:08|Subindex 008|8. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|



EL70x7 

Version: 2.2.0 

219 

Configuration by means of the TwinCAT System Manager 

**Index 1C32 SM output parameter** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C32:0|SM output<br>parameter|Synchronization parameters for the outputs|UINT8|RO|0x20 (32dec)|
|1C32:01|Sync mode|Current synchronization mode:<br>•<br>0: Free Run<br>•<br>1: Synchronous with SM 2 event<br>•<br>2: DC-Mode - Synchronous with SYNC0 Event<br>•<br>3: DC-Mode - Synchronous with SYNC1 event|UINT16|RW|0x0001 (1dec)|
|1C32:02|Cycle time|Cycle time (in ns):<br>•<br>Free Run: Cycle time of the local timer<br>•<br>Synchronous with SM 2 event: Master cycle time<br>•<br>DC-Mode: SYNC0/SYNC1 Cycle Time|UINT32|RW|0x000F4240<br>(1000000dec)|
|1C32:03|Shift time|Time between SYNC0 event and output of the outputs (in<br>ns, DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:04|Sync modes<br>supported|Supported synchronization modes:<br>•<br>Bit 0 = 1: free run is supported<br>•<br>Bit 1 = 1: Synchronous with SM 2 event is supported<br>•<br>Bit 2-3 = 01: DC mode is supported<br>•<br>Bit 4-5 = 10: Output shift with SYNC1 event (only<br>DC mode)<br>•<br>Bit 14 = 1: dynamic times (measurement through<br>writingof 0x1C32:08)|UINT16|RO|0x0C07 (3079dec)|
|1C32:05|Minimum cycle<br>time|Minimum cycle time (in ns)|UINT32|RO|0x0003D090<br>(250000dec)|
|1C32:06|Calc and copy<br>time|Minimum time between SYNC0 and SYNC1 event (in ns,<br>DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:07|Minimum delay<br>time|Min. time between SYNC1 event and output of the<br>outputs(in ns, DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:08|Command|•<br>0: Measurement of the local cycle time is stopped<br>•<br>1: Measurement of the local cycle time is started<br>The entries 0x1C32:03, 0x1C32:05, 0x1C32:06,<br>0x1C32:07, 0x1C32:09, 0x1C33:03, 0x1C33:06, and<br>0x1C33:09 are updated with the maximum measured<br>values.<br>For a subsequent measurement the measured values<br>are reset|UINT16|RW|0x0000 (0dec)|
|1C32:09|Maximum delay<br>time|Max. time between SYNC1 event and output of the<br>outputs(in ns, DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:0B|SM event missed<br>counter|Number of missed SM events in OPERATIONAL (DC<br>mode only)|UINT16|RO|0x0000 (0dec)|
|1C32:0C|Cycle exceeded<br>counter|Number of occasions the cycle time was exceeded in<br>OPERATIONAL (cycle was not completed in time or the<br>next cycle began too early)|UINT16|RO|0x0000 (0dec)|
|1C32:0D|Shift too short<br>counter|Number of occasions that the interval between SYNC0<br>and SYNC1 event was too short(DC mode only)|UINT16|RO|0x0000 (0dec)|
|1C32:14|Frame repeat time||UINT32|RW|0x00000000(0dec)|
|1C32:20|Sync error|The synchronization was not correct in the last cycle,<br>(outputs were output too late; DC mode only)|BOOLEAN|RO|0x00 (0dec)|



220 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1C33 SM input parameter** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|1C33:0|SM input<br>parameter|Synchronization parameters for the inputs||UINT8|RO|0x20 (32dec)|
|1C33:01|Sync mode|Current synchronization mode:<br>•<br>0: Free Run<br>•<br>1: Synchronous with SM 3 event (no outputs<br>available)<br>•<br>2: DC - Synchronous with SYNC0 Event<br>•<br>3: DC - Synchronous with SYNC1 Event<br>•<br>34: Synchronous with SM 2 event (outputs<br>available)||UINT16|RW|0x0022 (34dec)|
|1C33:02|Cycle time|as|0x1C32:02 [<br>}<br>220<br>]|UINT32|RW|0x000F4240<br>(1000000dec)|
|1C33:03|Shift time|Time between SYNC0 event and reading of the inputs (in<br>ns, onlyDC mode)||UINT32|RO|0x00000000 (0dec)|
|1C33:04|Sync modes<br>supported|Supported synchronization modes:<br>•<br>Bit 0: free run is supported<br>•<br>Bit 1: synchronous with SM 2 event is supported<br>(outputs available)<br>•<br>Bit 1: synchronous with SM 3 event is supported (no<br>outputs available)<br>•<br>Bit 2-3 = 01: DC mode is supported<br>•<br>Bit 4-5 = 01: input shift through local event (outputs<br>available)<br>•<br>Bit 4-5 = 10: input shift with SYNC1 event (no<br>outputs available)<br>•<br>Bit 14 = 1: dynamic times (measurement through<br>writingof 0x1C32:08 [<br>}<br>220<br>]<br>or 1C33:08)||UINT16|RO|0x0C07 (3079dec)|
|1C33:05|Minimum cycle<br>time|as|0x1C32:05 [<br>}<br>220<br>]|UINT32|RO|0x0003D090<br>(250000dec)|
|1C33:06|Calc and copy<br>time|Time between reading of the inputs and availability of the<br>inputs for the master(in ns, onlyDC mode)||UINT32|RO|0x00000000 (0dec)|
|1C33:07|Minimum delay<br>time|Min. time between SYNC1 event and output of the inputs<br>(in ns, DC mode only)||UINT32|RO|0x00000000 (0dec)|
|1C33:08|Command|as|0x1C32:08 [<br>}<br>220<br>]|UINT16|RW|0x0000 (0dec)|
|1C33:09|Maximum delay<br>time|Max. time between SYNC1 event and reading of the<br>inputs(in ns, onlyDC mode)||UINT32|RO|0x00000000 (0dec)|
|1C33:0B|SM event missed<br>counter|as|0x1C32:11 [<br>}<br>220<br>]|UINT16|RO|0x0000 (0dec)|
|1C33:0C|Cycle exceeded<br>counter|as|0x1C32:12 [<br>}<br>220<br>]|UINT16|RO|0x0000 (0dec)|
|1C33:0D|Shift too short<br>counter|as|0x1C32:13 [<br>}<br>220<br>]|UINT16|RO|0x0000 (0dec)|
|1C33:14|Frame repeat time|as|0x1C32:14 [<br>}<br>220<br>]|UINT32|RW|0x00000000 (0dec)|
|1C33:20|Sync error|as|0x1C32:32 [<br>}<br>220<br>]|BOOLEAN|RO|0x00 (0dec)|



## **Index F000 Modular device profile** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|F000:0|Modular device<br>profile|General information for the modular device profile|UINT8|RO|0x02 (2dec)|
|F000:01|Module index<br>distance|Index spacing of the objects of the individual channels|UINT16|RO|0x0010 (16dec)|
|F000:02|Maximum number<br>of modules|Number of channels|UINT16|RO|0x0003 (3dec)|



EL70x7 

Version: 2.2.0 

221 

Configuration by means of the TwinCAT System Manager 

## **Index F008 Code word** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|F008:0|Code word|see note! [<br>}<br>38<br>]|UINT32|RW|0x00000000 (0dec)|



222 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **7.2 EL7047 - Object description and parameterization** 

## **EtherCAT XML Device Description** 

The display matches that of the CoE objects from the EtherCAT XML Device Description. We recommend downloading the latest XML file from the download area of the Beckhoff website and installing it according to installation instructions. 

## **Parameterization via the CoE list (CAN over EtherCAT)** 

The terminal is parameterized via the CoE - Online tab (double-click on the respective object) or via the Process Data tab (allocation of PDOs). Please note the following general CoE information when using/manipulating the CoE parameters [} 37]: 

- Keep a startup list if components have to be replaced 

- Differentiation between online/offline dictionary, existence of current XML description 

- use "CoE reload" for resetting changes 

## _**NOTICE**_ 

## **Risk of damage to the device!** 

We strongly advise not to change settings in the CoE objects while the axis is active, since this could impair the control. 

## **Introduction** 

The CoE overview contains objects for different intended applications: 

## **Object overview** 

- Restore object [} 223] 

- Configuration data [} 224] 

- Command object [} 228] 

- Input data [} 229] 

- Output data [} 230] 

- Information / diagnostic data (channel specific) [} 233] 

- Manufacturer configuration data (device-specific) [} 234] 

- Information / diagnostic data (device-specific) [} 235] 

- Standard objects [} 235] 

## **7.2.1 Restore object** 

**Index 1011 Restore default parameters** 

**Index Name Meaning Data type Flags Default (hex)** 1011:0 Restore default Restore default parameters UINT8 RO 0x01 (1dec) parameters 1011:01 SubIndex 001 If this object is set to **"0x64616F6C"** in the set value UINT32 RW 0x00000000 (0dec) ~~ae~~ dialog, all backup objects are reset to their delivery state. 

EL70x7 

Version: 2.2.0 

223 

Configuration by means of the TwinCAT System Manager 

## **7.2.2 Configuration data** 

## **Index 8000 ENC Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8000:0|ENC Settings<br>Ch.1|Maximum subindex|UINT8|RO|0x0E (14dec)|
|8000:08|Disable filter|Deactivates the input filters.|BOOLEAN|RW|0x00(0dec)|
|8000:0A|Enable micro<br>increments|The lower 8 bits of the counter value are extrapolated.|BOOLEAN|RW|0x00 (0dec)|
|8000:0E|Reversion of<br>rotation|Activates reversion of rotation of the encoder.|BOOLEAN|RW|0x00 (0dec)|



## **Index 8010 STM Motor Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8010:0|STM Motor<br>Settings Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|8010:01|Maximal current|Maximum permanent motor coil current<br>**Unit**: 1 mA|UINT16|RW|0x1388 (5000dec)|
|8010:02|Reduced current|Reduced coil current<br>**Unit**: 1 mA|UINT16|RW|0x09C4 (2500dec)|
|8010:03|Nominal voltage|Nominal voltage (supply voltage) of the motor<br>**Unit**: 10 mV|UINT16|RW|0x1388 (5000dec)|
|8010:04|Motor coil<br>resistance|Internal resistance of the motor<br>**Unit**: 10 mOhm|UINT16|RW|0x0064 (100dec)|
|8010:05|Motor EMF|Countervoltage of the motor<br>**Unit**: 1 mV /(rad/s)|UINT16|RW|0x0000 (0dec)|
|8010:06|Motor fullsteps|Number of full motor steps|UINT16|RW|0x00C8(200dec)|
|8010:07|Encoder<br>increments (4-<br>fold)|Number of encoder increments per revolution with<br>quadruple evaluation|UINT16|RW|0x1000 (4096dec)|
|8010:09|Start velocity|Minimum starting velocity of the motor<br>**Unit**:10000 corresponds to 100% [<br>}<br>167<br>]|UINT16|RW|0x0000 (0dec)|
|8010:0A|Motor coil<br>inductance|Inductance of the motor<br>**Unit**: 0.01 mH|UINT16|RW|0x0000 (0dec)|
|8010:10|Drive on delay<br>time|Delay between activation of driver stage and<br>„ready= 1“|UINT16|RW|0x0064 (100dec)|
|8010:11|Drive off delay<br>time|Delay between deactivation of driver stage and<br>„ready= 0“|UINT16|RW|0x0096 (150dec)|



## **Index 8011 STM Controller Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8011:0|STM Controller<br>Settings Ch.1|Maximum subindex|UINT8|RO|0x02 (2dec)|
|8011:01|Kpfactor(curr.)|Kpcontrol factor of the current controller|UINT16|RW|0x0096(150dec)|
|8011:02|Ki factor(curr.)|Ki control factor of the current controller|UINT16|RW|0x000A(10dec)|



224 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 8012 STM Features Ch.1 (part 1)** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8012:0|STM Features<br>Ch.1|Maximum subindex|UINT8|RO|0x3A (58dec)|
|8012:01|Operation mode|permitted values:|BIT4|RW|0x00 (0dec)|
|||0: Automatic||||
|||1: Velocitydirect||||
|||3: Position controller||||
|||4: Ext. Velocitymode||||
|||5: Ext. Position mode||||
|||6: Velocitysensorless||||
|8012:05|Speed range|permitted values:|BIT3|RW|0x01 (1dec)|
|||0: 1000 Fullsteps/sec||||
|||1: 2000 Fullsteps/sec||||
|||2: 4000 Fullsteps/sec||||
|||3: 8000 Fullsteps/sec||||
|||4: 16000 Fullsteps/sec||||
|||5: 32000 Fullsteps/sec||||
|8012:08|Feedback type|permitted values:|BIT1|RW|0x01 (1dec)|
|||0: Encoder||||
|||1: Internal counter||||
|8012:09|Invert motor<br>polarity|Invert the direction of rotation of the motor|BOOLEAN|RW|0x00 (0dec)|
|8012:0A|Error on steplost|Error on loss of step|BOOLEAN|RW|0x00(0dec)|
|8012:0B|Fan cartridge<br>present|Fan cartridge present|BOOLEAN|RW|0x00 (0dec)|
|8012:11|Select info data 1|permitted values:|UINT8|RW|0x0B (11dec)|
|||0: Status word||||
|||7: Motor velocity||||
|||11: Motor load||||
|||13: Motor dc current||||
|||101: Internal temperature||||
|||103: Control voltage||||
|||104: Motor supplyvoltage||||
|||150: Drive - Status word||||
|||151: Drive – State||||
|||152: Drive - Position lag (low word)||||
|||153: Drive - Position lag (high word)||||



EL70x7 

Version: 2.2.0 

225 

Configuration by means of the TwinCAT System Manager 

## **Index 8012 STM Features Ch.1 (part 2)** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8012:19|Select info data 2|permitted values:|UINT8|RW|0x0D (13dec)|
|||0: Status word||||
|||7: Motor velocity||||
|||11: Motor load||||
|||13: Motor dc current||||
|||101: Internal temperature||||
|||103: Control voltage||||
|||104: Motor supplyvoltage||||
|||150: Drive - Status word||||
|||151: Drive - State||||
|||152: Drive - Position lag (low word)||||
|||153: Drive - Position lag (high word)||||
|8012:30|Invert digital input<br>1|Invert digital input|BOOLEAN|RW|0x00 (0dec)|
|8012:31|Invert digital input<br>2|Invert digital input|BOOLEAN|RW|0x00 (0dec)|
|8012:32|Function for input<br>1|permitted values:|BIT4|RW|0x00 (0dec)|
|||0: Normal input||||
|||1: Hardware enable||||
|||2: PLC cam||||
|8012:36|Function for input<br>2|permitted values:|BIT4|RW|0x00 (0dec)|
|||0: Normal input||||
|||1: Hardware enable||||
|||2: PLC cam||||
|8012:3A|Function for<br>output 1|permitted values:|BIT4|RW|0x0F (15dec)|
|||0: Normal output||||
|||1: Break(linked with driver enable)||||
|||15: Disabled||||



**Index 8014 STM Controller Settings 3 Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8014:0|STM Controller<br>Settings 3 Ch.1|Maximum subindex|UINT8|RO|0x09 (9dec)|
|8014:01|Feed forward<br>(pos.)|Pilot control of the position controller|UINT32|RW|0x000186A0<br>(100000dec)|
|8014:02|Kpfactor(pos.)|Kpcontrol factor of theposition controller|UINT16|RW|0x01F4(500dec)|
|8014:03|Kp factor (velo.)|Kp control factor of the velocity controller<br>**Unit**: 0.1 mA /(rad/s)|UINT32|RW|0x00000032<br>(50dec)|
|8014:04|Tn (velo.)|Time constant Tn of the velocity controller<br>**Unit**: 0.01 ms|UINT16|RW|0xC350 (50000dec)|
|8014:05|Sensorless param<br>1|First parameter (sensorless control)|UINT16|RW|0x0000 (0dec)|
|8014:06|Sensorless param<br>2|Second parameter (sensorless control)|UINT16|RW|0x0000 (0dec)|
|8014:07|Cross over<br>velocity1|First velocity transition (sensorless control)<br>**Unit**: 0.1 rad/s|UINT16|RW|0x0000 (0dec)|
|8014:08|Cross over<br>velocity2|Second velocity transition (sensorless control)<br>**Unit**: 0.1 rad/s|UINT16|RW|0x0000 (0dec)|
|8014:09|Cross over<br>velocity3|Third velocity transition (sensorless control)<br>**Unit**: 0.1 rad/s|UINT16|RW|0x0000 (0dec)|



226 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 8020 POS Settings Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8020:0|POS Settings<br>Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|8020:01|Velocity min.|Minimum set velocity<br>(range: 0-10000)|INT16|RW|0x0064 (100dec)|
|8020:02|Velocity max.|Maximum set velocity<br>(range: 0-10000)|INT16|RW|0x2710 (10000dec)|
|8020:03|Acceleration pos.|Acceleration in positive direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:04|Acceleration neg.|Acceleration in negative direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:05|Deceleration pos.|Deceleration in positive direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:06|Deceleration neg.|Deceleration in negative direction of rotation<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:07|Emergency<br>deceleration|Emergency deceleration (both directions of rotation)<br>**Unit**: 1 ms|UINT16|RW|0x0064 (100dec)|
|8020:08|Calibration<br>position|Calibration position|UINT32|RW|0x00000000 (0dec)|
|8020:09|Calibration<br>velocity (towards<br>plc cam)|Calibration velocity towards the cam<br>(range: 0-10000)|INT16|RW|0x0064 (100dec)|
|8020:0A|Calibration<br>Velocity (off plc<br>cam)|Calibration velocity away from the cam<br>(range: 0-10000)|INT16|RW|0x000A (10dec)|
|8020:0B|Target window|Target window|UINT16|RW|0x000A(10dec)|
|8020:0C|In-Target timeout|Target position timeout<br>**Unit**: 1 ms|UINT16|RW|0x03E8 (1000dec)|
|8020:0D|Dead time<br>compensation|Dead time compensation<br>**Unit**: 1µs|INT16|RW|0x0032 (50dec)|
|8020:0E|Modulo factor|Modulo factor/position|UINT32|RW|0x00000000(0dec)|
|8020:0F|Modulo tolerance<br>window|Tolerance window for modulo positioning|UINT32|RW|0x00000000 (0dec)|
|8020:10|Position lagmax.|Maximum allowable steperror|UINT16|RW|0x0000(0dec)|
|8020:11|Calibration<br>acceleration<br>(aroundplc cam)|Acceleration and braking ramps for homing runs|UINT16|RW|0x0000 (0dec)|



EL70x7 

Version: 2.2.0 

227 

Configuration by means of the TwinCAT System Manager 

## **Index 8021 POS Features Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|8021:0|POS Features<br>Ch.1|Maximum subindex|UINT8|RO|0x16 (22dec)|
|8021:01|Start type|permitted values:|UINT16|RW|0x0001 (1dec)|
|||0: Idle||||
|||1: Absolute||||
|||2: Relative||||
|||3: Endlessplus||||
|||4: Endless minus||||
|||6: Additive||||
|||24832: Calibration(Hardware sync)||||
|||24576: Calibration(Plc cam)||||
|||28416: Calibration(Clear manual)||||
|||28160: Calibration(Set manual)||||
|||28161: Calibration(Set manual auto)||||
|||1029: Modulo current||||
|||773: Modulo minus||||
|||517: Moduloplus||||
|||261: Modulo short||||
|8021:11|Time information|permitted values:|BIT2|RW|0x00 (0dec)|
|||0: Elapsed time<br>current drive time since start of the travel command||||
|8021:13|Invert calibration<br>cam search<br>direction|Inversion of the direction of rotation towards the cam|BOOLEAN|RW|0x01 (1dec)|
|8021:14|Invert sync<br>impulse search<br>direction|Inversion of the direction of rotation away from the cam|BOOLEAN|RW|0x00 (0dec)|
|8021:15|Emergency stop<br>on position lag<br>error|Triggers an emergency stop if the maximum following<br>error is exceeded.|BOOLEAN|RW|0x00 (0dec)|
|8021:16|Enhanced diag<br>history|Provides detailed messages about the status of the<br>positioninginterface in the diaghistory.|BOOLEAN|RW|0x00 (0dec)|



## **7.2.3 Command object** 

## **Index FB00 STM Command** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|FB00:0|STM Command|Maximum subindex|UINT8|RO|0x03(3dec)|
|FB00:01|Request|Requesting a command<br>0x8000: Software reset|OCTET-<br>STRING[2]|RW|{0}|
|FB00:02|Status|Status of the command|UINT8|RO|0x00 (0dec)|
|||0: No error, without return value||||
|||1: No error, with return value||||
|||2: With error, without return value||||
|||3: With error, with return value||||
|||... reserved||||
|||255: Command execution active||||
|FB00:03|Response|Return value of the executed command|OCTET-<br>STRING[4]|RO|{0}|



228 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **7.2.4 Input data** 

## **Index 6000 ENC Inputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|6000:0|ENC Inputs Ch.1|Maximum subindex|UINT8|RO|0x16(22dec)|
|6000:01|Latch C valid|The counter value was latched with the C track.|BOOLEAN|RO|0x00(0dec)|
|6000:02|Latch extern valid|The counter value was stored via the external latch.|BOOLEAN|RO|0x00(0dec)|
|6000:03|Set counter done|The counter was set.|BOOLEAN|RO|0x00(0dec)|
|6000:04|Counter underflow|Counter underflow|BOOLEAN|RO|0x00(0dec)|
|6000:05|Counter overflow|Counter overflow|BOOLEAN|RO|0x00(0dec)|
|6000:08|Extrapolation stall|The extrapolatedpart of the counter is invalid.|BOOLEAN|RO|0x00(0dec)|
|6000:09|Status of input A|Status of the A-input|BOOLEAN|RO|0x00(0dec)|
|6000:0A|Status of input B|Status of the B-input|BOOLEAN|RO|0x00(0dec)|
|6000:0B|Status of input C|Status of the C-input|BOOLEAN|RO|0x00(0dec)|
|6000:0D|Status of extern<br>latch|Status of the ext. latch input|BOOLEAN|RO|0x00 (0dec)|
|6000:0E|Sync error|The Sync error bit is only required for DC mode. It<br>indicates whether a synchronization error has occurred<br>duringtheprevious cycle.|BOOLEAN|RO|0x00 (0dec)|
|6000:10|TxPDO Toggle|The TxPDO toggle is toggled by the slave when the data<br>of the associated TxPDO is updated.|BOOLEAN|RO|0x00 (0dec)|
|6000:11|Counter value|The counter value|UINT32|RO|0x00000000(0dec)|
|6000:12|Latch value|The latch value|UINT32|RO|0x00000000(0dec)|
|6000:16|Timestamp|Time stampof the last counter change|UINT32|RO|0x00000000(0dec)|



## **Index 6010 STM Inputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**||**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|---|
|6010:0|STM Inputs Ch.1|Maximum subindex|||UINT8|RO|0x15(21dec)|
|6010:01|Readyto enable|Driver stage is readyfor enabling|||BOOLEAN|RO|0x00(0dec)|
|6010:02|Ready|Driver stage is readyfor operation|||BOOLEAN|RO|0x00(0dec)|
|6010:03|Warning|A warninghas occurred(see index||0xA010 [<br>}<br>234<br>]<br>).|BOOLEAN|RO|0x00 (0dec)|
|6010:04|Error|An error has occurred(see index|0xA010 [<br>}<br>234<br>]<br>).||BOOLEAN|RO|0x00 (0dec)|
|6010:05|Moving positive|Driver stage is activated inpositive direction.|||BOOLEAN|RO|0x00(0dec)|
|6010:06|Movingnegative|Driver stage is activated in negative direction.|||BOOLEAN|RO|0x00(0dec)|
|6010:07|Torque reduced|Reduced torque is active.|||BOOLEAN|RO|0x00(0dec)|
|6010:08|Motor stall|A loss of stephas occurred.|||BOOLEAN|RO|0x00(0dec)|
|6010:0C|Digital input 1|Digital input 1|||BOOLEAN|RO|0x00(0dec)|
|6010:0D|Digital input 2|Digital input 2|||BOOLEAN|RO|0x00(0dec)|
|6010:0E|Sync error|The Sync error bit is only required for DC mode. It<br>indicates whether a synchronization error has occurred<br>duringtheprevious cycle.|||BOOLEAN|RO|0x00 (0dec)|
|6010:10|TxPDO Toggle|The TxPDO toggle is toggled by the slave when the data<br>of the associated TxPDO is updated.|||BOOLEAN|RO|0x00 (0dec)|
|6010:11|Info data 1|Synchronous information (selection via subindex<br>0x8012:11 [<br>}<br>225<br>]<br>)|||UINT16|RO|0x0000 (0dec)|
|6010:12|Info data 2|Synchronous information (selection via subindex<br>0x8012:19 [<br>}<br>225<br>]<br>)|||UINT16|RO|0x0000 (0dec)|
|6010:13|Motor load|Current motor load<br>**Unit:** 0.01°|||INT16|RO|0x0000 (0dec)|
|6010:14|Internalposition|Internal microstep position|||UINT32|RO|0x00000000(0dec)|
|6010:15|Externalposition|Encoderposition|||UINT32|RO|0x00000000(0dec)|



EL70x7 

Version: 2.2.0 

229 

Configuration by means of the TwinCAT System Manager 

## **Index 6020 POS Inputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|6020:0|POS Inputs Ch.1|Maximum subindex|UINT8|RO|0x23(35dec)|
|6020:01|Busy|A current travel command is active.|BOOLEAN|RO|0x00(0dec)|
|6020:02|In-Target|Motor has arrived at target.|BOOLEAN|RO|0x00(0dec)|
|6020:03|Warning|A warninghas occurred.|BOOLEAN|RO|0x00(0dec)|
|6020:04|Error|An error has occurred.|BOOLEAN|RO|0x00(0dec)|
|6020:05|Calibrated|The Motor is calibrated.|BOOLEAN|RO|0x00(0dec)|
|6020:06|Accelerate|The Motor is in the accelerationphase.|BOOLEAN|RO|0x00(0dec)|
|6020:07|Decelerate|The Motor is in the decelerationphase.|BOOLEAN|RO|0x00(0dec)|
|6020:11|Actualposition|Current targetposition of the travel commandgenerator|UINT32|RO|0x00000000(0dec)|
|6020:21|Actual velocity|Current set velocityof the travel commandgenerator|INT16|RO|0x0000(0dec)|
|6020:22|Actual drive time|Travel command time information (see subindex<br>0x8021:11 [<br>}<br>228<br>]<br>)|UINT32|RO|0x00000000 (0dec)|
|6020:23|Actualposition lag|Lagofposition|UINT32|RO|0x00000000(0dec)|



## **7.2.5 Output data** 

## **Index 7000 ENC Outputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|7000:0|ENC Outputs<br>Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|7000:01|Enable latch C|Activate latchingvia the C-track.|BOOLEAN|RO|0x00(0dec)|
|7000:02|Enable latch<br>extern on positive<br>edge|Activate external latch with positive edge.|BOOLEAN|RO|0x00 (0dec)|
|7000:03|Set counter|Set the counter value.|BOOLEAN|RO|0x00(0dec)|
|7000:04|Enable latch<br>extern on<br>negative edge|Activate external latch with negative edge.|BOOLEAN|RO|0x00 (0dec)|
|7000:11|Set counter value|This is the counter value to be set via "Set counter".|UINT32|RO|0x00000000(0dec)|



## **Index 7010 STM Outputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|7010:0|STM Outputs<br>Ch.1|Maximum subindex|UINT8|RO|0x21 (33dec)|
|7010:01|Enable|Activates the output stage|BOOLEAN|RO|0x00(0dec)|
|7010:02|Reset|All errors that may have occurred are reset by setting this<br>bit(risingedge).|BOOLEAN|RO|0x00 (0dec)|
|7010:03|Reduce torque|Reduced torque (coil current) is active (see subindex<br>0x8010:02 [<br>}<br>224<br>]<br>).|BOOLEAN|RO|0x00 (0dec)|
|7010:0C|Digital output 1|Digital output 1|BOOLEAN|RO|0x00(0dec)|
|7010:11|Position|Set position specification<br>**Unit:**Increments [<br>}<br>170<br>]|UINT32|RO|0x00000000 (0dec)|
|7010:21|Velocity|Set velocity specification<br>**Unit:**+/-32767 corresponds to +/-100% [<br>}<br>167<br>]|INT16|RO|0x0000 (0dec)|



230 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 7020 POS Outputs Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|7020:0|POS Outputs<br>Ch.1|Maximum subindex||UINT8|RO|0x24 (36dec)|
|7020:01|Execute|Start travel command (rising edge), or prematurely abort<br>travel command(fallingedge)||BOOLEAN|RO|0x00 (0dec)|
|7020:02|Emergency Stop|Prematurely abort travel command with an emergency<br>ramp (risingedge)||BOOLEAN|RO|0x00 (0dec)|
|7020:11|Targetposition|Specification of the targetposition||UINT32|RO|0x00000000(0dec)|
|7020:21|Velocity|Specification of the maximum set velocity||INT16|RO|0x0000(0dec)|
|7020:22|Start type|0x0000<br>Idle|No travel command is being<br>executed|UINT16|RO|0x0000 (0dec)|
|||0x0001<br>Absolute|Absolute target position||||
|||0x1001<br>Absolute(Change)|Change during an active travel<br>command||||
|||0x0002<br>Relative|Target position relative to the<br>currentposition||||
|||0x1002<br>Relative(Change)|Change during an active travel<br>command||||
|||0x0003<br>Endlessplus|Endless driving in positive<br>direction of rotation||||
|||0x0004<br>Endless minus|Endless driving in negative<br>direction of rotation||||
|||0x0105<br>Modulo short|Shortest distance to the next<br>moduloposition||||
|||0x0115<br>Modulo short<br>extended|Shortest distance to the next<br>modulo position (without modulo<br>window)||||
|||0x0205<br>Modulo plus|Drive in positive direction of<br>rotation to the next modulo<br>position||||
|||0x0215<br>Modulo plus<br>extended|Drive in positive direction of<br>rotation to the next modulo<br>position(without modulo window)||||
|||0x0305<br>Modulo minus|Drive in negative direction of<br>rotation to the next modulo<br>position||||
|||0x0315<br>Modulo minus<br>extended|Drive in negative direction of<br>rotation to the next modulo<br>position(without modulo window)||||
|||0x0405<br>Modulo current|Drive in the last implemented<br>direction of rotation to the next<br>moduloposition||||
|||0x0415<br>Modulo current<br>extended|Drive in the last implemented<br>direction of rotation to the next<br>modulo position (without modulo<br>window)||||
|||0x0006<br>Additive|New target position relative/<br>additive to the last targetposition||||
|||0x1006<br>Additive(Change)|Change during an active travel<br>command||||
|||0x6000<br>Calibration, PLC cam|Calibration with cam||||
|||0x6100<br>Calibration, HW sync|Calibration with cam and C-track||||
|||0x6E00<br>Calibration, set<br>manual|Set calibration manually||||
|||0x6E01<br>Calibration, set<br>manual auto|Set automatic calibration, for<br>"Enable = 1"||||
|||0x6F00<br>Calibration, clear<br>manual|Clear calibration manually||||
|7020:23|Acceleration|Acceleration specification||UINT16|RO|0x0000(0dec)|
|7020:24|Deceleration|Deceleration specification||UINT16|RO|0x0000(0dec)|



EL70x7 

Version: 2.2.0 

231 

Configuration by means of the TwinCAT System Manager 

## **Index 7021 POS Outputs 2 Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|7021:0|POS Outputs<br>Ch.1|Maximum subindex||UINT8|RO|0x24 (36dec)|
|7021:03|Enable auto start|Enable auto start||BOOLEAN|RO|0x00(0dec)|
|7021:11|Targetposition|Specification of the targetposition||UINT32|RO|0x00000000(0dec)|
|7021:21|Velocity|Specification of the maximum set velocity||INT16|RO|0x0000(0dec)|
|7021:22|Start type|0x0000<br>Idle|No travel command is being<br>executed|UINT16|RO|0x0000 (0dec)|
|||0x0001<br>Absolute|Absolute target position||||
|||0x1001<br>Absolute(Change)|Change during an active travel<br>command||||
|||0x0002<br>Relative|Target position relative to the<br>currentposition||||
|||0x1002<br>Relative(Change)|Change during an active travel<br>command||||
|||0x0003<br>Endlessplus|Endless driving in positive<br>direction of rotation||||
|||0x0004<br>Endless minus|Endless driving in negative<br>direction of rotation||||
|||0x0105<br>Modulo short|Shortest distance to the next<br>moduloposition||||
|||0x0115<br>Modulo short<br>extended|Shortest distance to the next<br>modulo position (without modulo<br>window)||||
|||0x0205<br>Modulo plus|Drive in positive direction of<br>rotation to the next modulo<br>position||||
|||0x0215<br>Modulo plus<br>extended|Drive in positive direction of<br>rotation to the next modulo<br>position(without modulo window)||||
|||0x0305<br>Modulo minus|Drive in negative direction of<br>rotation to the next modulo<br>position||||
|||0x0315<br>Modulo minus<br>extended|Drive in negative direction of<br>rotation to the next modulo<br>position(without modulo window)||||
|||0x0405<br>Modulo current|Drive in the last implemented<br>direction of rotation to the next<br>moduloposition||||
|||0x0415<br>Modulo current<br>extended|Drive in the last implemented<br>direction of rotation to the next<br>modulo position (without modulo<br>window)||||
|||0x0006<br>Additive|New target position relative/<br>additive to the last targetposition||||
|||0x1006<br>Additive(Change)|Change during an active travel<br>command||||
|||0x6000<br>Calibration, PLC cam|Calibration with cam||||
|||0x6100<br>Calibration, HW sync|Calibration with cam and C-track||||
|||0x6E00<br>Calibration, set<br>manual|Set calibration manually||||
|||0x6E01<br>Calibration, set<br>manual auto|Set automatic calibration, for<br>"Enable = 1"||||
|||0x6F00<br>Calibration, clear<br>manual|Clear calibration manually||||
|7021:23|Acceleration|Acceleration specification||UINT16|RO|0x0000(0dec)|
|7021:24|Deceleration|Deceleration specification||UINT16|RO|0x0000(0dec)|



232 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **7.2.6 Information / diagnostic data (channel specific)** 

## **Index 9010 STM Info data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**||**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|9010:0|STM Info data<br>Ch.1|Maximum subindex||UINT8|RO|0x13 (19dec)|
|9010:01|Status word|Status word(see index|0xA010 [<br>}<br>234<br>]<br>)|UINT16|RO|0x0000 (0dec)|
|9010:08|Motor velocity|Current motor velocity||INT16|RO|0x0000(0dec)|
|9010:09|Internalposition|Internalposition(micro|increments)|UINT32|RO|0x00000000(0dec)|
|9010:0B|Motor load|Current motor load<br>**Unit**: 0.01°||INT16|RO|0x0000 (0dec)|
|9010:0D|Motor dc current|Current motor current (DC vector)<br>**Unit**: 1 mA||INT16|RO|0x0000 (0dec)|
|9010:0E|Tn (curr.)|Internally calculated time constant of the current<br>controller<br>**Unit**: 0.01 ms||UINT16|RO|0x0000 (0dec)|
|9010:13|Externalposition|Externalposition(connected encoder)||UINT32|RO|0x00000000(0dec)|



## **Index 9020 POS Info data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|9020:0|POS Info data<br>Ch.1|Maximum subindex|UINT8|RO|0x04 (4dec)|
|9020:01|Status word|Status word|UINT16|RO|0x0000(0dec)|
|9020:03|State (drive<br>controller)|permitted values:|UINT16|RO|0x0000 (0dec)|
|||0: Init||||
|||1: Idle||||
|||272: Go cam||||
|||273: On cam||||
|||16: Start||||
|||17: Acceleration||||
|||18: Constant||||
|||19: Deceleration||||
|||288: Go sync impulse||||
|||289: Leave cam||||
|||4096: Pre target||||
|||4097: In target||||
|||32: EmergencyStop||||
|||33: Normal stop||||
|||304: Calibration stop||||
|||8192: Drive end||||
|||8193: Wait for init||||
|||320: Is calibrated||||
|||321: Not calibrated||||
|||16384: Drive warning||||
|||32768: Error||||
|||65535: Undefined||||
|||256: Calibration start||||
|9020:04|Actualposition lag|Current steperror|INT32|RO|0x00000000(0dec)|



EL70x7 

Version: 2.2.0 

233 

Configuration by means of the TwinCAT System Manager 

**Index A010 STM Diag data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|A010:0|STM Diag data<br>Ch.1|Maximum subindex|UINT8|RO|0x11 (17dec)|
|A010:01|Saturated|Driver stage operates with maximum dutycycle.|BOOLEAN|RO|0x00(0dec)|
|A010:02|Over temperature|Internal terminal temperature isgreater than 80 °C.|BOOLEAN|RO|0x00(0dec)|
|A010:03|Torque overload|Dutycycle output at 100 %|BOOLEAN|RO|0x00(0dec)|
|A010:04|Under voltage|Supplyvoltage less than 7 V|BOOLEAN|RO|0x00(0dec)|
|A010:05|Over voltage|Supply voltage 10 % higher than the nominal voltage<br>(see 0x8010:03)|BOOLEAN|RO|0x00 (0dec)|
|A010:06|Short circuit|Short circuit of motor coil|BOOLEAN|RO|0x00(0dec)|
|A010:08|No controlpower|Nopower supplyto driver stage|BOOLEAN|RO|0x00(0dec)|
|A010:09|Misc error|•<br>Initialization failed or<br>•<br>Internal terminal temperature is higher than 100 °C<br>(see 0xF80F:05).|BOOLEAN|RO|0x00 (0dec)|
|A010:0A|Configuration|CoE change has not yet been adopted into the current<br>configuration.|BOOLEAN|RO|0x00 (0dec)|
|A010:0B|Motor stall|A loss of stephas occurred.|BOOLEAN|RO|0x00(0dec)|
|A010:0C|Open load A||BOOLEAN|RO|0x00(0dec)|
|A010:0D|Open load B||BOOLEAN|RO|0x00(0dec)|
|A010:11|Actual operation<br>mode|permitted values:|BIT4|RO|0x00 (0dec)|
|||0: Automatic||||
|||1: Velocitydirect||||
|||2: Velocitycontroller||||
|||3: Position controller||||
|||4: Ext. Velocitymode||||
|||5: Ext. Position mode||||
|||6: Velocitysensorless||||



**Index A020 POS Diag data Ch.1** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|A020:0|POS Diag data<br>Ch.1|Maximum subindex|UINT8|RO|0x06 (6dec)|
|A020:01|Command<br>rejected|Travel command was rejected.|BOOLEAN|RO|0x00 (0dec)|
|A020:02|Command<br>aborted|Travel command was aborted.|BOOLEAN|RO|0x00 (0dec)|
|A020:03|Target overrun|Targetposition was overrun in the opposite direction.|BOOLEAN|RO|0x00(0dec)|
|A020:04|Target timeout|The target window was not reached within the in-target<br>timeout.|BOOLEAN|RO|0x00 (0dec)|
|A020:05|Position lag|The maximum followingerror was exceeded.|BOOLEAN|RO|0x00(0dec)|
|A020:06|EmergencyStop|An emergencystopwas triggered(automatic or manual).|BOOLEAN|RO|0x00(0dec)|



## **7.2.7 Vendor configuration data (device specific)** 

## **Index F80F STM Vendor data** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|F80F:0|STM Vendor data|Maximum subindex|UINT8|RO|0x05(5dec)|
|F80F:04|Warning<br>temperature|Temperature warning threshold<br>**Unit**: 1 °C|INT8|RW|0x50 (80dec)|
|F80F:05|Switch off<br>temperature|Switch-off temperature<br>**Unit**: 1 °C|INT8|RW|0x64 (100dec)|



234 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

**Index F010 Module list** 

## **7.2.8 Information / diagnostic data (device specific)** 

|**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F010:0<br>Module list<br>Maximum subindex<br>UINT8<br>RW<br>0x03(3dec)<br>F010:01<br>SubIndex 001<br>Encoder profile number<br>UINT32<br>RW<br>0x000001FF<br>(511dec)<br>F010:02<br>SubIndex 002<br>Stepper motor profile number<br>UINT32<br>RW<br>0x000002BF<br>(703dec)<br>F010:03<br>SubIndex 003<br>Positioning interface profile number<br>UINT32<br>RW<br>0x000002C0<br>(704dec)<br>~~——_———~~|**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F010:0<br>Module list<br>Maximum subindex<br>UINT8<br>RW<br>0x03(3dec)<br>F010:01<br>SubIndex 001<br>Encoder profile number<br>UINT32<br>RW<br>0x000001FF<br>(511dec)<br>F010:02<br>SubIndex 002<br>Stepper motor profile number<br>UINT32<br>RW<br>0x000002BF<br>(703dec)<br>F010:03<br>SubIndex 003<br>Positioning interface profile number<br>UINT32<br>RW<br>0x000002C0<br>(704dec)<br>~~——_———~~|
|---|---|
|**Index F081 Download revision**||
|**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F081:0<br>Download revision<br>Maximum subindex<br>UINT8<br>RO<br>0x01(1dec)<br>F081:01<br>Revision number<br>Revision number<br>UINT32<br>RW<br>0x00000000(0dec)<br>~~or~~||
|**Index F900 STM Info data**<br>**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>F900:0<br>STM Info data<br>Maximum subindex<br>UINT8<br>RO<br>0x06(6dec)<br>F900:01<br>Software version<br>(driver)<br>Software version of the output driver<br>STRING<br>RO<br>F900:02<br>Internal<br>temperature<br>Internal terminal temperature<br>**Unit**: 1 °C<br>INT8<br>RO<br>0x00 (0dec)<br>F900:04<br>Control voltage<br>Control voltage<br>**Unit**: 1 mV, 10 mV with field-oriented control<br>UINT16<br>RO<br>0x0000 (0dec)<br>F900:05<br>Motor supply<br>voltage<br>Motor supply voltage<br>**Unit**: 1 mV, 10 mV with field-oriented control<br>UINT16<br>RO<br>0x0000 (0dec)<br>F900:06<br>Cycle time<br>Current EtherCAT cycle time<br>**Unit**: 1µs<br>UINT16<br>RO<br>0x0000 (0dec)<br>~~===~~||
|**Index FB40 Memory interface**<br>**Index**<br>**(hex)**<br>**Name**<br>**Meaning**<br>**Data type**<br>**Flags**<br>**Default**<br>FB40:0<br>Memoryinterface<br>Maximum subindex<br>UINT8<br>RO<br>0x03(3dec)<br>FB40:01<br>Address<br>reserved<br>UINT32<br>RW<br>0x00000000(0dec)<br>FB40:02<br>Length<br>reserved<br>UINT16<br>RW<br>0x0000(0dec)<br>FB40:03<br>Data<br>reserved<br>OCTET-<br>STRING[8]<br>RW<br>{0}<br>**7.2.9**<br>**Standard objects**<br>**EtherCAT XML Device Description**<br>~~ee~~||
|The display matches that of the CoE objects from the EtherCATXML<br>Device Description. We||
|recommend downloading the latest XML file from the download area of the Beckhoff website and||
|installing it according to installation instructions.||



## **Standard objects (0x1000-0x1FFF)** 

**Index 1000 Device type Index Name Meaning Data type Flags Default (hex)** 1000:0 Device type Device type of the EtherCAT slave: the Lo-Word contains UINT32 RO 0x00001389 the CoE profile used (5001). The Hi-Word contains the (5001dec) ~~ee~~ module profile according to the modular device profile. 

EL70x7 

Version: 2.2.0 

235 

Configuration by means of the TwinCAT System Manager 

## **Index 1008 Device name** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1008:0|Device name|Device name of the EtherCAT slave|STRING|RO|EL7047|
|**Index 1009 Hardware version**||||||
|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|1009:0|Hardware version|Hardware version of the EtherCAT slave|STRING|RO||



## **Index 100ASoftware version** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|100A:0|Software version|Firmware version of the EtherCAT slave|STRING|RO|01|



## **Index 1018 Identity** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1018:0|Identity|Information for identifyingthe slave|UINT8|RO|0x04(4dec)|
|1018:01|Vendor ID|Vendor ID of the EtherCAT slave|UINT32|RO|0x00000002(2dec)|
|1018:02|Product code|Product code of the EtherCAT slave|UINT32|RO|0x1B873052<br>(461844562dec)|
|1018:03|Revision|Revision number of the EtherCAT slave; the low word (bit<br>0-15) indicates the special terminal number, the high<br>word(bit 16-31)refers to the device description|UINT32|RO|0x00000000 (0dec)|
|1018:04|Serial number|Serial number of the EtherCAT slave; the low byte (bit<br>0-7) of the low word contains the year of production, the<br>high byte (bit 8-15) of the low word contains the week of<br>production, the high word(bit 16-31)is 0|UINT32|RO|0x00000000 (0dec)|



## **Index 10F0 Backup parameter handling** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|10F0:0|Backup parameter<br>handling|Information for standardized loading and saving of<br>backupentries|UINT8|RO|0x01 (1dec)|
|10F0:01|Checksum|Checksum across all backup entries of the EtherCAT<br>slave|UINT32|RO|0x00000000 (0dec)|



## **Index 10F3 Diagnosis History** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|10F3:0|Diagnosis History|Maximum subindex|UINT8|RO|0x37(55dec)|
|10F3:01|Maximum<br>Messages|Maximum number of stored messages. A maximum of 50<br>messages can be stored.|UINT8|RO|0x00 (0dec)|
|10F3:02|Newest Message|Subindex of the latest message|UINT8|RO|0x00(0dec)|
|10F3:03|Newest<br>Acknowledged<br>Message|Subindex of the last confirmed message|UINT8|RW|0x00 (0dec)|
|10F3:04|New Messages<br>Available|Indicates that a new message is available.|BOOLEAN|RO|0x00 (0dec)|
|10F3:05|Flags|not used|UINT16|RW|0x0000(0dec)|
|10F3:06|Diagnosis<br>Message 001|Message 1|OCTET-<br>STRING[28]|RO|{0}|
|...|...|...|...|...|...|
|10F3:37|Diagnosis<br>Message 050|Message 50|OCTET-<br>STRING[28]|RO|{0}|



236 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 10F8 Actual Time Stamp** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|10F8:0|Actual Time<br>Stamp|Timestamp|UINT64|RO||
|**Index 1400 ENC RxPDO-Par Control compact**||||||
|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|1400:0|ENC RxPDO-Par<br>Control compact|PDO Parameter RxPDO 1|UINT8|RO|0x06 (6dec)|
|1400:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 1.|OCTET-<br>STRING[6]|RO|01 16 00 00 00 00|



## **Index 1401 ENC RxPDO-Par Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1401:0|ENC RxPDO-Par<br>Control|PDO Parameter RxPDO 2|UINT8|RO|0x06 (6dec)|
|1401:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 2.|OCTET-<br>STRING[6]|RO|00 16 00 00 00 00|



## **Index 1403 STM RxPDO-Par Position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1403:0|STM RxPDO-Par<br>Position|PDO Parameter RxPDO 4|UINT8|RO|0x06 (6dec)|
|1403:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 4.|OCTET-<br>STRING[6]|RO|04 16 05 16 06 16|



## **Index 1404 STM RxPDO-Par Velocity** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1404:0|STM RxPDO-Par<br>Velocity|PDO Parameter RxPDO 5|UINT8|RO|0x06 (6dec)|
|1404:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 5.|OCTET-<br>STRING[6]|RO|03 16 05 16 06 16|



## **Index 1405 POS RxPDO-Par Control compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1405:0|POS RxPDO-Par<br>Control compact|PDO Parameter RxPDO 6|UINT8|RO|0x06 (6dec)|
|1405:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 6.|OCTET-<br>STRING[6]|RO|03 16 04 16 06 16|



## **Index 1406 POS RxPDO-Par Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1406:0|POS RxPDO-Par<br>Control|PDO Parameter RxPDO 7|UINT8|RO|0x06 (6dec)|
|1406:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 7.|OCTET-<br>STRING[6]|RO|03 16 04 16 05 16|



EL70x7 

Version: 2.2.0 

237 

Configuration by means of the TwinCAT System Manager 

## **Index 1407 POS RxPDO-Par Control 2** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1407:0|POS RxPDO-Par<br>Control 2|PDO Parameter RxPDO 8|UINT8|RO|0x06 (6dec)|
|1407:06|Exclude RxPDOs|Specifies the RxPDOs (index of RxPDO mapping<br>objects) that must not be transferred together with<br>RxPDO 8|OCTET-<br>STRING[6]|RO|03 16 04 16 05 16|



## **Index 1600 ENC RxPDO-Map Control compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1600:0|ENC RxPDO-Map<br>Control compact|PDO Mapping RxPDO 1|UINT8|RO|0x06 (6dec)|
|1600:01|SubIndex 001|1. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x01(Enable latch C))|UINT32|RO|0x7000:01, 1|
|1600:02|SubIndex 002|2. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x02(Enable latch extern onpositive edge))|UINT32|RO|0x7000:02, 1|
|1600:03|SubIndex 003|3. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x03(Set counter))|UINT32|RO|0x7000:03, 1|
|1600:04|SubIndex 004|4. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry 0x04 (Enable latch extern on negative<br>edge))|UINT32|RO|0x7000:04, 1|
|1600:05|SubIndex 005|5. PDO Mappingentry (12 bits align)|UINT32|RO|0x0000:00, 12|
|1600:06|SubIndex 006|6. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x11(Set counter value))|UINT32|RO|0x7000:11, 16|



## **Index 1601 ENC RxPDO-Map Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1601:0|ENC RxPDO-Map<br>Control|PDO Mapping RxPDO 2|UINT8|RO|0x06 (6dec)|
|1601:01|SubIndex 001|1. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x01(Enable latch C))|UINT32|RO|0x7000:01, 1|
|1601:02|SubIndex 002|2. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x02(Enable latch extern onpositive edge))|UINT32|RO|0x7000:02, 1|
|1601:03|SubIndex 003|3. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x03(Set counter))|UINT32|RO|0x7000:03, 1|
|1601:04|SubIndex 004|4. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry 0x04 (Enable latch extern on negative<br>edge))|UINT32|RO|0x7000:04, 1|
|1601:05|SubIndex 005|5. PDO Mappingentry (12 bits align)|UINT32|RO|0x0000:00, 12|
|1601:06|SubIndex 006|6. PDO Mapping entry (object 0x7000 (ENC Outputs<br>Ch.1), entry0x11(Set counter value))|UINT32|RO|0x7000:11, 32|



## **Index 1602 STM RxPDO-Map Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1602:0|STM RxPDO-Map<br>Control|PDO Mapping RxPDO 3|UINT8|RO|0x06 (6dec)|
|1602:01|SubIndex 001|1. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x01(Enable))|UINT32|RO|0x7010:01, 1|
|1602:02|SubIndex 002|2. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x02(Reset))|UINT32|RO|0x7010:02, 1|
|1602:03|SubIndex 003|3. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x03(Reduce torque))|UINT32|RO|0x7010:03, 1|
|1602:04|SubIndex 004|4. PDO Mappingentry (8 bits align)|UINT32|RO|0x0000:00, 8|
|1602:05|SubIndex 005|5. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x0C(Digital output 1))|UINT32|RO|0x7010:0C, 1|
|1602:06|SubIndex 006|6. PDO Mappingentry (4 bits align)|UINT32|RO|0x0000:00, 4|



238 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1603 STM RxPDO-Map Position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1603:0|STM RxPDO-Map<br>Position|PDO Mapping RxPDO 4|UINT8|RO|0x01 (1dec)|
|1603:01|SubIndex 001|1. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x11(Position))|UINT32|RO|0x7010:11, 32|



## **Index 1604 STM RxPDO-Map Velocity** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1604:0|STM RxPDO-Map<br>Velocity|PDO Mapping RxPDO 5|UINT8|RO|0x01 (1dec)|
|1604:01|SubIndex 001|1. PDO Mapping entry (object 0x7010 (STM Outputs<br>Ch.1), entry0x21(Velocity))|UINT32|RO|0x7010:21, 16|



## **Index 1605 POS RxPDO-Map Control compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1605:0|POS RxPDO-Map<br>Control compact|PDO Mapping RxPDO 6|UINT8|RO|0x04 (4dec)|
|1605:01|SubIndex 001|1. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x01(Execute))|UINT32|RO|0x7020:01, 1|
|1605:02|SubIndex 002|2. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x02(Emergencystop))|UINT32|RO|0x7020:02, 1|
|1605:03|SubIndex 003|3. PDO Mappingentry (14 bits align)|UINT32|RO|0x0000:00, 14|
|1605:04|SubIndex 004|4. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x11(Targetposition))|UINT32|RO|0x7020:11, 32|



## **Index 1606 POS RxPDO-Map Control** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1606:0|POS RxPDO-Map<br>Control|PDO Mapping RxPDO 7|UINT8|RO|0x08 (8dec)|
|1606:01|SubIndex 001|1. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x01(Execute))|UINT32|RO|0x7020:01, 1|
|1606:02|SubIndex 002|2. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x02(Emergencystop))|UINT32|RO|0x7020:02, 1|
|1606:03|SubIndex 003|3. PDO Mappingentry (14 bits align)|UINT32|RO|0x0000:00, 14|
|1606:04|SubIndex 004|4. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x11(Targetposition))|UINT32|RO|0x7020:11, 32|
|1606:05|SubIndex 005|5. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x21(Velocity))|UINT32|RO|0x7020:21, 16|
|1606:06|SubIndex 006|6. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x22(Start type))|UINT32|RO|0x7020:22, 16|
|1606:07|SubIndex 007|7. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x23(Acceleration))|UINT32|RO|0x7020:23, 16|
|1606:08|SubIndex 008|8. PDO Mapping entry (object 0x7020 (POS Outputs<br>Ch.1), entry0x24(Deceleration))|UINT32|RO|0x7020:24, 16|



EL70x7 

Version: 2.2.0 

239 

Configuration by means of the TwinCAT System Manager 

## **Index 1607 POS RxPDO-Map Control 2** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1606:0|POS RxPDO-Map<br>Control|PDO Mapping RxPDO 7|UINT8|RO|0x08 (8dec)|
|1607:01|SubIndex 001|1. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00,2|
|1607:02|SubIndex 002|2. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x03(Enable auto start))|UINT32|RO|0x7021:03, 1|
|1607:03|SubIndex 003|3. PDO Mappingentry (13 bits align)|UINT32|RO|0x0000:00, 13|
|1607:04|SubIndex 004|4. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x11(Targetposition))|UINT32|RO|0x7021:11, 32|
|1607:05|SubIndex 005|5. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x21(Velocity))|UINT32|RO|0x7021:21, 16|
|1607:06|SubIndex 006|6. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x22(Start type))|UINT32|RO|0x7021:22, 16|
|1607:07|SubIndex 007|7. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x23(Acceleration))|UINT32|RO|0x7021:23, 16|
|1607:08|SubIndex 008|8. PDO Mapping entry (object 0x7021 (POS Outputs 2<br>Ch.1), entry0x24(Deceleration))|UINT32|RO|0x7021:24, 16|



## **Index 1800 ENC TxPDO-Par Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1800:0|ENC TxPDO-Par<br>Status compact|PDO parameter TxPDO 1|UINT8|RO|0x06 (6dec)|
|1800:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 1.|OCTET-<br>STRING[2]|RO|01 1A|



## **Index 1801 ENC TxPDO-Par Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1801:0|ENC TxPDO-Par<br>Status|PDO parameter TxPDO 2|UINT8|RO|0x06 (6dec)|
|1801:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 2.|OCTET-<br>STRING[2]|RO|00 1A|



## **Index 1806 POS TxPDO-Par Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1806:0|POS TxPDO-Par<br>Status compact|PDO parameter TxPDO 7|UINT8|RO|0x06 (6dec)|
|1806:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 7.|OCTET-<br>STRING[2]|RO|07 1A|



## **Index 1807 POS TxPDO-Par Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1807:0|POS TxPDO-Par<br>Status|PDO parameter TxPDO 8|UINT8|RO|0x06 (6dec)|
|1807:06|Exclude TxPDOs|Specifies the TxPDOs (index of TxPDO mapping objects)<br>that must not be transferred together with TxPDO 8.|OCTET-<br>STRING[2]|RO|06 1A|



240 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1A00 ENC TxPDO-Map Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A00:0|ENC TxPDO-Map<br>Status compact|PDO Mapping TxPDO 1|UINT8|RO|0x11 (17dec)|
|1A00:01|SubIndex 001|1. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x01(Latch C valid))|UINT32|RO|0x6000:01, 1|
|1A00:02|SubIndex 002|2. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x02(Latch extern valid))|UINT32|RO|0x6000:02, 1|
|1A00:03|SubIndex 003|3. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x03(Set counter done))|UINT32|RO|0x6000:03, 1|
|1A00:04|SubIndex 004|4. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x04(Counter underflow))|UINT32|RO|0x6000:04, 1|
|1A00:05|SubIndex 005|5. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x05(Counter overflow))|UINT32|RO|0x6000:05, 1|
|1A00:06|SubIndex 006|6. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00, 2|
|1A00:07|SubIndex 007|7. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x08(Extrapolation stall))|UINT32|RO|0x6000:08, 1|
|1A00:08|SubIndex 008|8. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x09(Status of input A))|UINT32|RO|0x6000:09, 1|
|1A00:09|SubIndex 009|9. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0A(Status of input B))|UINT32|RO|0x6000:0A, 1|
|1A00:0A|SubIndex 010|10. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0B(Status of input C))|UINT32|RO|0x6000:0B, 1|
|1A00:0B|SubIndex 011|11. PDO Mappingentry (1 bit align)|UINT32|RO|0x0000:00, 1|
|1A00:0C|SubIndex 012|12. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0D(Sync error))|UINT32|RO|0x6000:0D, 1|
|1A00:0D|SubIndex 013|13. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0E(Status of extern latch))|UINT32|RO|0x6000:0E, 1|
|1A00:0E|SubIndex 014|14. PDO Mappingentry (1 bit align)|UINT32|RO|0x0000:00, 1|
|1A00:0F|SubIndex 015|15. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x6000:10, 1|
|1A00:10|SubIndex 016|16. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x11(Counter value))|UINT32|RO|0x6000:11, 16|
|1A00:11|SubIndex 017|17. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x12(Latch value))|UINT32|RO|0x6000:12, 16|



EL70x7 

Version: 2.2.0 

241 

Configuration by means of the TwinCAT System Manager 

## **Index 1A01 ENC TxPDO-Map Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A01:0|ENC TxPDO-Map<br>Status|PDO Mapping TxPDO 2|UINT8|RO|0x11 (17dec)|
|1A01:01|SubIndex 001|1. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x01(Latch C valid))|UINT32|RO|0x6000:01, 1|
|1A01:02|SubIndex 002|2. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x02(Latch extern valid))|UINT32|RO|0x6000:02, 1|
|1A01:03|SubIndex 003|3. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x03(Set counter done))|UINT32|RO|0x6000:03, 1|
|1A01:04|SubIndex 004|4. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x04(Counter underflow))|UINT32|RO|0x6000:04, 1|
|1A01:05|SubIndex 005|5. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x05(Counter overflow))|UINT32|RO|0x6000:05, 1|
|1A01:06|SubIndex 006|6. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00, 2|
|1A01:07|SubIndex 007|7. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x08(Extrapolation stall))|UINT32|RO|0x6000:08, 1|
|1A01:08|SubIndex 008|8. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x09(Status of input A))|UINT32|RO|0x6000:09, 1|
|1A01:09|SubIndex 009|9. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0A(Status of input B))|UINT32|RO|0x6000:0A, 1|
|1A01:0A|SubIndex 010|10. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0B(Status of input C))|UINT32|RO|0x6000:0B, 1|
|1A01:0B|SubIndex 011|11. PDO Mappingentry (2 bits align)|UINT32|RO|0x0000:00, 1|
|1A01:0C|SubIndex 012|12. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0D(Status of extern latch))|UINT32|RO|0x6000:0D, 1|
|1A01:0D|SubIndex 013|13. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x0E(Sync error))|UINT32|RO|0x6000:0E, 1|
|1A01:0E|SubIndex 014|14. PDO Mappingentry (1 bits align)|UINT32|RO|0x0000:00, 1|
|1A01:0F|SubIndex 015|15. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x6000:10, 1|
|1A01:10|SubIndex 016|16. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x11(Counter value))|UINT32|RO|0x6000:11, 32|
|1A01:11|SubIndex 017|17. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x12(Latch value))|UINT32|RO|0x6000:12, 32|



## **Index 1A02 ENC TxPDO-Map Timest. compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A02:0|ENC TxPDO-Map<br>Timest. compact|PDO Mapping TxPDO 3|UINT8|RO|0x01 (1dec)|
|1A02:01|SubIndex 001|1. PDO Mapping entry (object 0x6000 (ENC Inputs<br>Ch.1), entry0x16(Timestamp))|UINT32|RO|0x6000:16, 32|



242 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1A03 STM TxPDO-Map Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A03:0|STM TxPDO-Map<br>Status|PDO Mapping TxPDO 4|UINT8|RO|0x0E (14dec)|
|1A03:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x01(Readyto enable))|UINT32|RO|0x6010:01, 1|
|1A03:02|SubIndex 002|2. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x02(Ready))|UINT32|RO|0x6010:02, 1|
|1A03:03|SubIndex 003|3. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x03(Warning))|UINT32|RO|0x6010:03, 1|
|1A03:04|SubIndex 004|4. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x04(Error))|UINT32|RO|0x6010:04, 1|
|1A03:05|SubIndex 005|5. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x05(Moving positive))|UINT32|RO|0x6010:05, 1|
|1A03:06|SubIndex 006|6. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x06(Movingnegative))|UINT32|RO|0x6010:06, 1|
|1A03:07|SubIndex 007|7. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x07(Torque reduced))|UINT32|RO|0x6010:07, 1|
|1A03:08|SubIndex 008|8. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x08(Motor stall))|UINT32|RO|0x6010:08, 1|
|1A03:09|SubIndex 009|9. PDO Mappingentry (3 bits align)|UINT32|RO|0x0000:00, 3|
|1A03:0A|SubIndex 010|10. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x0E(Sync error))|UINT32|RO|0x6010:0C, 1|
|1A03:0B|SubIndex 011|11. PDO Mappingentry (1 bits align)|UINT32|RO|0x6010:0D, 1|
|1A03:0C|SubIndex 012|12. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x6010:0E, 1|
|1A03:0D|SubIndex 013|13. PDO Mappingentry (1 bits align)|UINT32|RO|0x0000:00, 1|
|1A03:0E|SubIndex 014|14. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x10(TxPDO Toggle))|UINT32|RO|0x6010:10, 1|



**Index 1A04 STM TxPDO-Map Synchron info data** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A04:0|STM TxPDO-Map<br>Synchron info<br>data|PDO Mapping TxPDO 5|UINT8|RO|0x02 (2dec)|
|1A04:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x11(Info data 1))|UINT32|RO|0x6010:11, 16|
|1A04:02|SubIndex 002|2. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x12(Info data 2))|UINT32|RO|0x6010:12, 16|



## **Index 1A05 STM TxPDO-Map Motor load** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A05:0|STM TxPDO-Map<br>Motor load|PDO Mapping TxPDO 6|UINT8|RO|0x01 (1dec)|
|1A05:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x13(Motor load))|UINT32|RO|0x6010:13, 16|



EL70x7 

Version: 2.2.0 

243 

Configuration by means of the TwinCAT System Manager 

**Index 1A06 POS TxPDO-Map Status compact** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A06:0|POS TxPDO-Map<br>Status compact|PDO Mapping TxPDO 7|UINT8|RO|0x09 (9dec)|
|1A06:01|SubIndex 001|1. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x01(Busy))|UINT32|RO|0x6020:01, 1|
|1A06:02|SubIndex 002|2. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x02(In-Target))|UINT32|RO|0x6020:02, 1|
|1A06:03|SubIndex 003|3. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x03(Warning))|UINT32|RO|0x6020:03, 1|
|1A06:04|SubIndex 004|4. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x04(Error))|UINT32|RO|0x6020:04, 1|
|1A06:05|SubIndex 005|5. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x05(Calibrated))|UINT32|RO|0x6020:05, 1|
|1A06:06|SubIndex 006|6. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x06(Accelerate))|UINT32|RO|0x6020:06, 1|
|1A06:07|SubIndex 007|7. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x07(Decelerate))|UINT32|RO|0x6020:07, 1|
|1A06:08|SubIndex 008|8. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x08(Readyto execute))|UINT32|RO|0x6020:08, 1|
|1A06:09|SubIndex 009|9. PDO Mappingentry (8 bits align)|UINT32|RO|0x0000:00, 8|



**Index 1A07 POS TxPDO-Map Status** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A07:0|POS TxPDO-Map<br>Status|PDO Mapping TxPDO 8|UINT8|RO|0x0C (12dec)|
|1A07:01|SubIndex 001|1. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x01(Busy))|UINT32|RO|0x6020:01, 1|
|1A07:02|SubIndex 002|2. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x02(In-Target))|UINT32|RO|0x6020:02, 1|
|1A07:03|SubIndex 003|3. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x03(Warning))|UINT32|RO|0x6020:03, 1|
|1A07:04|SubIndex 004|4. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x04(Error))|UINT32|RO|0x6020:04, 1|
|1A07:05|SubIndex 005|5. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x05(Calibrated))|UINT32|RO|0x6020:05, 1|
|1A07:06|SubIndex 006|6. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x06(Accelerate))|UINT32|RO|0x6020:06, 1|
|1A07:07|SubIndex 007|7. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x07(Decelerate))|UINT32|RO|0x6020:07, 1|
|1A07:08|SubIndex 008|8. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x08(Readyto execute))|UINT32|RO|0x6020:08, 1|
|1A07:09|SubIndex 009|9. PDO Mappingentry (8 bits align)|UINT32|RO|0x0000:00, 8|
|1A07:0A|SubIndex 010|10. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x11(Actualposition))|UINT32|RO|0x6020:11, 32|
|1A07:0B|SubIndex 011|11. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x21(Actual velocity))|UINT32|RO|0x6020:21, 16|
|1A07:0C|SubIndex 012|12. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x22(Actual drive time))|UINT32|RO|0x6020:22, 32|



**Index 1A08 STM TxPDO-Map Internal position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A08:0|STM TxPDO-Map<br>Internalposition|PDO Mapping TxPDO 9|UINT8|RO|0x01 (1dec)|
|1A08:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x14(Internalposition))|UINT32|RO|0x6010:14, 32|



244 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1A09 STM TxPDO-Map External position** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A09:0|STM TxPDO-Map<br>Externalposition|PDO Mapping TxPDO 10|UINT8|RO|0x01 (1dec)|
|1A09:01|SubIndex 001|1. PDO Mapping entry (object 0x6010 (STM Inputs<br>Ch.1), entry0x15(Externalposition))|UINT32|RO|0x6010:15, 32|



## **Index 1A0A POS TxPDO-Map Actual position lag** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1A0A:0|POS TxPDO-Map<br>Actualposition lag|PDO Mapping TxPDO 11|UINT8|RO|0x01 (1dec)|
|1A0A:01|SubIndex 001|1. PDO Mapping entry (object 0x6020 (POS Inputs<br>Ch.1), entry0x23(Actualposition lag))|UINT32|RO|0x6020:23, 32|



## **Index 1C00 Sync manager type** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C00:0|Sync manager<br>type|Using the sync managers|UINT8|RO|0x04 (4dec)|
|1C00:01|SubIndex 001|Sync-Manager Type Channel 1: Mailbox Write|UINT8|RO|0x01(1dec)|
|1C00:02|SubIndex 002|Sync-Manager Type Channel 2: Mailbox Read|UINT8|RO|0x02(2dec)|
|1C00:03|SubIndex 003|Sync-Manager Type Channel 3: Process Data Write<br>(Outputs)|UINT8|RO|0x03 (3dec)|
|1C00:04|SubIndex 004|Sync-Manager Type Channel 4: Process Data Read<br>(Inputs)|UINT8|RO|0x04 (4dec)|



## **Index 1C12 RxPDO assign** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C12:0|RxPDO assign|PDO Assign Outputs|UINT8|RW|0x03(3dec)|
|1C12:01|Subindex 001|1. allocated RxPDO (contains the index of the associated<br>RxPDO mappingobject)|UINT16|RW|0x1600 (5632dec)|
|1C12:02|Subindex 002|2. allocated RxPDO (contains the index of the associated<br>RxPDO mappingobject)|UINT16|RW|0x1602 (5634dec)|
|1C12:03|Subindex 003|3. allocated RxPDO (contains the index of the associated<br>RxPDO mappingobject)|UINT16|RW|0x1604 (5636dec)|



## **Index 1C13 TxPDO assign** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C13:0|TxPDO assign|PDO Assign Inputs|UINT8|RW|0x02(2dec)|
|1C13:01|Subindex 001|1. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x1A00 (6656dec)|
|1C13:02|Subindex 002|2. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x1A03 (6659dec)|
|1C13:03|Subindex 003|3. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C13:04|Subindex 004|4. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C13:05|Subindex 005|5. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|
|1C13:06|Subindex 006|6. allocated TxPDO (contains the index of the associated<br>TxPDO mappingobject)|UINT16|RW|0x0000 (0dec)|



EL70x7 

Version: 2.2.0 

245 

Configuration by means of the TwinCAT System Manager 

**Index 1C32 SM output parameter** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|1C32:0|SM output<br>parameter|Synchronization parameters for the outputs|UINT8|RO|0x20 (32dec)|
|1C32:01|Sync mode|Current synchronization mode:<br>•<br>0: Free Run<br>•<br>1: Synchronous with SM 2 event<br>•<br>2: DC-Mode - Synchronous with SYNC0 Event<br>•<br>3: DC-Mode - Synchronous with SYNC1 event|UINT16|RW|0x0001 (1dec)|
|1C32:02|Cycle time|Cycle time (in ns):<br>•<br>Free Run: Cycle time of the local timer<br>•<br>Synchronous with SM 2 event: Master cycle time<br>•<br>DC-Mode: SYNC0/SYNC1 Cycle Time|UINT32|RW|0x000F4240<br>(1000000dec)|
|1C32:03|Shift time|Time between SYNC0 event and output of the outputs (in<br>ns, DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:04|Sync modes<br>supported|Supported synchronization modes:<br>•<br>Bit 0 = 1: free run is supported<br>•<br>Bit 1 = 1: Synchronous with SM 2 event is supported<br>•<br>Bit 2-3 = 01: DC mode is supported<br>•<br>Bit 4-5 = 10: Output shift with SYNC1 event (only<br>DC mode)<br>•<br>Bit 14 = 1: dynamic times (measurement through<br>writingof 0x1C32:08)|UINT16|RO|0x0C07 (3079dec)|
|1C32:05|Minimum cycle<br>time|Minimum cycle time (in ns)|UINT32|RO|0x0003D090<br>(250000dec)|
|1C32:06|Calc and copy<br>time|Minimum time between SYNC0 and SYNC1 event (in ns,<br>DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:07|Minimum delay<br>time|Min. time between SYNC1 event and output of the<br>outputs(in ns, DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:08|Command|•<br>0: Measurement of the local cycle time is stopped<br>•<br>1: Measurement of the local cycle time is started<br>The entries 0x1C32:03, 0x1C32:05, 0x1C32:06,<br>0x1C32:07, 0x1C32:09, 0x1C33:03, 0x1C33:06, and<br>0x1C33:09 are updated with the maximum measured<br>values.<br>For a subsequent measurement the measured values<br>are reset|UINT16|RW|0x0000 (0dec)|
|1C32:09|Maximum delay<br>time|Time between SYNC1 event and output of the outputs (in<br>ns, DC mode only)|UINT32|RO|0x00000000 (0dec)|
|1C32:0B|SM event missed<br>counter|Number of missed SM events in OPERATIONAL (DC<br>mode only)|UINT16|RO|0x0000 (0dec)|
|1C32:0C|Cycle exceeded<br>counter|Number of occasions the cycle time was exceeded in<br>OPERATIONAL (cycle was not completed in time or the<br>next cycle began too early)|UINT16|RO|0x0000 (0dec)|
|1C32:0D|Shift too short<br>counter|Number of occasions that the interval between SYNC0<br>and SYNC1 event was too short(DC mode only)|UINT16|RO|0x0000 (0dec)|
|1C32:14|Frame repeat time||UINT32|RW|0x00000000(0dec)|
|1C32:20|Sync error|The synchronization was not correct in the last cycle,<br>(outputs were output too late; DC mode only)|BOOLEAN|RO|0x00 (0dec)|



246 

Version: 2.2.0 

EL70x7 

Configuration by means of the TwinCAT System Manager 

## **Index 1C33 SM input parameter** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|---|
|1C33:0|SM input<br>parameter|Synchronization parameters for the inputs||UINT8|RO|0x20 (32dec)|
|1C33:01|Sync mode|Current synchronization mode:<br>•<br>0: Free Run<br>•<br>1: Synchronous with SM 3 event (no outputs<br>available)<br>•<br>2: DC - Synchronous with SYNC0 Event<br>•<br>3: DC - Synchronous with SYNC1 Event<br>•<br>34: Synchronous with SM 2 event (outputs<br>available)||UINT16|RW|0x0022 (34dec)|
|1C33:02|Cycle time|as|0x1C32:02 [<br>}<br>246<br>]|UINT32|RW|0x000F4240<br>(1000000dec)|
|1C33:03|Shift time|Time between SYNC0 event and reading of the inputs (in<br>ns, onlyDC mode)||UINT32|RO|0x00000000 (0dec)|
|1C33:04|Sync modes<br>supported|Supported synchronization modes:<br>•<br>Bit 0: free run is supported<br>•<br>Bit 1: synchronous with SM 2 event is supported<br>(outputs available)<br>•<br>Bit 1: synchronous with SM 3 event is supported (no<br>outputs available)<br>•<br>Bit 2-3 = 01: DC mode is supported<br>•<br>Bit 4-5 = 01: input shift through local event (outputs<br>available)<br>•<br>Bit 4-5 = 10: input shift with SYNC1 event (no<br>outputs available)<br>•<br>Bit 14 = 1: dynamic times (measurement through<br>writingof 0x1C32:08 [<br>}<br>246<br>]<br>or 0x1C33:08)||UINT16|RO|0x0C07 (3079dec)|
|1C33:05|Minimum cycle<br>time|as|0x1C32:05 [<br>}<br>246<br>]|UINT32|RO|0x0003D090<br>(250000dec)|
|1C33:06|Calc and copy<br>time|Time between reading of the inputs and availability of the<br>inputs for the master(in ns, onlyDC mode)||UINT32|RO|0x00000000 (0dec)|
|1C33:07|Minimum delay<br>time|Min. time between SYNC1 event and reading of the<br>inputs(in ns, onlyDC mode)||UINT32|RO|0x00000000 (0dec)|
|1C33:08|Command|as|0x1C32:08 [<br>}<br>246<br>]|UINT16|RW|0x0000 (0dec)|
|1C33:09|Maximum delay<br>time|Max. time between SYNC1 event and reading of the<br>inputs(in ns, onlyDC mode)||UINT32|RO|0x00000000 (0dec)|
|1C33:0B|SM event missed<br>counter|as|0x1C32:11 [<br>}<br>246<br>]|UINT16|RO|0x0000 (0dec)|
|1C33:0C|Cycle exceeded<br>counter|as|0x1C32:12 [<br>}<br>246<br>]|UINT16|RO|0x0000 (0dec)|
|1C33:0D|Shift too short<br>counter|as|0x1C32:13 [<br>}<br>246<br>]|UINT16|RO|0x0000 (0dec)|
|1C33:14|Frame repeat time|as|1C32:14|UINT32|RW|0x00000000(0dec)|
|1C33:20|Sync error|as|0x1C32:32 [<br>}<br>246<br>]|BOOLEAN|RO|0x00 (0dec)|



**Index F000 Modular device profile** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|F000:0|Modular device<br>profile|General information for the modular device profile|UINT8|RO|0x02 (2dec)|
|F000:01|Module index<br>distance|Index spacing of the objects of the individual channels|UINT16|RO|0x0010 (16dec)|
|F000:02|Maximum number<br>of modules|Number of channels|UINT16|RO|0x0003 (3dec)|



EL70x7 

Version: 2.2.0 

247 

Configuration by means of the TwinCAT System Manager 

## **Index F008 Code word** 

|**Index**<br>**(hex)**|**Name**|**Meaning**|**Data type**|**Flags**|**Default**|
|---|---|---|---|---|---|
|F008:0|Code word|see note! [<br>}<br>38<br>]|UINT32|RW|0x00000000 (0dec)|



248 

Version: 2.2.0 

EL70x7 

