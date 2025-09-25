# Devices

We need to implement devices to communicate, configure, and identify them.

## Implementation Checklist

Here is a checklist of what we need to implement:

- Device Port
  - derive `#[derive(Debug, Clone, Copy)]`
- Device Struct `struct EL0000`
  - derive `#[derive(Device)]` derives `Device` trait
  - derive or implement `Debug` trait
  - field `txpdo` instance of `EL0000TxPdo` (only if the device has inputs)
  - field `rxpdo` instance of `EL0000RxPdo` (only if the device has outputs)
  - field `configuration` instance of `EL0000Configuration` (only if configurable)
  - implement `NewDevice` trait
  - implement `EthercatDeviceProcessing` trait
    - optional: override `input_post_process()` method for custom input data processing
    - optional: override `output_pre_process()` method for custom output data preparation
- TxPdo / RxPdo Struct `struct EL0000TxPdo` and/or `struct EL0000RxPdo`
  - derive `#[derive(Debug, Clone)]`
  - derive `#[derive(TxPdo)]` and/or `#[derive(RxPdo)]`
  - fields
      - tag with `#[pdo_object_index(0x1...)]` for the PDO object index
      - wrap PDO object in `Option`
- PDO Objects `struct EL0000PdoObject` or other PDO object structs
  - derive `#[derive(Debug, Clone, Default, PartialEq)]`
  - derive `#[derive(PdoObject)]`
  - tag with `#[pdo_object(bits = 16)]` for the size of the object
  - implement `TxPdoObject` and/or `RxPdoObject` traits
- Predefined PDO Assignment `enum EL0000PredefinedPdoAssignment` (if supported)
  - derive `#[derive(Debug, Clone)]`
  - implement `PredefinedPdoAssignment<EL0000TxPdo, EL0000RxPdo>`
- Configuration `struct EL0000Configuration` (if supported)
  - derive `#[derive(Debug, Clone)]`
  - implement `Configuration` trait
  - field `pdo_assignment` of type `EL0000PredefinedPdoAssignment`
  - fields for the configuration parameters
- [Device Identification Adresses](./identification.md)

These are helpful resources:
- Other implementations of similar devices
- Configuration Values: the official datasheet
- Configuration Implementation: [Docs](./coe.md)
- PDO Schema: The "Process Data" tab in the TwinCAT Software by Beckhoff
- PDO Implementation: [Docs](./pdo.md)
- Identification: Attach the device and read the identity values with QiTech Control in the "Setup > EtherCAT > Devices" tab

ESI Files are not needed but could be an alternative reference, though not a very readable one.

## EthercatDeviceProcessing

The `EthercatDeviceProcessing` trait provides hooks for custom processing of input and output data that happens between the EtherCAT data exchange and the device's IO layer. Every EtherCAT device must implement this trait, even if it doesn't need custom processing.

This trait provides two optional methods:

- `input_post_process()`: Called after the device has received input data from the EtherCAT bus, allowing transformation of the raw PDO data before it's exposed through the IO interfaces. 
- `output_pre_process()`: Called before data is sent to the EtherCAT bus, allowing preparation or transformation of data from the IO interfaces before it's written to the PDO objects.

For most devices, the default implementations (which do nothing) are sufficient. You only need to override these methods when the raw PDO data requires transformation before being used by the IO layer or vice versa.

Example implementation:

```rust
impl EthercatDeviceProcessing for EL0000 {
    // Optional: Only override when needed
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        // Process data after it's received from EtherCAT but before it's accessed via IO interfaces
        // Example: Transform raw sensor data into engineering units
        Ok(())
    }
    
    // Optional: Only override when needed
    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        // Process data before it's sent to EtherCAT
        // Example: Apply limits or transforms to output values
        Ok(())
    }
}
```

## Identity

We need to extract the possible EtherCAT identity values matching this driver implementation and provide them as constants.

Beckhoff has the official vendor ID of `0x2`. The product ID is a u32 which is unique for each device like `EL0000-....`, `EL2003-....`, `EL3001-....`, etc.

Hardware revisions or variants (like `EL2521-0024`) change the revision ID.

There can be multiple revisions for one product ID which should be supported by the driver so we annotate different revision constants with a suffix like `_A`, `_B`, `_0024_A`, etc.

For each revision, we need a tuple combining all three values. This makes pattern matching easier.
```rust
pub const EL0000_VENDOR_ID: u32 = 0x2;
pub const EL0000_PRODUCT_ID: u32 = 0x07d83052;
pub const EL0000_REVISION_A: u32 = 0x00110000;

pub const EL0000_IDENTITY_A: SubDeviceIdentityTuple =
        (EL0000_VENDOR_ID, EL0000_PRODUCT_ID, EL0000_REVISION_A);
```

## Creation

Add the device constructor to the `device_from_subdevice` function in the `server` crate.
This function is used to create the device at runtime.

```rust
pub fn (
    subdevice_identity_tuple: SubDeviceIdentityTuple,
) -> Result<Arc<RwLock<dyn Device>>, anyhow::Error> {
    match subdevice_identity_tuple {
        EK0000_IDENTITY_A => Ok(Arc::new(RwLock::new(EL0000::new()))),
        // ...
    }
}
```


# Device Implementation Guide

This section describes how to implement EtherCAT devices in the framework used by the QiTech Control project (via `ethercat-hal`, `server`, etc.). We include a running example of a simple digital output device (EL2004) and discuss how it fits into the larger architecture.

## Architecture & Context

The QiTech Control system is built as:

- `ethercat-hal` / `ethercat-hal-derive`: Hardware abstraction, PDO / CoE handling, device traits.  
- `server`: Glue code that reads device identities, instantiates the correct device types, performs CoE configuration, and runs cyclic process data.  
- The `docs` directory documents APIs and device patterns.  
  :contentReference[oaicite:0]{index=0}  

In this architecture:

1. During initialization, subdevice identity tuples (Vendor ID, Product ID, Revision) are read.
2. The `server` matches identity tuples to concrete `Device` types and constructs them (via `NewEthercatDevice`).
3. The device instance then handles PDO exchange, and optional CoE (configuration) writes.
4. Higher-level application code interacts with devices via traits like `DigitalOutputDevice`, `DigitalInputDevice`, etc.

---

## Example: Simple Digital Output Device (EL2004)

Below is a minimal example of a digital-output-only EtherCAT device. It shows how to map PDOs, implement traits, and define identity constants.

```rust
use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
use crate::pdo::{RxPdo, basic::BoolPdoObject};
use ethercat_hal_derive::{EthercatDevice, RxPdo};

/// EL2004 4-channel digital output device — 24 V DC, 0.5 A per channel
#[derive(EthercatDevice)]
pub struct EL2004 {
    /// The Rx PDO where output bits are written by the master
    pub rxpdo: EL2004RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL2004 {}

impl std::fmt::Debug for EL2004 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2004")
    }
}

impl NewEthercatDevice for EL2004 {
    fn new() -> Self {
        Self {
            rxpdo: EL2004RxPdo::default(),
            is_used: false,
        }
    }
}

impl DigitalOutputDevice<EL2004Port> for EL2004 {
    fn set_output(&mut self, port: EL2004Port, value: DigitalOutputOutput) {
        let expect_text = "All channels should be Some(_)";
        match port {
            EL2004Port::DO1 => {
                self.rxpdo.channel1.as_mut().expect(expect_text).value = value.into()
            }
            EL2004Port::DO2 => {
                self.rxpdo.channel2.as_mut().expect(expect_text).value = value.into()
            }
            EL2004Port::DO3 => {
                self.rxpdo.channel3.as_mut().expect(expect_text).value = value.into()
            }
            EL2004Port::DO4 => {
                self.rxpdo.channel4.as_mut().expect(expect_text).value = value.into()
            }
        }
    }

    fn get_output(&self, port: EL2004Port) -> DigitalOutputOutput {
        let expect_text = "All channels should be Some(_)";
        DigitalOutputOutput(match port {
            EL2004Port::DO1 => self.rxpdo.channel1.as_ref().expect(expect_text).value,
            EL2004Port::DO2 => self.rxpdo.channel2.as_ref().expect(expect_text).value,
            EL2004Port::DO3 => self.rxpdo.channel3.as_ref().expect(expect_text).value,
            EL2004Port::DO4 => self.rxpdo.channel4.as_ref().expect(expect_text).value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum EL2004Port {
    DO1,
    DO2,
    DO3,
    DO4,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL2004RxPdo {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<BoolPdoObject>,
    #[pdo_object_index(0x1602)]
    pub channel3: Option<BoolPdoObject>,
    #[pdo_object_index(0x1603)]
    pub channel4: Option<BoolPdoObject>,
}

impl Default for EL2004RxPdo {
    fn default() -> Self {
        Self {
            channel1: Some(BoolPdoObject::default()),
            channel2: Some(BoolPdoObject::default()),
            channel3: Some(BoolPdoObject::default()),
            channel4: Some(BoolPdoObject::default()),
        }
    }
}

/// Identity constants for EL2004
pub const EL2004_VENDOR_ID: u32 = 0x2;
pub const EL2004_PRODUCT_ID: u32 = 131346514;
pub const EL2004_REVISION_A: u32 = 1179648;
pub const EL2004_IDENTITY_A: SubDeviceIdentityTuple =
    (EL2004_VENDOR_ID, EL2004_PRODUCT_ID, EL2004_REVISION_A);
