/*
 * Wago Stepper Controller 750-672
 * 70 VDC / 7.5 A
 */

use anyhow::Ok;
use bitvec::field::BitField;

use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    helpers::counter_wrapper_u16_i128::CounterWrapperU16U128, io::stepper_velocity_wago_750_671::{ControlByteC1, ControlByteC2, ControlByteC3, StatusByteS1, StatusByteS2, StatusByteS3},
};

#[derive(Clone)]
pub struct Wago750_672 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_672RxPdo,
    pub txpdo: Wago750_672TxPdo,
    module: Option<Module>,

    pub enabled: bool,
}

/*
* Wago has two different kind of applications when it comes to
* running the Stepper Controller.
* - Positioning
* - Frequency/Speed Control
* !!!! IMPORTANT !!!!
* It seems like this is only true for the 750 671.
* The 750 672 has differnt commands inside of ControlByteC1
* where w can choose the Speed control mode - just like that
* we always want to use the latter one so the following
* process images are for that usecase.
*/

#[derive(Clone, Debug, Default)]
pub struct Wago750_672RxPdo {
    pub c0: u8,            // C0
    pub velocity: i16,     // D0/D1
    pub acceleration: u16, // D2/D3
    pub c3: u8,            // C3
    pub c2: u8,            // C2
    pub c1: u8,            // C1
}

#[derive(Clone, Debug, Default)]
pub struct Wago750_672TxPdo {
    pub s0: u8,               // S0
    pub actual_velocity: i16, // D0/D1
    pub position_l: u8,       // D4
    pub position_m: u8,       // D5
    pub position_h: u8,       // D6
    pub s3: u8,               // S3
    pub s2: u8,               // S2
    pub s1: u8,               // S1
}

impl EthercatDeviceUsed for Wago750_672 {
    fn is_used(&self) -> bool {
        self.is_used
    }
    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_672 {}

impl EthercatDynamicPDO for Wago750_672 {
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

impl Wago750_672 {


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
}

impl EthercatDeviceProcessing for Wago750_672 {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}



impl EthercatDevice for Wago750_672 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {

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

        if self.txpdo.s1 & 0b00011111 != 0b00011111 {
            // should be running now
            // tracing::warn!("Should be running now!");
            self.rxpdo.c1 = 0b00011011;
        }

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

impl NewEthercatDevice for Wago750_672 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rxpdo: Wago750_672RxPdo::default(),
            txpdo: Wago750_672TxPdo::default(),
            enabled: false,
        }
    }
}

impl std::fmt::Debug for Wago750_672 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_672")
    }
}

pub const WAGO_750_672_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_672_PRODUCT_ID: u32 = 108139752;
pub const WAGO_750_672_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_672_VENDOR_ID, WAGO_750_672_PRODUCT_ID);
