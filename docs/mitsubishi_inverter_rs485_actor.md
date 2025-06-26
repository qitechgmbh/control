# Mitsubishi Inverter RS485 Actor Documentation

## Overview

The Mitsubishi Inverter RS485 Actor is a comprehensive driver implementation for controlling Mitsubishi inverters via RS485 communication using the Modbus RTU protocol and a SerialInterfaceDevice. This actor provides a state machine-based approach to handle communication with the inverter while maintaining proper timing and priority-based request handling.

## Architecture

### State Machine

The actor operates using a finite state machine with the following states:

```rust
pub enum State {
    Uninitialized,           // Initial state, waiting for serial interface initialization
    ReadyToSend,            // Ready to send new requests
    WaitingForRequestAccept, // Waiting for EL6021 to accept the transmitted request
    WaitingForResponse,     // Waiting for response from the inverter
    WaitingForReceiveAccept, // Waiting for EL6021 to accept the received response
}
```

After WaitingForReceiveAccept the state is set to ReadyToSend again

## System Registers

The driver provides access to various Mitsubishi system registers:

| Register                          | Address | Description                            |
| --------------------------------- | ------- | -------------------------------------- |
| InverterReset                     | 40002   | Reset/Restart the inverter             |
| ParameterClear                    | 40003   | Clear parameters                       |
| AllParameterClear                 | 40004   | Clear all parameters                   |
| ParamClearNonCommunication        | 40006   | Clear non-communication parameters     |
| AllParameterClearNonCommunication | 40007   | Clear all non-communication parameters |
| InverterStatusAndControl          | 40009   | Inverter status and control            |
| OperationModeAndSetting           | 40010   | Operation mode settings                |
| RunningFrequencyRAM               | 40014   | Running frequency (RAM)                |
| RunningFrequencyEEPROM            | 40015   | Running frequency (EEPROM)             |
| MotorFrequency                    | 40201   | Actual motor output frequency          |

## Control Requests

### Available Control Commands

```rust
pub enum MitsubishiControlRequests {
    None,
    ResetInverter,                    // Reset/Restart the inverter
    ClearAllParameters,               // Clear ALL parameters
    ClearNonCommunicationParameter,   // Clear a non-communication parameter
    ClearNonCommunicationParameters,  // Clear all non-communication parameters
    ReadInverterStatus,               // Read inverter status
    StopMotor,                        // Stop the motor
    StartForwardRotation,             // Start motor in forward direction
    StartReverseRotation,             // Start motor in reverse direction
    ReadRunningFrequency,             // Read current frequency (RAM)
    WriteRunningFrequency,            // Write frequency setpoint
    ReadMotorFrequency,               // Read actual output frequency
}
```

### Priority System

The actor implements a sophisticated priority system to ensure critical commands are executed first:

- **Highest Priority (65535)**: `StopMotor`, `ResetInverter`
- **High Priority (65534)**: `StartForwardRotation`, `StartReverseRotation`
- **Medium Priority (65533)**: `ReadMotorFrequency`
- **Lower Priority (65531)**: `ReadRunningFrequency`
- **Lowest Priority (65529)**: `ReadInverterStatus`

The priority system includes an "ignored times" mechanism that increases the effective priority of requests that have been waiting, ensuring all requests eventually get executed.

In essence it works like this:
Step 1:
| Request | Priority | Ignored | Effective Priority |
| ------- | -------- | ------- | ------------------ |
| A | 9 | 0 | 9 |
| B | 8 | 0 | 8 |

"A" gets executed and added again by some function

Step 2:
| Request | Priority | Ignored | Effective Priority |
| ------- | -------- | ------- | ------------------ |
| A | 9 | 0 | 9 |
| B | 8 | 1 | 9 |

"A" gets executed and added again by some function

Step 3:
| Request | Priority | Ignored | Effective Priority |
| ------- | -------- | ------- | ------------------ |
| A | 9 | 0 | 9 |
| B | 8 | 1 | 10 |

Now B gets executed even though it has a lower priority than A

## Request Types and Timeouts

Different request types have specific timeout requirements specfific to the cs80 mitsubishi inverters:

```rust
pub enum RequestType {
    OperationCommand,  // < 12ms timeout (monitoring, operation commands, frequency setting)
    ReadWrite,         // < 30ms timeout (parameter read/write, EEPROM frequency)
    ParamClear,        // < 5s timeout (parameter clearing operations)
    Reset,             // ~300ms timeout (reset operations)
}
```

## Response Handling

### Response Types

```rust
pub enum ResponseType {
    NoResponse,
    ReadFrequency,
    ReadMotorFrequency,
    WriteFrequency,
    InverterStatus,
    InverterControl,
}
```

### Inverter Status Structure

```rust
pub struct MitsubishiInverterStatus {
    pub running: bool,           // Motor is running
    pub forward_running: bool,   // Forward rotation active
    pub reverse_running: bool,   // Reverse rotation active
    pub su: bool,               // Speed up signal
    pub ol: bool,               // Overload signal
    pub no_function: bool,      // No function active
    pub fu: bool,               // Frequency up signal
    pub abc_: bool,             // ABC phase signal
    pub fault_occurence: bool,  // Fault has occurred
}
```

## Usage Examples

### Basic Initialization

```rust
let serial_interface = SerialInterface::new(/* parameters */);
let mut inverter = MitsubishiInverterRS485Actor::new(serial_interface);
```

### Setting Frequency

```rust
use uom::si::{f64::Frequency, frequency::hertz};

let target_frequency = Frequency::new::<hertz>(50.0);
inverter.set_frequency_target(target_frequency);
```

### Motor Control

```rust
// Start motor in forward direction
inverter.add_request(MitsubishiControlRequests::StartForwardRotation.into());

// Stop motor
inverter.add_request(MitsubishiControlRequests::StopMotor.into());

// Start motor in reverse direction
inverter.add_request(MitsubishiControlRequests::StartReverseRotation.into());
```

### Reading Status

```rust
// Read current motor frequency
inverter.add_request(MitsubishiControlRequests::ReadMotorFrequency.into());

// Read inverter status
inverter.add_request(MitsubishiControlRequests::ReadInverterStatus.into());
```

### State Machine Flow

1. **Uninitialized**: Wait for serial interface initialization
2. **ReadyToSend**: Send highest priority request from queue
3. **WaitingForRequestAccept**: Wait for EL6021 to accept transmission
4. **WaitingForResponse**: Wait for inverter response within timeout
5. **WaitingForReceiveAccept**: Wait for EL6021 to accept reception

## Error Handling

The driver includes comprehensive error handling:

- **Modbus Exception Codes**: Illegal function, illegal data address, illegal data value
- **Timeout Management**: Automatic timeout calculation based on baudrate and message size
- **Communication Errors**: Graceful handling of serial communication failures
- **State Recovery**: Automatic state recovery on communication errors

## Frequency Conversion

The driver handles frequency conversion between different units:

```rust
// Convert Hz to centihz (0.01 Hz units) for Mitsubishi protocol
pub fn convert_hz_float_to_word(&mut self, value: Frequency) -> u16 {
    let scaled = value.get::<centihertz>();
    scaled.round() as u16
}
```

## Thread Safety

The actor is designed to be used in async environments with proper Send bounds on all futures, ensuring thread safety in multi-threaded applications.

## Dependencies

- `bitvec`: For bit manipulation operations
- `uom`: For unit-of-measurement handling (frequency)
- `ethercat_hal`: For EtherCAT serial interface integration
- `anyhow`: For error handling
- Standard library components for collections, time, and async operations
