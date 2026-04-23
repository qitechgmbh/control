/*
 * Wago Stepper Controller 750-672
 * 70 VDC / 7.5 A
 */

use std::collections::VecDeque;

use anyhow::Ok;
use bitvec::field::BitField;

use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    io::{
        digital_input::{DigitalInputDevice, DigitalInputInput},
        stepper_velocity_wago_750_672::{
            C1Command, C1Flag, C2Flag, C3Flag, ControlByteC1, S1Flag, S2Flag, S3Flag, StatusByteS1,
            StatusByteS2, StatusByteS3,
        },
    },
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
    log_counter: u32,
    enable_stuck_counter: u32,
    mailbox_toggle: bool,
    mailbox_queue: VecDeque<[u8; 6]>,
    mailbox_pending: Option<[u8; 6]>,
    mailbox_in_flight: bool,
    mailbox_active: bool,
    config_defaults_queued: bool,
    pub desired_command: C1Command,
    pub start_requested: bool,
    pub last_tms_enabling_block: Option<u32>,
    pub last_diag_return_code: Option<u8>,
}

impl Wago750_672 {
    pub(crate) fn queue_mailbox_command(&mut self, opcode: u8, p1: u8, p2: u8, p3: u8, p4: u8) {
        self.mailbox_toggle = !self.mailbox_toggle;
        let control_mbx = if self.mailbox_toggle { 0x80 } else { 0x00 };
        self.mailbox_queue
            .push_back([opcode, control_mbx, p1, p2, p3, p4]);
        self.mailbox_active = true;
    }

    fn queue_config_write_u8(&mut self, address: u16, value: u8) {
        let [addr_l, addr_h] = address.to_le_bytes();
        self.queue_mailbox_command(0x50, addr_l, addr_h, 1, 0);
        self.queue_mailbox_command(0x51, value, 0, 0, 0);
    }

    pub fn queue_velocity_control_pointer_defaults(&mut self) {
        if self.config_defaults_queued {
            return;
        }

        self.config_defaults_queued = true;

        // 750-672 manual, Bit Field for I/O Driver:
        // map velocity-control linkable bits to the cyclic KBUS control bytes.
        self.queue_config_write_u8(176, 0x40); // Ptr_Enable       <- KBUS_CTRL1_0
        self.queue_config_write_u8(177, 0x41); // Ptr_Stop2_N      <- KBUS_CTRL1_1
        self.queue_config_write_u8(178, 0x42); // Ptr_Start        <- KBUS_CTRL1_2
        self.queue_config_write_u8(179, 0x43); // Ptr_Command[1]   <- KBUS_CTRL1_3
        self.queue_config_write_u8(180, 0x44); // Ptr_Command[2]   <- KBUS_CTRL1_4
        self.queue_config_write_u8(181, 0x45); // Ptr_Command[3]   <- KBUS_CTRL1_5
        self.queue_config_write_u8(182, 0x46); // Ptr_Command[4]   <- KBUS_CTRL1_6
        self.queue_config_write_u8(183, 0x47); // Ptr_Command[5]   <- KBUS_CTRL1_7
        self.queue_config_write_u8(184, 0x01); // Ptr_Enable_Drive <- ONE
        self.queue_config_write_u8(185, 0x57); // Ptr_Reset_Quit   <- KBUS_CTRL3_7
        self.queue_config_write_u8(191, 0x4F); // Ptr_Error_Quit   <- KBUS_CTRL2_7
        self.queue_config_write_u8(194, 0x01); // Ptr_Stop1_N      <- ONE
    }

    fn mailbox_busy(&self) -> bool {
        self.mailbox_active
            || self.mailbox_pending.is_some()
            || self.mailbox_in_flight
            || !self.mailbox_queue.is_empty()
    }

    fn process_mailbox_response(&mut self, b: &[u8; 12]) {
        if !self.mailbox_in_flight || (b[0] & 0x20) == 0 {
            return;
        }

        let opcode = self.mailbox_pending.map(|pending| pending[0]).unwrap_or(0);
        let expected_toggle = self
            .mailbox_pending
            .map(|pending| (pending[1] & 0x80) != 0)
            .unwrap_or(false);
        let status_toggle = (b[3] & 0x80) != 0;
        if status_toggle != expected_toggle {
            return;
        }

        let return_code = b[3] & 0x7F;
        let value = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
        self.last_diag_return_code = Some(return_code);
        if opcode == 0x4C {
            self.last_tms_enabling_block = Some(value);
        }
        self.mailbox_pending = None;
        self.mailbox_in_flight = false;
        if self.mailbox_queue.is_empty() {
            self.mailbox_active = false;
            self.rxpdo.control_byte &= !0x20;
        }

        tracing::warn!(
            "750-672 mailbox response | opcode=0x{:02X} return_code=0x{:02X} value=0x{:08X} arm_enabling={} overcurrent={} error_ack_present={} error={} reset_not_completed={} incomplete_tms_params={} faulty_intermediate_voltage={} faulty_24v={}",
            opcode,
            return_code,
            value,
            (value & (1 << 0)) != 0,
            (value & (1 << 1)) != 0,
            (value & (1 << 2)) != 0,
            (value & (1 << 3)) != 0,
            (value & (1 << 4)) != 0,
            (value & (1 << 5)) != 0,
            (value & (1 << 6)) != 0,
            (value & (1 << 7)) != 0,
        );
    }

    fn dispatch_next_mailbox_command(&mut self) {
        if self.mailbox_in_flight || self.mailbox_pending.is_some() {
            return;
        }

        if let Some(bytes) = self.mailbox_queue.pop_front() {
            self.mailbox_pending = Some(bytes);
            self.mailbox_in_flight = true;
            self.mailbox_active = true;
            self.rxpdo.control_byte |= 0x20; // C0.5 = mailbox process image
            tracing::warn!(
                "750-672 mailbox dispatch | opcode=0x{:02X} mb1=0x{:02X} mb2=0x{:02X} mb3=0x{:02X} mb4=0x{:02X} mb5=0x{:02X}",
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
            );
        }
    }
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
    pub control_byte: u8,  // C0
    pub velocity: i16,     // D0/D1
    pub acceleration: u16, // D2/D3
    pub control_byte3: u8, // C3
    pub control_byte2: u8, // C2
    pub control_byte1: u8, // C1
}

#[derive(Clone, Debug, Default)]
pub struct Wago750_672TxPdo {
    pub status_byte0: u8,     // S0
    pub actual_velocity: i16, // D0/D1
    pub position_l: u8,       // D4
    pub position_m: u8,       // D5
    pub position_h: u8,       // D6
    pub status_byte3: u8,     // S3
    pub status_byte2: u8,     // S2
    pub status_byte1: u8,     // S1
}

/// Digital input port enumeration for the 6 inputs
#[derive(Debug, Clone, Copy)]
pub enum Wago750_672InputPort {
    DI1,
    DI2,
    DI3,
    DI4,
    DI5,
    DI6,
}

// Get the Digital Inputs from the Status Byte S3
impl DigitalInputDevice<Wago750_672InputPort> for Wago750_672 {
    fn get_input(&self, port: Wago750_672InputPort) -> Result<DigitalInputInput, anyhow::Error> {
        let s3 = StatusByteS3::from_bits(self.txpdo.status_byte3);
        Ok(DigitalInputInput {
            value: match port {
                Wago750_672InputPort::DI1 => s3.has_flag(S3Flag::Input1),
                Wago750_672InputPort::DI2 => s3.has_flag(S3Flag::Input2),
                Wago750_672InputPort::DI3 => s3.has_flag(S3Flag::Input3),
                Wago750_672InputPort::DI4 => s3.has_flag(S3Flag::Input4),
                Wago750_672InputPort::DI5 => s3.has_flag(S3Flag::Input5),
                Wago750_672InputPort::DI6 => s3.has_flag(S3Flag::Input6),
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
            status_byte0: b[0],
            actual_velocity: i16::from_le_bytes([b[2], b[3]]),
            position_l: b[6],
            position_m: b[7],
            position_h: b[8],
            status_byte3: b[9],
            status_byte2: b[10],
            status_byte1: b[11],
        };

        self.log_counter += 1;
        if self.log_counter >= 1000 {
            self.log_counter = 0;
            println!(
                "[750-672 INPUT] Received from EtherCAT: S0=0x{:02X} V={} S3=0x{:02X} S2=0x{:02X} S1=0x{:02X} | state={:?}",
                b[0],
                i16::from_le_bytes([b[2], b[3]]),
                b[9],
                b[10],
                b[11],
                self.state
            );
        }

        let s1 = StatusByteS1::from_bits(self.txpdo.status_byte1);
        let s2 = StatusByteS2::from_bits(self.txpdo.status_byte2);
        let s3 = StatusByteS3::from_bits(self.txpdo.status_byte3);
        let desired_command = self.desired_command;
        let command_ack = s1.bits() & 0b1111_1000;

        self.process_mailbox_response(&b);
        self.dispatch_next_mailbox_command();

        if !matches!(self.state, InitState::Off) && s3.has_flag(S3Flag::Reset) {
            self.state = InitState::ResetQuit;
        } else if !matches!(self.state, InitState::Off) && s2.has_flag(S2Flag::Error) {
            self.state = InitState::ErrorQuit;
        }

        // Yes this statemachine is unfortunately needed in here to make sure
        // bits are correctly set and reset in the correct cycle.
        match self.state {
            InitState::Off => {
                // Do nothing
                self.initialized = false;
                self.rxpdo.control_byte1 = 0;
                self.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
                self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);
                self.enable_stuck_counter = 0;
            }
            InitState::Enable => {
                if self.mailbox_busy() {
                    self.rxpdo.control_byte1 = 0;
                    return Ok(());
                }

                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_command(desired_command)
                    .bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);

                // The 750-672 manual allows mode selection to be requested
                // before Ready; the module delays the command until Ready and
                // Stop_N_ACK are active. Waiting at plain Enable can deadlock
                // on some speed-control configurations.
                if s1.has_flag(S1Flag::Ready) && command_ack == desired_command as u8 {
                    self.initialized = true;
                    self.enable_stuck_counter = 0;
                    self.state = if self.start_requested {
                        InitState::StartPulseStart
                    } else {
                        InitState::Running
                    };
                } else if s1.has_flag(S1Flag::Ready) {
                    self.enable_stuck_counter = 0;
                    self.state = InitState::SetMode;
                } else {
                    self.enable_stuck_counter = self.enable_stuck_counter.saturating_add(1);
                    if self.enable_stuck_counter >= 100 {
                        self.enable_stuck_counter = 0;
                    }
                }
            }
            InitState::SetMode => {
                if self.mailbox_busy() {
                    self.rxpdo.control_byte1 = 0;
                    return Ok(());
                }

                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_command(desired_command)
                    .bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);

                // The command acknowledgement lives in the upper C1/S1 bits.
                if s1.has_flag(S1Flag::Ready) && command_ack == desired_command as u8 {
                    self.initialized = true;
                    self.state = if self.start_requested {
                        InitState::StartPulseStart
                    } else {
                        InitState::Running
                    };
                }
            }
            InitState::StartPulseStart => {
                if self.mailbox_busy() {
                    self.rxpdo.control_byte1 = 0;
                    return Ok(());
                }

                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_flag(C1Flag::Start)
                    .with_command(desired_command)
                    .bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);

                // Switch state after Start pulse is acknowledged.
                if s1.has_flag(S1Flag::StartAck) {
                    self.start_requested = false;
                    self.state = InitState::StartPulseEnd;
                }
            }
            InitState::StartPulseEnd => {
                if self.mailbox_busy() {
                    self.rxpdo.control_byte1 = 0;
                    return Ok(());
                }

                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_command(desired_command)
                    .bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);

                // Switch state once the start acknowledgement has dropped.
                if !s1.has_flag(S1Flag::StartAck) {
                    self.state = InitState::Running;
                }
            }
            InitState::Running => {
                if self.mailbox_busy() {
                    self.rxpdo.control_byte1 = 0;
                    return Ok(());
                }

                let c1 = ControlByteC1::new()
                    .with_flag(C1Flag::Enable)
                    .with_flag(C1Flag::Stop2N)
                    .with_command(desired_command)
                    .bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
                self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);

                if s2.has_flag(S2Flag::Error) {
                    self.state = InitState::ErrorQuit;
                }

                if s3.has_flag(S3Flag::Reset) {
                    self.state = InitState::ResetQuit;
                }

                if self.start_requested {
                    self.state = InitState::StartPulseStart;
                }
            }
            InitState::ErrorQuit => {
                self.rxpdo.control_byte2 |= C2Flag::ErrorQuit as u8;
                self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);
                if !s2.has_flag(S2Flag::Error) {
                    tracing::error!("Stepper Controller Error acknowledged. Reenabling...");
                    self.initialized = false;
                    self.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
                    self.state = InitState::Enable;
                }
            }
            InitState::ResetQuit => {
                self.rxpdo.control_byte3 |= C3Flag::ResetQuit as u8;
                self.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
                if !s3.has_flag(S3Flag::Reset) {
                    tracing::error!("Stepper Controller Reset acknowledged. Reenabling...");
                    self.initialized = false;
                    self.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);
                    self.state = InitState::Enable;
                }
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

        let output_bytes = if let Some(mailbox) = self.mailbox_pending {
            [
                self.rxpdo.control_byte,
                0,
                mailbox[0],
                mailbox[1],
                mailbox[2],
                mailbox[3],
                mailbox[4],
                mailbox[5],
                0,
                self.rxpdo.control_byte3,
                self.rxpdo.control_byte2,
                self.rxpdo.control_byte1,
            ]
        } else {
            [
                self.rxpdo.control_byte,
                0,
                self.rxpdo.velocity.to_le_bytes()[0],
                self.rxpdo.velocity.to_le_bytes()[1],
                self.rxpdo.acceleration.to_le_bytes()[0],
                self.rxpdo.acceleration.to_le_bytes()[1],
                0,
                0,
                0,
                self.rxpdo.control_byte3,
                self.rxpdo.control_byte2,
                self.rxpdo.control_byte1,
            ]
        };

        if self.log_counter == 0 {
            println!(
                "[750-672 OUTPUT] Sending to EtherCAT: C0=0x{:02X} V={} A={} C3=0x{:02X} C2=0x{:02X} C1=0x{:02X} | state={:?}",
                output_bytes[0],
                i16::from_le_bytes([output_bytes[2], output_bytes[3]]),
                u16::from_le_bytes([output_bytes[4], output_bytes[5]]),
                output_bytes[9],
                output_bytes[10],
                output_bytes[11],
                self.state
            );
        }

        for i in 0..12 {
            output[base + i * 8..base + (i + 1) * 8].store_le(output_bytes[i]);
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
            log_counter: 0,
            enable_stuck_counter: 0,
            mailbox_toggle: false,
            mailbox_pending: None,
            mailbox_in_flight: false,
            mailbox_queue: VecDeque::new(),
            mailbox_active: false,
            config_defaults_queued: false,
            desired_command: C1Command::SpeedControl,
            start_requested: true,
            last_tms_enabling_block: None,
            last_diag_return_code: None,
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
