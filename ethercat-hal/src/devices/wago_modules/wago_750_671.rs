/*
 * Wago Stepper Controller 750-671
 * 24 VDC / 1.5 A
 */

use bitvec::field::BitField;

use crate::{devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
}, helpers::counter_wrapper_u16_i128::CounterWrapperU16U128};

#[derive(Clone)]
pub struct Wago750_671 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_671RxPdo,
    pub txpdo: Wago750_671TxPdo,
    module: Option<Module>,
    pub counter_wrapper: CounterWrapperU16U128,
}

/*
* Wago has two different kind of applications when it comes to
* running the Stepper Controller.
* - Positioning
* - Frequency/Speed Control
* we always want to use the latter one so the following
* process images are for that usecase.
*/

#[derive(Clone, Debug, Default)]
pub struct Wago750_671RxPdo {
    pub b: [u8; 12],
}

#[derive(Clone, Debug, Default)]
pub struct Wago750_671TxPdo {
    pub b: [u8; 12],
}

impl EthercatDeviceUsed for Wago750_671 {
    fn is_used(&self) -> bool {
        self.is_used
    }
    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_671 {}

impl EthercatDynamicPDO for Wago750_671 {
    fn get_tx_offset(&self) -> usize {
        self.tx_bit_offset
    }
    fn get_rx_offset(&self) -> usize {
        self.rx_bit_offset
    }
    fn set_tx_offset(&mut self, offset: usize) {
        self.tx_bit_offset = offset
    }
    fn set_rx_offset(&mut self, offset: usize) {
        self.rx_bit_offset = offset
    }
}

/// [P]rocess [I]mage byte offsets
/// The process image is 12 bytes long.
/// missing bytes in the enum are reserved.
pub struct Wago750_671PI {
    input: InputPI,
    output: OutputPI,
}
pub struct OutputPI;
impl OutputPI {
     pub const C0: usize = 0; // Control byte C0
     pub const D0: usize = 2; // Velocity L
     pub const D1: usize = 3; // Velocity H
     pub const D2: usize = 4; // Acceleration L
     pub const D3: usize = 5; // Acceleration H
     pub const C3: usize = 9; // Control Byte C3
     pub const C2: usize = 10; // Control Byte C2
     pub const C1: usize = 11; // Control Byte C1
}

pub struct InputPI;
impl InputPI {
     pub const S0: usize = 0; // Status byte S0
     pub const D0: usize = 2; // Actual Velocity L
     pub const D1: usize = 3; // Actual Velocity H
     pub const D4: usize = 6; // Actual position L
     pub const D5: usize = 7; // Actual position M
     pub const D6: usize = 8; // Actual position H
     pub const S3: usize = 9; // Status byte S3
     pub const S2: usize = 10; // Status byte S2
     pub const S1: usize = 11; // Status byte S1
}

// C0 (mailbox enable is bit 5). In cyclic mode it must be 0.
const C0_MBX: u8 = 1 << 5;

// C1 bits (speed control application)
const C1_ENABLE: u8 = 1 << 0;
const C1_STOP2_N: u8 = 1 << 1;
const C1_START: u8 = 1 << 2;
const C1_M_SPEED_CONTROL: u8 = 1 << 3;

// C2 bits (speed control application)
const C2_ERROR_QUIT: u8 = 1 << 7;

// C3 bits (same meaning retained; Reset_Quit is bit 7)
const C3_RESET_QUIT: u8 = 1 << 7;

// S1 bits (acks)
const S1_READY: u8 = 1 << 0;
const S1_STOP_N_ACK: u8 = 1 << 1;
const S1_START_ACK: u8 = 1 << 2;
const S1_M_SPEED_CONTROL_ACK: u8 = 1 << 3;

// S2 bits
const S2_ERROR: u8 = 1 << 7;

// S3 bits
const S3_RESET: u8 = 1 << 7;

impl Wago750_671 {
    /// Must be called (or left default) to ensure mailbox is OFF (C0.5 = 0).
    /// If you ever enabled mailbox elsewhere, call this again.
    pub fn set_mailbox_enabled(&mut self, enabled: bool) {
        if enabled {
            self.rxpdo.b[self.pro] |= C0_MBX;
        } else {
            self.rxpdo.b[OutputPI::C0] &= !C0_MBX;
        }
    }

    /// Set speed setpoint and acceleration (Velocity Control process image).
    /// vel: i16 (sign determines direction)
    /// acc: u16 (must be > 0, acc==0 will trigger error)
    pub fn set_speed_setpoint(&mut self, vel: i16, acc: u16) {
        let v = vel.to_le_bytes();
        self.rxpdo.b[OutputPI::D0] = v[0]; // Velocity L
        self.rxpdo.b[OutputPI::D1] = v[1]; // Velocity H

        let a = acc.to_le_bytes();
        self.rxpdo.b[OutputPI::D2] = a[0]; // Acceleration L
        self.rxpdo.b[OutputPI::D3] = a[1]; // Acceleration H
    }

    /// Apply control state for speed control application.
    ///
    /// Typical usage:
    /// - keep enable=true continuously after DI1 is high
    /// - keep speed_mode=true
    /// - pulse start_pulse=true for ONE cycle to accept setpoints / (re)start output
    pub fn apply_speed_control_state(&mut self, enable: bool, speed_mode: bool, start_pulse: bool) {
        // Always keep mailbox disabled in cyclic operation for this mode
        self.set_mailbox_enabled(false);

        let mut c1: u8 = 0;

        if enable {
            c1 |= C1_ENABLE | C1_STOP2_N;
        }
        if speed_mode {
            c1 |= C1_M_SPEED_CONTROL;
        }
        if start_pulse {
            c1 |= C1_START;
        }

        self.rxpdo.b[OFF_C1] = c1;
    }

    /// Error acknowledgement is edge-triggered (0->1). Pulse for one cycle.
    pub fn apply_error_quit(&mut self, pulse: bool) {
        if pulse {
            self.rxpdo.b[OutputPI::C2] |= C2_ERROR_QUIT;
        } else {
            self.rxpdo.b[OutputPI::C2] &= !C2_ERROR_QUIT;
        }
    }

    /// Reset acknowledgement (Reset_Quit) to clear S3.Reset after warm start / power-on reset.
    /// Pulse for one cycle when S3.Reset is set.
    pub fn apply_reset_quit(&mut self, pulse: bool) {
        if pulse {
            self.rxpdo.b[OutputPI::C3] |= C3_RESET_QUIT;
        } else {
            self.rxpdo.b[OutputPI::C3] &= !C3_RESET_QUIT;
        }
    }

    // Status helper functions
    pub fn s1(&self) -> u8 {
        self.txpdo.b[InputPI::S1]
    }
    pub fn s2(&self) -> u8 {
        self.txpdo.b[InputPI::S2]
    }
    pub fn s3(&self) -> u8 {
        self.txpdo.b[InputPI::S3]
    }

    pub fn ready(&self) -> bool {
        (self.s1() & S1_READY) != 0
    }
    pub fn stop_n_ack(&self) -> bool {
        (self.s1() & S1_STOP_N_ACK) != 0
    }
    pub fn start_ack(&self) -> bool {
        (self.s1() & S1_START_ACK) != 0
    }
    pub fn speed_mode_ack(&self) -> bool {
        (self.s1() & S1_M_SPEED_CONTROL_ACK) != 0
    }
    pub fn error_active(&self) -> bool {
        (self.s2() & S2_ERROR) != 0
    }
    pub fn reset_active(&self) -> bool {
        (self.s3() & S3_RESET) != 0
    }

    /// Actual velocity feedback (slave -> master), i16 little-endian.
    pub fn actual_velocity(&self) -> i16 {
        i16::from_le_bytes([self.txpdo.b[InputPI::D0], self.txpdo.b[InputPI::D0]])
    }

    /// Actual position feedback is 23-bit + sign in other modes; in speed control it is still
    /// updated in the background. Here we just expose the raw 24-bit little-endian value.
    pub fn actual_position_raw24(&self) -> i32 {
        let b0 = self.txpdo.b[6] as u32;
        let b1 = self.txpdo.b[7] as u32;
        let b2 = self.txpdo.b[8] as u32;
        let u = b0 | (b1 << 8) | (b2 << 16);
        u as i32
    }
    pub fn position(&self) -> i128 {
        self.counter_wrapper.current()
    }
}

impl EthercatDeviceProcessing for Wago750_671 {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        let pos_l = self.txpdo.b[6];
        let pos_m = self.txpdo.b[7];
        let pos_h = self.txpdo.b[8];

        // WAGO position is 24-bit, but the Counter-Wrapper is 16-bit based
        // We use the lower 16 bits for the wrapper.
        let raw_u16 = u16::from_le_bytes([pos_l, pos_m]);

        let s2 = self.txpdo.b[10];
        let overflow = (s2 & (1 << 3)) != 0;
        let underflow = (s2 & (1 << 2)) != 0;

        self.counter_wrapper.update(raw_u16, underflow, overflow);
        Ok(())
    }

    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        if let Some(new_counter_u16) = self.counter_wrapper.pop_override() {
            // Write back the lower 16 bits
            self.rxpdo.b[6] = (new_counter_u16 & 0xFF) as u8;
            self.rxpdo.b[7] = (new_counter_u16 >> 8) as u8;

            // MSB must be written as well (sign extension or zero)
            self.rxpdo.b[8] = 0;

            // Tell the controller to accept the new position
            self.rxpdo.b[9] |= 1 << 6;
        }
        Ok(())
    }
}

impl EthercatDevice for Wago750_671 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;

        for byte_i in 0..12 {
            let bits = &input[base + byte_i * 8..base + (byte_i + 1) * 8];
            self.txpdo.b[byte_i] = bits.load_le::<u8>();
        }

        // Debug
        // println!("750-671 IN: {:02X?}", self.txpdo.b);

        Ok(())
    }

    fn input_len(&self) -> usize {
        12 * 8
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.rx_bit_offset;

        for byte_i in 0..12 {
            let bits = &mut output[base + byte_i * 8..base + (byte_i + 1) * 8];
            bits.store_le(self.rxpdo.b[byte_i]);
        }

        Ok(())
    }

    fn output_len(&self) -> usize {
        12 * 8
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn is_module(&self) -> bool {
        true
    }

    fn input_checked(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let expected = self.input_len();
        let actual = input.len();
        if actual != expected {
            return Err(anyhow::anyhow!(
                "[{}::Device::input_checked] Input length is {} ({} bytes) and must be {} bits ({} bytes)",
                module_path!(),
                actual,
                actual / 8,
                expected,
                expected / 8
            ));
        }
        Ok(())
    }

    fn output_checked(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let expected = self.output_len();
        let actual = output.len();
        if actual != expected {
            return Err(anyhow::anyhow!(
                "[{}::Device::output_checked] Output length is {} ({} bytes) and must be {} bits ({} bytes)",
                module_path!(),
                actual,
                actual / 8,
                expected,
                expected / 8
            ));
        }
        Ok(())
    }

    fn get_module(&self) -> Option<crate::devices::Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: crate::devices::Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module)
    }
}

impl NewEthercatDevice for Wago750_671 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rxpdo: Wago750_671RxPdo::default(),
            txpdo: Wago750_671TxPdo::default(),
            counter_wrapper: CounterWrapperU16U128::new(),
        }
    }
}

impl std::fmt::Debug for Wago750_671 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_671")
    }
}

pub const WAGO_750_671_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_671_PRODUCT_ID: u32 = 108074216;
pub const WAGO_750_671_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_671_VENDOR_ID, WAGO_750_671_PRODUCT_ID);
