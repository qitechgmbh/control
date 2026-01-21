/*
 * Wago Stepper Controller 750-671
 * 24 VDC / 1.5 A
 */

use bitvec::field::BitField;

use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    helpers::counter_wrapper_u16_i128::CounterWrapperU16U128,
};

#[derive(Clone)]
pub struct Wago750_671 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_671RxPdo,
    pub txpdo: Wago750_671TxPdo,
    module: Option<Module>,
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
    pub c0: u8,            // C0
    pub velocity: i16,     // D0/D1
    pub acceleration: u16, // D2/D3
    pub c3: u8,            // C3
    pub c2: u8,            // C2
    pub c1: u8,            // C1
}

#[derive(Clone, Debug, Default)]
pub struct Wago750_671TxPdo {
    pub s0: u8,               // S0
    pub actual_velocity: i16, // D0/D1
    pub position_l: u8,       // D4
    pub position_m: u8,       // D5
    pub position_h: u8,       // D6
    pub s3: u8,               // S3
    pub s2: u8,               // S2
    pub s1: u8,               // S1
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

pub struct ControlByteC1;
impl ControlByteC1 {
    pub const ENABLE: u8 = 0x01; // Bit 0
    pub const STOP2_N: u8 = 0x02; // Bit 1
    pub const START: u8 = 0x04; // Bit 2
    pub const M_SPEED_CONTROL: u8 = 0x08; // Bit 3
    pub const M_PROGRAM: u8 = 0x10; // Bit 4
    pub const M_REFERENCE: u8 = 0x20; // Bit 5
    pub const M_JOG: u8 = 0x40; // Bit 6
    pub const M_DRIVE_BYMBX: u8 = 0x80; // Bit 7
}

pub struct ControlByteC2;
impl ControlByteC2 {
    pub const FREQ_RANGE_SEL_LSB: u8 = 0x01; // Bit 0
    pub const FREQ_RANGE_SEL_MSB: u8 = 0x02; // Bit 1
    pub const ACCELERATION_RANGE_SEL_LSB: u8 = 0x04; // Bit 2
    pub const ACCELERATION_RANGE_SEL_MSB: u8 = 0x08; // Bit 3
    // RESERVED                                     // Bit 4
    // RESERVED                                     // Bit 5
    pub const PRE_CALC: u8 = 0x40; // Bit 6
    pub const ERROR_QUIT: u8 = 0x80; // Bit 7
}

pub struct ControlByteC3;
impl ControlByteC3 {
    // RESERVED                             // Bit 0
    // RESERVED                             // Bit 1
    // RESERVED                             // Bit 2
    // RESERVED                             // Bit 3
    pub const LIMIT_SWITCH_POS: u8 = 0x10; // Bit 4
    pub const LIMIT_SWITCH_NEG: u8 = 0x20; // Bit 5
    pub const SETUP_SPEED_ACTIVE: u8 = 0x40; // Bit 6
    pub const RESET_QUIT: u8 = 0x80; // Bit 7
}

pub struct StatusByteS1;
impl StatusByteS1 {
    pub const READY: u8 = 0x01; // Bit 0
    pub const STOP_N_ACK: u8 = 0x02; // Bit 1
    pub const START_ACK: u8 = 0x04; // Bit 2
    pub const M_SPEED_CONTROL_ACK: u8 = 0x08; // Bit 3
    pub const M_PROGRAM_ACK: u8 = 0x10; // Bit 4
    pub const M_REFERENCE_ACK: u8 = 0x20; // Bit 5
    pub const M_JOG_ACK: u8 = 0x40; // Bit 6
    pub const M_DRIVE_BYMBX_ACK: u8 = 0x80; // Bit 7
}

pub struct StatusByteS2;
impl StatusByteS2 {
    // RESERVED                     // Bit 0
    const BUSY: u8 = 0x02; // Bit 1
    const STAND_STILL: u8 = 0x04; // Bit 2
    const ON_SPEED: u8 = 0x08; // Bit 3
    const DIRECTION: u8 = 0x10; // Bit 4
    // RESERVED                     // Bit 5
    const PRE_CALC_ACK: u8 = 0x40; // Bit 6
    const ERROR: u8 = 0x80; // Bit 7
}

pub struct StatusByteS3;
impl StatusByteS3 {
    pub const INPUT1: u8 = 0x01; // Bit 0
    // RESERVED                                 // Bit 1
    // RESERVED                                 // Bit 2
    // RESERVED                                 // Bit 3
    // RESERVED                                 // Bit 4
    // RESERVED                                 // Bit 5
    pub const SETUP_SPEED_ACTIVE_ACK: u8 = 0x40; // Bit 6
    pub const RESET: u8 = 0x80; // Bit 7
}

#[derive(Debug, Clone, Copy)]
enum SpeedControlState {
    Init,
    WaitReady,
    SelectMode,
    StartPulse,
    Running,
    ErrorAck,
}

impl Wago750_671 {
    /// Set speed setpoint and acceleration (Velocity Control process image).
    /// vel: i16 (sign determines direction)
    /// acc: u16 (must be > 0, acc==0 will trigger error)
    pub fn set_speed_setpoint(&mut self, vel: i16, acc: u16) {
        self.rxpdo.velocity = vel;
        self.rxpdo.acceleration = acc;
    }

    /// Apply control state for speed control application.
    ///
    /// Typical usage:
    /// - keep enable=true continuously after DI1 is high
    /// - keep speed_mode=true
    /// - pulse start_pulse=true for ONE cycle to accept setpoints / (re)start output
    pub fn apply_speed_control_state(&mut self, enable: bool, speed_mode: bool, start_pulse: bool) {
        let mut c1 = 0;

        if enable {
            c1 |= ControlByteC1::ENABLE | ControlByteC1::STOP2_N;
        }
        if speed_mode {
            c1 |= ControlByteC1::M_SPEED_CONTROL;
        }
        if start_pulse {
            c1 |= ControlByteC1::START;
        }

        self.rxpdo.c1 = c1;
    }

    pub fn write_control_bits(&mut self, c1: u8, c2: u8, c3: u8) {
        self.rxpdo.c1 = c1;
        self.rxpdo.c2 = c2;
        self.rxpdo.c3 = c3;
    }

    /// Error acknowledgement is edge-triggered (0->1). Pulse for one cycle.
    pub fn apply_error_quit(&mut self, pulse: bool) {
        if pulse {
            self.rxpdo.c2 |= ControlByteC2::ERROR_QUIT;
        } else {
            self.rxpdo.c2 &= !ControlByteC2::ERROR_QUIT;
        }
    }

    /// Reset acknowledgement (Reset_Quit) to clear S3.Reset after warm start / power-on reset.
    /// Pulse for one cycle when S3.Reset is set.
    pub fn apply_reset_quit(&mut self, pulse: bool) {
        if pulse {
            self.rxpdo.c3 |= ControlByteC3::RESET_QUIT;
        } else {
            self.rxpdo.c3 &= !ControlByteC3::RESET_QUIT;
        }
    }

    // Status helper functions
    pub fn s1(&self) -> u8 {
        self.txpdo.s1
    }
    pub fn s2(&self) -> u8 {
        self.txpdo.s2
    }
    pub fn s3(&self) -> u8 {
        self.txpdo.s3
    }

    pub fn ready(&self) -> bool {
        (self.s1() & StatusByteS1::READY) != 0
    }
    pub fn stop_n_ack(&self) -> bool {
        (self.s1() & StatusByteS1::STOP_N_ACK) != 0
    }
    pub fn start_ack(&self) -> bool {
        (self.s1() & StatusByteS1::START_ACK) != 0
    }
    pub fn speed_mode_ack(&self) -> bool {
        (self.s1() & StatusByteS1::M_SPEED_CONTROL_ACK) != 0
    }
    pub fn error_active(&self) -> bool {
        (self.s2() & StatusByteS2::ERROR) != 0
    }
    pub fn reset_active(&self) -> bool {
        (self.s3() & StatusByteS3::RESET) != 0
    }

    /// Actual velocity feedback (slave -> master), i16 little-endian.
    pub fn actual_velocity(&self) -> i16 {
        self.txpdo.actual_velocity
    }

    /// Actual position feedback is 23-bit + sign in other modes; in speed control it is still
    /// updated in the background. Here we just expose the raw 24-bit little-endian value.
    pub fn actual_position_raw24(&self) -> i32 {
        (self.txpdo.position_l as i32)
            | ((self.txpdo.position_m as i32) << 8)
            | ((self.txpdo.position_h as i32) << 16)
    }
}

impl EthercatDeviceProcessing for Wago750_671 {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl EthercatDevice for Wago750_671 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;

        let mut b = [0u8; 12];
        for i in 0..12 {
            b[i] = input[base + i * 8..base + (i + 1) * 8].load_le();
        }

        self.txpdo = Wago750_671TxPdo {
            s0: b[0],
            actual_velocity: i16::from_le_bytes([b[2], b[3]]),
            position_l: b[6],
            position_m: b[7],
            position_h: b[8],
            s3: b[9],
            s2: b[10],
            s1: b[11],
        };

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

        let b = [
            self.rxpdo.c0,
            0,
            self.rxpdo.velocity.to_le_bytes()[0],
            self.rxpdo.velocity.to_le_bytes()[1],
            self.rxpdo.acceleration.to_le_bytes()[0],
            self.rxpdo.acceleration.to_le_bytes()[1],
            0,
            0,
            0,
            self.rxpdo.c3,
            self.rxpdo.c2,
            self.rxpdo.c1,
        ];

        for i in 0..12 {
            output[base + i * 8..base + (i + 1) * 8].store_le(b[i]);
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
