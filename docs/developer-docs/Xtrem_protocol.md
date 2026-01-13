# XTREM Communication Protocol

**Version:** 3.007
**Device:** XTREM / XTREM-S Weighing Module
**Standard:** OIML R76:2006 / EN45501:2015 compliant

---

## Overview

The **XTREM communication protocol** allows a terminal or host controller to exchange data and commands with the XTREM weighing module.
The device transmits and receives ASCII-coded frames over a serial interface (RS232, RS485, Wi-Fi, or Ethernet, depending on configuration).

Since we are using the Ethernet interface we need to provide some kind of DHCP to assign an IP - Address to each XTREM Device.
They then may be addressed via a **broadcast** IP (e.g.: 192.168.4.255/24).
A **Unicast** communication is applied through the device ID inside the XTREM Frame.

---

## Frame Structure

Each message—both query and response—has the same format:

```
STX ID_O ID_D F D_ADDRESS D_L DATA LRC ETX
```

| Field         | Description                                       | Length (bytes) |
| :------------ | :------------------------------------------------ | :------------: |
| **STX**       | Start of message (ASCII `02h`)                    |        1       |
| **ID_O**      | Sender address (2 ASCII hex chars)                |        2       |
| **ID_D**      | Destination address (2 ASCII hex chars)           |        2       |
| **F**         | Function code (`R`, `W`, `E`, etc.)               |        1       |
| **D_ADDRESS** | Data address (register, 4 ASCII hex chars)        |        4       |
| **D_L**       | Data length (2 ASCII hex chars)                   |        2       |
| **DATA**      | Optional data field                               |    Variable    |
| **LRC**       | Longitudinal Redundancy Check (2 ASCII hex chars) |        2       |
| **ETX**       | End of message (ASCII `03h`)                      |        1       |

In the default configuration the Frame also requires a CRLF at the end.

**Example:**

```
STX ‘1’ ‘7’ ‘0’ ‘1’ ‘R’ ‘0’ ‘1’ ‘0’ ‘1’ ‘0’ ‘0’ LRC ETX CR LF
```

---

## Function Codes

| Function | ASCII | Hex | Description     |
| :------- | :---- | :---| :-------------- |
| **R**    | R     | 52h | Read request    |
| **r**    | r     | 72h | Read response   |
| **W**    | W     | 57h | Write request   |
| **w**    | w     | 77h | Write response  |
| **E**    | E     | 45h | Execute command |
| **e**    | e     | 65h | Execute response|

### Write response codes

* `'0'` – OK
* `'1'` – Write denied (sealed)
* `'2'` – Read-only register
* `'3'` – Invalid value / out of range

### Execute response codes

* `'0'` – Executed successfully
* `'1'` – Blocked by seal switch

---

## LRC Calculation

The **Longitudinal Redundancy Check** is a one-byte XOR of all message bytes, excluding:

* `STX`
* `ETX`
* The LRC itself

Example C function:

```c
unsigned char LRC(unsigned char* s) {
    unsigned char lrc = 0;
    while (*s) {
        lrc ^= *(s++);
    }
    return lrc;
}
```

Example Rust function:
```rust
pub fn compute_lrc(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

```

---

## Key Data Registers

| Address (hex) | Description                | Type | Notes                                 |
| :------------ | :------------------------- | :--: | :------------------------------------ |
| **0000h**     | Serial number              |   R  | Factory-set                           |
| **0001h**     | Device ID                  |  RW  | Network address                       |
| **0007h**     | Hardware version           |   R  | Read-only                             |
| **0008h**     | Software version           |   R  | Read-only                             |
| **0009h**     | Sealing switch status      |   R  | `0=UNLOCK`, `1=LOCK`                  |
| **0010h**     | Baud rate                  |  RW  | `0–4` → 9600–115200 baud              |
| **0013h**     | Stream mode output rate    |  RW  | Time in ms                            |
| **0100h**     | Device state               |   R  | Status bits (power, Wi-Fi, ADC, etc.) |
| **0101h**     | Weight value               |   R  | Gross weight                          |
| **0102h**     | Tare value                 |  RE  | Read/Execute                          |
| **0103h**     | Net weight                 |   R  | Gross − Tare                          |
| **0110h**     | ADC counts (instant)       |   R  | Raw ADC data                          |
| **0111h**     | ADC counts (filtered)      |   R  | Post-filter data                      |
| **1010h**     | Stop stream mode           |   E  | Execute                               |
| **1011h**     | Start stream mode (weight) |   E  | Continuous output                     |
| **9999h**     | Device reset               |   E  | Software reset                        |
| **EEEEh**     | Factory reset              |   E  | Restores defaults                     |

---

## Post-Factory Reset Configuration

After performing a factory reset, several registers must be configured to ensure proper operation of the device.

Note: The unit may remain set to 'g' after a factory reset, but the values received may be in 'kg' depending on the device configuration.

The following registers should be reviewed and set as required:
1. **Address: 0022 | Max (Max1) (scale maximum capacity)**
    The default value for this register may vary between devices (e.g., 60000 or 6000). Set this value to 6000 to ensure correct operation.
2. **Address: 0023 | e (e1) (scale interval)**
3. **Address: 0026 | Decimal position**
