# Wago Stepper Controller - Developer Documentation

**Models:** 750-671 & 750-672  
**Last Updated:** February 2026  

## Table of Contents

1. [Overview](#overview)
2. [Hardware Quick Reference](#hardware-quick-reference)
3. [Architecture & Code Organization](#architecture--code-organization)
4. [Control & Status Bytes](#control--status-bytes)
5. [Implementation Details](#implementation-details)
6. [Prescalers & Calculations](#prescalers--calculations)
7. [Common Pitfalls & Debug Tips](#common-pitfalls--debug-tips)

---

## Overview

The Wago Stepper Controllers (750-671 and 750-672) drive stepper motors via EtherCAT. This documentation focuses on **velocity/speed control mode** implementation.

### Key Differences Between Models

| Feature         | 750-671                                    | 750-672                              |
| --------------- | ------------------------------------------ | ------------------------------------ |
| Max Voltage     | 50 VDC                                     | 70 VDC                               |
| Max Current     | 5 A                                        | 7.5 A                                |
| Speed Control   | Via Application Selector in config         | Via Control Byte C1 command          |
| Control Byte C1 | Different bit layout (M_Speed_Control bit) | Command field (0x18 = Speed Control) |

**For 750-671:** Must set `Application_Selector = 2` and `PWM_Period = 0` in WAGO I/O CHECK before using speed control.

**For 750-672:** Speed control is selected directly via command in Control Byte C1 (no config needed).

### Process Image

Both models use **12 bytes bidirectional** (96 bits each way):

```
Input (TxPDO - Controller → PLC):
 [0] S0 (status)
 [1] Reserved
 [2-3] Actual Velocity (i16)
 [4-5] Reserved
 [6-8] Actual Position (24-bit: L, M, H)
 [9] S3 (digital inputs, warnings)
 [10] S2 (busy, error, on_speed, etc.)
 [11] S1 (acknowledgments)

Output (RxPDO - PLC → Controller):
 [0] C0 (control)
 [1] Reserved
 [2-3] Target Velocity (i16, clamped to ±25000)
 [4-5] Acceleration (u16)
 [6-8] Position (24-bit: L, M, H)
 [9] C3 (position/reset control)
 [10] C2 (prescalers, error quit)
 [11] C1 (main control: enable, start, mode)
```

---

## Hardware Quick Reference

### Connections

| Pin | Name | Purpose |
|-----|------|---------|
| DI1+ | Digital Input 1 | Enable input (connect to 24V or control via software) |
| DI2+ | Digital Input 2 | Referencing input |
| DI- | Ground | Reference potential for DI1/DI2 |
| A1, A2 | Motor Winding A | Connect to motor phase A |
| B1, B2 | Motor Winding B | Connect to motor phase B |

### Status LEDs

- **LED H:** Shows error codes via blink patterns (see Wago manual section 3.3 for error codes)
- Other LEDs show Ready, Enable, Busy states (see manual section 2.1.2.3)

---

## Architecture & Code Organization

### File Structure

```
devices/
  wago_modules/
    wago_750_672.rs          # Device struct, EtherCAT interface
    
io/
  stepper_velocity_wago_750_672.rs  # High-level wrapper & byte helpers
```

### Key Components

#### 1. `Wago750_672` Struct (EtherCAT Device)

Located in `devices/wago_modules/wago_750_672.rs`

```rust
pub struct Wago750_672 {
    is_used: bool,
    tx_bit_offset: usize,  // Input data offset
    rx_bit_offset: usize,  // Output data offset
    pub rxpdo: Wago750_672RxPdo,  // Data we send
    pub txpdo: Wago750_672TxPdo,  // Data we receive
    module: Option<Module>,
    pub state: InitState,
    pub initialized: bool,
}
```

**Responsibilities:**
- Implements EtherCAT device traits
- Handles bit-level I/O with process image
- Runs state machine in `input()` method
- Thread-safe (used via `Arc<RwLock<>>`)

#### 2. `StepperVelocityWago750672` Wrapper

Located in `io/stepper_velocity_wago_750_672.rs`

```rust
pub struct StepperVelocityWago750672 {
    pub device: Arc<RwLock<Wago750_672>>,
    pub state: InitState,
    pub target_velocity: i16,
    pub target_acceleration: u16,
    pub enabled: bool,
    pub freq_range_sel: u8,
    pub acc_range_sel: u8,
}
```

**Responsibilities:**
- Provides high-level API (`set_velocity()`, `set_enabled()`)
- Manages prescaler settings
- Abstracts away state machine complexity

### Why Two Layers?

1. **`Wago750_672`:** Low-level EtherCAT device that handles cycle-by-cycle I/O
2. **`StepperVelocityWago750672`:** High-level wrapper for application logic

This separation allows:
- EtherCAT device to run in tight real-time loop
- Application code to use simple API without worrying about cycles/acknowledgments
- Easy testing of each layer independently

---

## Control & Status Bytes

### Design Pattern: Type-Safe Byte Construction

Instead of hardcoding hex values like `0x1B`, we use builder pattern with enums:

```rust
// ❌ BAD - magic numbers, error-prone
dev.rxpdo.c1 = 0x1B;

// ✅ GOOD - explicit, self-documenting
let c1 = ControlByteC1::new()
    .with_flag(C1Flag::Enable)
    .with_flag(C1Flag::Stop2N)
    .with_command(C1Command::SpeedControl)
    .bits();
dev.rxpdo.c1 = c1;
```

### Wago 750-672 Control Bytes

#### Control Byte C1 (Offset 11) - Main Control

```rust
pub struct ControlByteC1(u8);

pub enum C1Flag {
    Enable = 0b0000_0001,  // Bit 0: Must be 1 to enable
    Stop2N = 0b0000_0010,  // Bit 1: Must be 1 for normal operation
    Start  = 0b0000_0100,  // Bit 2: Rising edge triggers action
}

pub enum C1Command {
    Idle          = 0b0000_0000,  // Bits 3-7: Operating mode
    SinglePosition = 0b0000_1000,
    RunProgram    = 0b0001_0000,
    SpeedControl  = 0b0001_1000,  // ← We use this
    Reference     = 0b0010_0000,
    JogMode       = 0b0010_1000,
    Mailbox       = 0b0011_0000,
}
```

**Implementation:**

```rust
impl ControlByteC1 {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_flag(mut self, flag: C1Flag) -> Self {
        self.0 |= flag as u8;  // OR in the bit
        self
    }

    pub const fn with_command(mut self, cmd: C1Command) -> Self {
        self.0 = (self.0 & 0b0000_0111) | (cmd as u8);  // Mask lower 3 bits, set upper 5
        self
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}
```

**Key Points:**
- `const fn` allows compile-time evaluation
- Chainable builder pattern
- Command overwrites bits 3-7 while preserving 0-2
- Flags are additive (multiple can be set)

#### Control Byte C2 (Offset 10) - Prescalers & Error Handling

```rust
pub struct ControlByteC2(u8);

pub enum C2Flag {
    FreqRangeSelL = 0b0000_0001,  // Bit 0: Freq prescaler LSB
    FreqRangeSelH = 0b0000_0010,  // Bit 1: Freq prescaler MSB
    AccRangeSelL  = 0b0000_0100,  // Bit 2: Acc prescaler LSB
    AccRangeSelH  = 0b0000_1000,  // Bit 3: Acc prescaler MSB
    PreCalc       = 0b0100_0000,  // Bit 6: Pre-calculation
    ErrorQuit     = 0b1000_0000,  // Bit 7: Acknowledge error
}

impl ControlByteC2 {
    pub const fn with_freq_range(mut self, sel: u8) -> Self {
        self.0 = (self.0 & 0b1111_1100) | (sel & 0b0000_0011);
        self
    }

    pub const fn with_acc_range(mut self, sel: u8) -> Self {
        self.0 = (self.0 & 0b1111_0011) | ((sel & 0b0000_0011) << 2);
        self
    }
}
```

**Critical:** Prescalers can only be changed when controller is **disabled** (before Enable state).

#### Control Byte C3 (Offset 9) - Position & Reset

```rust
pub struct ControlByteC3(u8);

pub enum C3Flag {
    SetActualPos = 0b0000_0001,  // Bit 0: Set current position
    DirectionPos = 0b0000_0010,  // Bit 2: Force positive direction
    DirectionNeg = 0b0000_0100,  // Bit 3: Force negative direction
    ResetQuit    = 0b1000_0000,  // Bit 7: Acknowledge reset
}
```

### Status Bytes (Read-Only)

#### Status Byte S1 (Offset 11) - Acknowledgments

```rust
pub struct StatusByteS1(u8);

pub enum S1Flag {
    Ready      = 0b0000_0001,  // Bit 0: Acknowledges Enable
    Stop2NAck  = 0b0000_0010,  // Bit 1: Acknowledges Stop2_N
    StartAck   = 0b0000_0100,  // Bit 2: Acknowledges Start
}
// Bits 3-7: Mirrors C1 command when acknowledged

impl StatusByteS1 {
    pub const fn has_flag(self, flag: S1Flag) -> bool {
        (self.0 & flag as u8) != 0
    }
}
```

**Usage Pattern:**
```rust
// Wait for acknowledgment
if self.txpdo.s1 == c1 {
    // All bits acknowledged, proceed
}
```

#### Status Byte S2 (Offset 10) - Operation Status

```rust
pub struct StatusByteS2(u8);

pub enum S2Flag {
    OnTarget    = 0b0000_0001,  // Bit 0: Position reached (positioning mode)
    Busy        = 0b0000_0010,  // Bit 1: Executing command
    StandStill  = 0b0000_0100,  // Bit 2: Motor not moving
    OnSpeed     = 0b0000_1000,  // Bit 3: Target velocity reached
    Direction   = 0b0001_0000,  // Bit 4: Rotation direction (0=neg, 1=pos)
    ReferenceOk = 0b0010_0000,  // Bit 5: Referencing complete
    PreCalcAck  = 0b0100_0000,  // Bit 6: Pre-calc acknowledged
    Error       = 0b1000_0000,  // Bit 7: Error occurred (check LED H)
}
```

**Critical for velocity control:** Monitor `OnSpeed` bit to know when target velocity is reached.

#### Status Byte S3 (Offset 9) - Digital Inputs & Warnings

```rust
pub struct StatusByteS3(u8);

pub enum S3Flag {
    Input1  = 0b0000_0001,  // Bits 0-5: Digital input states
    Input2  = 0b0000_0010,
    Input3  = 0b0000_0100,
    Input4  = 0b0000_1000,
    Input5  = 0b0001_0000,
    Input6  = 0b0010_0000,
    Warning = 0b0100_0000,  // Bit 6: Warning condition
    Reset   = 0b1000_0000,  // Bit 7: Reset requested
}
```

---

## Implementation Details

### Critical Initialization Sequence

⚠️ **This sequence MUST be followed exactly or the controller won't work:**

1. **Set non-zero acceleration** (before enabling)
2. **Set Enable + Stop2_N** (wait for ack)
3. **Set operating mode** (0x18 for speed control, wait for ack)
4. **Issue Start pulse** (set to 1, wait for ack, set to 0, wait for ack clear)
5. **Running** (repeat step 4 every time velocity changes)

### State Machine (Why It Exists)

The state machine exists because **acknowledgments happen in different EtherCAT cycles**. You can't just set everything in one cycle.

```rust
#[derive(Debug, Clone)]
pub enum InitState {
    Off,               // Controller disabled
    Enable,            // Setting Enable + Stop2_N, waiting for ack
    SetMode,           // Setting speed control mode, waiting for ack
    StartPulseStart,   // Setting Start bit, waiting for ack
    StartPulseEnd,     // Clearing Start bit, waiting for ack clear
    Running,           // Normal operation
    ErrorQuit,         // Handling error
    ResetQuit,         // Handling reset
}
```

### State Machine Implementation

Located in `Wago750_672::input()` method (runs every EtherCAT cycle):

```rust
fn input(&mut self, input: &BitSlice) -> Result<(), anyhow::Error> {
    // 1. Read process image from controller
    let base = self.tx_bit_offset;
    let mut b = [0u8; 12];
    for i in 0..12 {
        b[i] = input[base + i * 8..base + (i + 1) * 8].load_le();
    }
    
    self.txpdo = Wago750_672TxPdo {
        s0: b[0],
        actual_velocity: i16::from_le_bytes([b[2], b[3]]),
        position_l: b[6],
        position_m: b[7],
        position_h: b[8],
        s3: b[9],
        s2: b[10],
        s1: b[11],
    };

    // 2. State machine logic
    match self.state {
        InitState::Off => {
            self.initialized = false;
            // Do nothing until external enable request
        }
        
        InitState::Enable => {
            // Build C1 byte with Enable and Stop2_N
            let c1 = ControlByteC1::new()
                .with_flag(C1Flag::Enable)
                .with_flag(C1Flag::Stop2N)
                .bits();
            self.rxpdo.c1 = c1;
            
            // Wait for acknowledgment
            if self.txpdo.s1 == c1 {
                self.state = InitState::SetMode;
            }
        }
        
        InitState::SetMode => {
            // Add speed control command
            let c1 = ControlByteC1::new()
                .with_flag(C1Flag::Enable)
                .with_flag(C1Flag::Stop2N)
                .with_command(C1Command::SpeedControl)
                .bits();
            self.rxpdo.c1 = c1;
            
            // Wait for mode acknowledgment
            if self.txpdo.s1 == c1 {
                self.initialized = true;
                self.state = InitState::StartPulseStart;
            }
        }
        
        InitState::StartPulseStart => {
            // Set Start bit
            let c1 = ControlByteC1::new()
                .with_flag(C1Flag::Enable)
                .with_flag(C1Flag::Stop2N)
                .with_flag(C1Flag::Start)  // ← Rising edge
                .with_command(C1Command::SpeedControl)
                .bits();
            self.rxpdo.c1 = c1;
            
            // Wait for Start acknowledgment
            if self.txpdo.s1 == c1 {
                self.state = InitState::StartPulseEnd;
            }
        }
        
        InitState::StartPulseEnd => {
            // Clear Start bit
            let c1 = ControlByteC1::new()
                .with_flag(C1Flag::Enable)
                .with_flag(C1Flag::Stop2N)
                .with_command(C1Command::SpeedControl)
                .bits();
            self.rxpdo.c1 = c1;
            
            // Wait for acknowledgment to clear
            if self.txpdo.s1 == c1 {
                self.state = InitState::Running;
            }
        }
        
        InitState::Running => {
            // Monitor for errors and resets
            let c2_error_mask = ControlByteC2::new().with_flag(C2Flag::ErrorQuit).bits();
            let c3_reset_mask = ControlByteC3::new().with_flag(C3Flag::ResetQuit).bits();
            
            if self.txpdo.s2 & c2_error_mask != 0 {
                self.state = InitState::ErrorQuit;
            } else if self.txpdo.s3 & c3_reset_mask != 0 {
                self.state = InitState::ResetQuit;
            }
        }
        
        InitState::ErrorQuit => {
            // Set Error_Quit bit to acknowledge
            self.rxpdo.c2 |= ControlByteC2::new().with_flag(C2Flag::ErrorQuit).bits();
            tracing::error!("Stepper Controller Errored. Trying to reenable...");
            self.state = InitState::Enable;  // Restart initialization
        }
        
        InitState::ResetQuit => {
            // Set Reset_Quit bit to acknowledge
            self.rxpdo.c3 |= ControlByteC3::new().with_flag(C3Flag::ResetQuit).bits();
            tracing::error!("Stepper Controller Reset. Trying to reenable...");
            self.state = InitState::Enable;  // Restart initialization
        }
    }
    
    Ok(())
}
```

**Key Implementation Notes:**

1. **State changes happen AFTER acknowledgment:** We wait for `self.txpdo.s1 == c1` before moving to next state
2. **Enable and Stop2_N stay high:** They're included in every C1 byte after Enable state
3. **Start pulse is a rising edge:** Must go 1 → 0, not just stay at 1
4. **Error recovery:** Automatically attempts to re-enable on error/reset

### High-Level API Implementation

Located in `StepperVelocityWago750672`:

```rust
pub fn set_enabled(&mut self, enabled: bool) {
    // Prevent redundant enables
    if self.enabled && enabled {
        return;
    }
    
    // Safety check: acceleration must be non-zero
    if self.get_target_acceleration() == 0 {
        return;
    }
    
    self.enabled = enabled;
    if enabled {
        self.change_init_state(InitState::Enable);
    } else {
        self.change_init_state(InitState::Off);
        self.write_control_byte(ControlByte::C1, 0b00000000);
    }
}

pub fn set_velocity(&mut self, velocity: i16) {
    self.target_velocity = velocity;
    
    let mut dev = block_on(self.device.write());
    
    // Clamp to valid range
    dev.rxpdo.velocity = velocity.clamp(-25000, 25000);
    
    // Trigger Start pulse if already initialized
    if dev.initialized {
        dev.state = InitState::StartPulseStart;
    }
}

pub fn set_acceleration(&mut self, acceleration: u16) {
    self.target_acceleration = acceleration;
    
    let mut dev = block_on(self.device.write());
    dev.rxpdo.acceleration = acceleration;
}
```

**Why the wrapper?**
- Application code doesn't need to know about state machine
- Handles common errors (zero acceleration check)
- Thread-safe access to device
- Clamping and validation in one place

### Changing Velocity at Runtime

⚠️ **Critical:** Must issue a **new Start pulse** for every velocity change:

```rust
// User calls this
stepper.set_velocity(15000);

// Internally, set_velocity() does:
dev.rxpdo.velocity = 15000;
if dev.initialized {
    dev.state = InitState::StartPulseStart;  // ← Triggers new pulse
}

// State machine then handles:
// StartPulseStart → StartPulseEnd → Running
// (Takes 2-3 EtherCAT cycles)
```

**Common mistake:** Setting velocity without triggering Start pulse - motor won't change speed.

---

## Prescalers & Calculations

Prescalers scale the velocity and acceleration values. They **can only be changed when disabled**.

### Frequency Prescaler (Bits 0-1 of C2)

Controls velocity scaling:

| Freq_Range_Sel | Prescaler | Max Frequency | Resolution |
|----------------|-----------|---------------|------------|
| 00 | 200 (default) | 10 kHz | Configurable |
| 01 | 80 | 25 kHz | 1 Hz |
| 10 | 20 | 100 kHz | 20 Hz |
| 11 | 4 | 500 kHz | 20 Hz |

**Formula:**
```
Output Frequency (fp) = Velocity × 80 / Freq_Prescaler [Hz]
```

**Example:**
```rust
// Freq_Range_Sel = 01 (prescaler = 80)
// Velocity = 10000
// Output: 10000 × 80 / 80 = 10000 Hz

stepper.set_freq_range_sel(1);  // Must be done BEFORE enable
stepper.set_velocity(10000);
```

### Acceleration Prescaler (Bits 2-3 of C2)

Controls acceleration scaling:

| Acc_Range_Sel | Multiplier | Time to Max Speed |
|---------------|------------|-------------------|
| 00 | 8 (default) | 7600 ms |
| 01 | 80 | 760 ms |
| 10 | 800 | 76 ms |
| 11 | 8000 | 7.6 ms |

**Formula:**
```
Acceleration (a) = Acceleration_Value × Acc_Multiplier / Freq_Prescaler [Hz/s]
```

**Example:**
```rust
// Acc_Range_Sel = 01 (multiplier = 80)
// Freq_Range_Sel = 01 (prescaler = 80)
// Acceleration_Value = 10000
// Output: 10000 × 80 / 80 = 10000 Hz/s

stepper.set_acc_range_sel(1);   // Must be done BEFORE enable
stepper.set_acceleration(10000);
```

### Setting Prescalers (Implementation)

```rust
pub fn set_freq_range_sel(&mut self, factor: u8) {
    // Safety checks
    if self.enabled || factor > 3 {
        return;  // Can't change while enabled
    }
    
    self.freq_range_sel = factor;
    
    let mut dev = block_on(self.device.write());
    let c2 = ControlByteC2::from_bits(dev.rxpdo.c2)
        .with_freq_range(factor)
        .bits();
    dev.rxpdo.c2 = c2;
}

pub fn set_acc_range_sel(&mut self, factor: u8) {
    if self.enabled || factor > 3 {
        return;
    }
    
    self.acc_range_sel = factor;
    
    let mut dev = block_on(self.device.write());
    let c2 = ControlByteC2::from_bits(dev.rxpdo.c2)
        .with_acc_range(factor)
        .bits();
    dev.rxpdo.c2 = c2;
}
```

**Usage:**
```rust
let mut stepper = StepperVelocityWago750672::new(device);

// Set prescalers FIRST (before enabling)
stepper.set_freq_range_sel(1);
stepper.set_acc_range_sel(1);
stepper.set_acceleration(10000);

// Then enable
stepper.set_enabled(true);

// Now can set velocity
stepper.set_velocity(15000);
```

---

## Common Pitfalls & Debug Tips

### 1. Controller Won't Enable

**Symptoms:**
- State stuck in `Enable`
- `initialized` never becomes true
- Motor doesn't move

**Debug checklist:**
```rust
// Check acceleration is non-zero
println!("Acceleration: {}", dev.rxpdo.acceleration);  // Must be > 0

// Check DI1 if hardware enable is configured
println!("DI1 state: {}", (dev.txpdo.s3 & 0x01) != 0);  // Should be 1

// Check status byte acknowledgment
println!("C1: {:#010b}", dev.rxpdo.c1);
println!("S1: {:#010b}", dev.txpdo.s1);  // Should match C1
```

**Common causes:**
```rust
// ❌ Forgot to set acceleration
stepper.set_acceleration(0);  // Won't enable!

// ✅ Always set non-zero acceleration
stepper.set_acceleration(10000);
stepper.set_enabled(true);
```

### 2. Velocity Won't Change

**Symptoms:**
	- Motor running but won't change speed
- `set_velocity()` called but no effect

**Debug:**
```rust
// Check initialized flag
println!("Initialized: {}", dev.initialized);

// Check state after velocity change
println!("State: {:?}", dev.state);  // Should become StartPulseStart

// Manually trigger Start pulse (bypass wrapper)
dev.state = InitState::StartPulseStart;
```

**Common causes:**
```rust
// ❌ Setting velocity before initialization
stepper.set_velocity(10000);  // Controller not ready yet

// ✅ Check initialized first
if stepper.device.read().initialized {
    stepper.set_velocity(10000);
}
```

### 3. Prescaler Not Working

**Symptoms:**
- Velocity calculation seems wrong
- Can't reach expected frequency

**Debug:**
```rust
// Check prescalers were set BEFORE enabling
println!("Freq prescaler: {}", stepper.freq_range_sel);
println!("Acc prescaler: {}", stepper.acc_range_sel);
println!("Enabled: {}", stepper.enabled);

// Check C2 byte
let dev = stepper.device.read();
println!("C2: {:#010b}", dev.rxpdo.c2);
// Bits 0-1: freq, Bits 2-3: acc
```

**Common causes:**
```rust
// ❌ Setting prescaler after enable
stepper.set_enabled(true);
stepper.set_freq_range_sel(1);  // Too late! Ignored.

// ✅ Set prescalers FIRST
stepper.set_freq_range_sel(1);
stepper.set_acc_range_sel(1);
stepper.set_enabled(true);
```

### 4. State Machine Stuck

**Symptoms:**
- State doesn't progress
- Acknowledgments not received

**Debug:**
```rust
// Add logging to state machine
match self.state {
    InitState::Enable => {
        println!("Enable state: C1={:#010b}, S1={:#010b}", self.rxpdo.c1, self.txpdo.s1);
        // ...
    }
}

// Check EtherCAT communication
// - Is cycle running?
// - Are other devices working?
// - Check bus topology
```

### 5. Error Bit Set

**Symptoms:**
- Error bit (S2 bit 7) is 1
- LED H blinking
- Controller stops

**Debug:**
```rust
// Check error state
let s2 = StatusByteS2::from_bits(dev.txpdo.s2);
if s2.has_flag(S2Flag::Error) {
    println!("ERROR DETECTED");
    
    // Count LED H blinks for error code
    // See Wago manual section 3.3
    
    // Check other status flags for clues
    println!("Busy: {}", s2.has_flag(S2Flag::Busy));
    println!("StandStill: {}", s2.has_flag(S2Flag::StandStill));
}
```

**Common error codes:**
- **2 blinks:** Overcurrent
- **3 blinks:** Position error
- **4 blinks:** Encoder fault
- See Wago manual for complete list
	
---

### Supporting Wago 750-671

The 750-671 uses different C1 bit layout:

1. **Create separate enum:**
```rust
// For 750-671
pub enum C1Flag671 {
    Enable         = 0b0000_0001,  // Bit 0
    Stop2N         = 0b0000_0010,  // Bit 1
    Start          = 0b0000_0100,  // Bit 2
    MSpeedControl  = 0b0000_1000,  // Bit 3 (instead of command field)
    MProgram       = 0b0001_0000,  // Bit 4
    MReference     = 0b0010_0000,  // Bit 5
    MJog           = 0b0100_0000,  // Bit 6
    MDriveByMBX    = 0b1000_0000,  // Bit 7
}
```

---

## Quick Reference Card

### Initialization Checklist
```rust
// 1. Create and configure
let stepper = StepperVelocityWago750672::new(device);

// 2. Set prescalers (optional, before enable)
stepper.set_freq_range_sel(1);   // 0-3
stepper.set_acc_range_sel(1);    // 0-3

// 3. Set non-zero acceleration (REQUIRED)
stepper.set_acceleration(10000);

// 4. Enable
stepper.set_enabled(true);

// 5. Set velocity (after initialized)
stepper.set_velocity(15000);     // ±25000 max
```

### Key Constants
```rust
const SPEED_CONTROL_CMD: u8 = 0x18;  // C1 command for 750-672
const MAX_VELOCITY: i16 = 25000;
const MIN_VELOCITY: i16 = -25000;
const DEFAULT_ACCELERATION: u16 = 10000;
```

### Critical Rules
1. ⚠️ **Never enable with zero acceleration**
2. ⚠️ **Always wait for acknowledgments**
3. ⚠️ **Issue Start pulse for every velocity change**
4. ⚠️ **Prescalers can only be changed when disabled**
5. ⚠️ **Keep Enable and Stop2_N high during operation**

### Common Bit Patterns
```rust
// Enable + Stop2_N
0b0000_0011

// Enable + Stop2_N + Speed Control
0b0001_1011

// Enable + Stop2_N + Speed Control + Start
0b0001_1111
```

### Debug Commands
```bash
# View byte values
println!("C1: {:#010b}", dev.rxpdo.c1);
println!("S1: {:#010b}", dev.txpdo.s1);

# Check specific flags
let s2 = StatusByteS2::from_bits(dev.txpdo.s2);
println!("Error: {}", s2.has_flag(S2Flag::Error));
println!("OnSpeed: {}", s2.has_flag(S2Flag::OnSpeed));

# Monitor state
println!("State: {:?}", dev.state);
println!("Initialized: {}", dev.initialized);
```

---

## Additional Resources

- **Wago Manual:** Section 2.1.2.3 (LEDs), 3.3 (Error Codes)
- **WAGO I/O CHECK:** Configuration software for 750-671 application selector
- **EtherCAT Specification:** www.ethercat.org
- **Code Location:** 
  - Device: `devices/wago_modules/wago_750_672.rs`
  - Wrapper: `io/stepper_velocity_wago_750_672.rs`

---

## Changelog

| Version | Date    | Changes                                     |
| ------- | ------- | ------------------------------------------- |
| 1.0     | 2025    | Initial implementation with state machine   |
| 2.0     | 2026-02 | Added comprehensive developer documentation |

---

**Need help?** Check the troubleshooting section or add debug logging using the template above.