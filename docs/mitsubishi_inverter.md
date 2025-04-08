# Communication Settings

## **117 N020 PU Communication Station**

**Range:** 00 to 31  
Specify the inverter station number.  
Enter the inverter station numbers when two or more inverters are connected to one personal computer.

## **118 N021 PU Communication Speed**

**Options:** 192, 48, 96, 192, 384, 576, 768, 1152  
Select the communication speed.  
The setting value × 100 equals the communication speed.  
For example, enter `192` to set the communication speed of **19200 bps**.

---

## **119 PU Communication Stop Bit Length / Data Length**

### **Stop Bit Length**

- `0` → **1 bit**
- `1` → **2 bits**

### **Data Length**

- `10` → **7 bits**
- `11` → **8 bits**

### **Parity Check**

- `0` → **Without parity check**
- `1` → **With parity check at odd numbers**
- `2` → **With parity check at even numbers**

---

## **120 N024 PU Connector Communication**

- `0` → **PU connector communication is disabled**

---

## **122 N026 PU Communication Check Time Interval**

**Range:** 0.1 to 999.8 s  
Set the interval for the communication check (signal loss detection) time.  
If a no-communication state persists for longer than the permissible time, the inverter output will be shut off.

- `9999` → **No communication check (signal loss detection)**

---

## **343 N080 Communication Error Count**

Displays the communication error count during **MODBUS RTU communication**. _(Read-only.)_

---

## **549 N000 Protocol Selection**

- `0` → **Mitsubishi inverter protocol (computer link)**
- `1` → **MODBUS RTU protocol**

# Communication Specifications

- The communication specifications are given below.

| **Item**                         | **Description**                                                               | **Related Parameter** |
| -------------------------------- | ----------------------------------------------------------------------------- | --------------------- |
| **Communication protocol**       | MODBUS RTU protocol                                                           | Pr.549                |
| **Conforming standard**          | EIA-485 (RS-485)                                                              | —                     |
| **Number of connectable units**  | 1: N (maximum 32 units), setting is 0 to 247 stations                         | Pr.117                |
| **Communication speed**          | Selected among 300/600/1200/2400/4800/9600/19200/38400/57600/76800/115200 bps | Pr.118                |
| **Control procedure**            | Asynchronous method                                                           | —                     |
| **Communication method**         | Half-duplex system                                                            | —                     |
| **Communication specifications** |                                                                               |                       |
| **Character system**             | Binary (fixed at 8 bits)                                                      | —                     |
| **Start bit**                    | 1 bit                                                                         | —                     |
| **Stop bit length**              | Select from the following three types:                                        | Pr.120                |
|                                  | - No parity check, stop bit length 2 bits                                     |                       |
|                                  | - Odd parity check, stop bit length 1 bit                                     |                       |
|                                  | - Even parity check, stop bit length 1 bit                                    |                       |
| **Parity check**                 | CRC code check                                                                | —                     |
| **Error check**                  | CRC code check                                                                | —                     |
| **Terminator**                   | Not available                                                                 | —                     |
| **Time delay setting**           | Not available                                                                 | —                     |

Message frames comprise four message fields shown above.  
A slave recognizes message data as one message when a **3.5 character long no-data time (T1: start/end)** is added before and after the data.

---

## **Message Frame Structure**

| **Start** | **Address** | **Function** | **Data**   | **CRC Check**              | **End** |
| --------- | ----------- | ------------ | ---------- | -------------------------- | ------- |
| **T1**    | 8 bits      | 8 bits       | n × 8 bits | **L** 8 bits, **H** 8 bits | **T1**  |

---

## **Message Field Descriptions**

### **Address Field**

- `"0 to 247"` can be set in the **8-bit** length field.
- Set `"0"` for **broadcast messages** (instructions to all addresses).
- `"1 to 247"` for **individual slave messages**.
- The **slave response** also contains the address set by the master.
- The value set in **Pr.117 PU communication station number** is the slave address.

### **Function Field**

- `"1 to 255"` can be set as the function code (8-bit).
- The **master sets the function** to be sent to the slave, and the slave performs the requested operation.
- An error response is generated if an unsupported function code is used.
- The normal response contains the **function code set by the master**.
- **Error response** contains `H80` + function code.

### **Data Field**

- The format **varies depending on the function code**.
- Example: Byte count, number of bytes, accessing content of holding registers.

### **CRC Check Field**

- Detects errors in the received message frame.
- Errors are detected in the CRC check, and the **2 bytes** of CRC data are appended to the message.
- **Order:** Lower byte first, then upper byte.
- **Process:**
  - The sender calculates and appends the CRC.
  - The receiver recalculates the CRC while receiving the message.
  - If the calculated CRC and received CRC **do not match**, an **error is returned**.

---

# Mitsubishi Inverter settings

You need to put the Mitsubishi Inverter into PU Mode
Protocol Selection: set Pr 549 to 1 (RTU)
set 551 to a non zero value (9999)
Operation mode selection: set Pr 79 to 1
PU communication station number: set Pr 117 to a value of 1-32 (this is also the slave address)
PU communication speed: set Pr 118 the same baudrate as the Beckhoff terminal. For example Terminal has 9600 then you need to set 96
PU communication stop bit: set Pr 119 according to Beckhoff terminal setting
PU communication parity check: set Pr 120 according to Beckhoff terminal setting

# el6021 settings needed for Modbus RTU

Continuous is needed for Modbus RTU
8000:04 Enable send fifo data continouus TRUE
8000:06 Enable half duplex
8000:11 Enable Baud rate 9600 Baud
8000:15 Data Frame 8N1

# TODO

- write functions that use tokio_modbus to generate the raw bytes for Modbus and to read the responses
- write a function that implements the wait times for Modbus RTU (3.5x the amount of bytes sent)
