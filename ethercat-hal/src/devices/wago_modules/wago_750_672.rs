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
    io::{digital_input::{DigitalInputDevice, DigitalInputInput}, stepper_velocity_wago_750_672::{C1Command, C1Flag, C2Flag, C3Flag, ControlByteC1, ControlByteC2, ControlByteC3, S3Flag, StatusByteS3}},
};

#[derive(Clone)]
pub struct Wago750_672 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_672RxPdo,
    pub txpdo: Wago750_672TxPdo,
    module: Option<Module>,
    pub state: InitState,
    pub initialized: bool,
}

// unfortunately we need to track and set bits over multiple cycles
// and wait for their acknowledgement so we have to use a state machine
// inside of our loop.
#[derive(Debug, Clone)]
pub enum InitState {
    Off,
    Enable,
    SetMode,
    StartPulseStart,
    StartPulseEnd,
    Running,
    ErrorQuit,
    ResetQuit,
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

/// Digital input port enumeration for the 6 inputs
#[derive(Debug, Clone, Copy)]
pub enum Wago750_672_InputPort {
    DI1,
    DI2,
    DI3,
    DI4,
    DI5,
    DI6,
}

// Get the Digital Inputs from the Status Byte S3
impl DigitalInputDevice<Wago750_672_InputPort> for Wago750_672 {
    fn get_input(&self, port: Wago750_672_InputPort) -> Result<DigitalInputInput, anyhow::Error> {
        let s3 = StatusByteS3::from_bits(self.txpdo.s3);
        Ok(DigitalInputInput {
            value: match port {
                Wago750_672_InputPort::DI1 => s3.has_flag(S3Flag::Input1),
                Wago750_672_InputPort::DI2 => s3.has_flag(S3Flag::Input2),
                Wago750_672_InputPort::DI3 => s3.has_flag(S3Flag::Input3),
                Wago750_672_InputPort::DI4 => s3.has_flag(S3Flag::Input4),
                Wago750_672_InputPort::DI5 => s3.has_flag(S3Flag::Input5),
                Wago750_672_InputPort::DI6 => s3.has_flag(S3Flag::Input6),
            },
        })
    }
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

        // Yes this statemachine is unfortunately needed in here to make sure
        // bits are correctly set and reset in the correct cycle.
        match self.state {
            InitState::Off => {
                // Do nothing
                self.initialized = false;
            }
            InitState::Enable => {
                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .bits();
                self.rxpdo.c1 = c1;

                // Switch state if ENABLE and STOP2_N is acknowledged
                if self.txpdo.s1 == c1 {
                    self.state = InitState::SetMode;
                }
            }
            InitState::SetMode => {
                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_command(C1Command::SpeedControl)
                    .bits();
                self.rxpdo.c1 = c1;

                // Switch state if SPEED MODE is acknowledged
                if self.txpdo.s1 == c1 {
                    self.initialized = true;
                    self.state = InitState::StartPulseStart;
                }
            }
            InitState::StartPulseStart => {
                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_flag(C1Flag::Start)
                    .with_command(C1Command::SpeedControl)
                    .bits();
                self.rxpdo.c1 = c1;

                // Switch state after StartPulse is acknowledged
                if self.txpdo.s1 == c1 {
                    self.state = InitState::StartPulseEnd;
                }
            }
            InitState::StartPulseEnd => {
                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_command(C1Command::SpeedControl)
                    .bits();
                self.rxpdo.c1 = c1;

                // Switch state after StartPulse is over
                if self.txpdo.s1 == c1 {
                    self.state = InitState::Running;
                }
            }
            InitState::Running => {
                let c2 = ControlByteC2::new().with_flag(C2Flag::ErrorQuit).bits();
                let c3 = ControlByteC3::new().with_flag(C3Flag::ResetQuit).bits();

                // Check for Error
                if self.txpdo.s2 & c2 != 0 {
                    self.state = InitState::ErrorQuit;
                } else {
                    self.rxpdo.c2 |= c2;
                }
                // Check for Reset
                if self.txpdo.s3 & c3 != 0 {
                    self.state = InitState::ResetQuit;
                } else {
                    self.rxpdo.c3 |= c3;
                }
            }
            InitState::ErrorQuit => {
                self.rxpdo.c2 |= ControlByteC2::new().with_flag(C2Flag::ErrorQuit).bits();
                tracing::error!("Stepper Controller Errored. Trying to reenable...");
                self.state = InitState::Enable;
            }
            InitState::ResetQuit => {
                self.rxpdo.c3 |= ControlByteC3::new().with_flag(C3Flag::ResetQuit).bits();
                tracing::error!("Stepper Controller Reset not Quit. Trying to reenable...");
                self.state = InitState::Enable;
            }
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
            state: InitState::Off,
            initialized: false,
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
