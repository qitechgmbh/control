/*
 * Wago Stepper Controller 750-671
 * 24 VDC / 1.5 A
 */

use anyhow::Ok;
use bitvec::field::BitField;

use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    io::digital_input::{DigitalInputDevice, DigitalInputInput},
    io::stepper_velocity_wago_750_671::{
        C1Flag, C1Mode, C2Flag, C3Flag, ControlByteC1, S1Flag, S2Flag, S3Flag, StatusByteS1,
        StatusByteS2, StatusByteS3,
    },
};

fn decode_i24(l: u8, m: u8, h: u8) -> i32 {
    let raw = (l as u32) | ((m as u32) << 8) | ((h as u32) << 16);
    if (raw & 0x0080_0000) != 0 {
        (raw | 0xFF00_0000) as i32
    } else {
        raw as i32
    }
}

#[derive(Clone)]
pub struct Wago750_671 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_671RxPdo,
    pub txpdo: Wago750_671TxPdo,
    module: Option<Module>,
    pub state: InitState,
    pub initialized: bool,
    last_error_snapshot: Option<[u8; 6]>,
    pub desired_mode: C1Mode,
    pub desired_stop2_n: bool,
    pub desired_control_byte3: u8,
    pub start_requested: bool,

    // Mailbox command support
    pub mailbox_active: bool,
    pub mailbox_toggle: bool,
    pub mailbox_pending: Option<[u8; 6]>,
    pub mailbox_in_flight: bool,
    pub mailbox_return_to_speed_mode: bool,
    pub mailbox_last_status_toggle: bool,
    pub mailbox_logged_dispatch: bool,
}
impl Wago750_671 {
    pub fn queue_mailbox_command(
        &mut self,
        opcode: u8,
        p1: u8,
        p2: u8,
        p3: u8,
        p4: u8,
        return_to_speed_mode: bool,
    ) {
        self.mailbox_toggle = !self.mailbox_toggle;
        let control_mbx = if self.mailbox_toggle { 0x80 } else { 0x00 };

        // MB0..MB5
        self.mailbox_pending = Some([opcode, control_mbx, p1, p2, p3, p4]);
        self.mailbox_active = true;
        self.mailbox_in_flight = false;
        self.mailbox_return_to_speed_mode = return_to_speed_mode;
        self.mailbox_logged_dispatch = false;
    }

    fn apply_mailbox_bytes(&mut self, bytes: [u8; 6]) {
        // Mailbox process image when C0.5 = 1:
        // byte2 MB0 opcode
        // byte3 MB1 control/toggle
        // byte4 MB2 param1
        // byte5 MB3 param2
        // byte6 MB4 param3
        // byte7 MB5 param4
        self.rxpdo.velocity = i16::from_le_bytes([bytes[0], bytes[1]]);
        self.rxpdo.acceleration = u16::from_le_bytes([bytes[2], bytes[3]]);
        self.rxpdo.target_position_l = bytes[4];
        self.rxpdo.target_position_m = bytes[5];
        self.rxpdo.target_position_h = 0;
    }

    fn set_mailbox_bit(&mut self, active: bool) {
        if active {
            self.rxpdo.control_byte0 |= 0b0010_0000; // C0.5 = MBX
        } else {
            self.rxpdo.control_byte0 &= !0b0010_0000;
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
* we always want to use the latter one so the following
* process images are for that usecase.
*/

#[derive(Clone, Debug, Default)]
pub struct Wago750_671RxPdo {
    pub control_byte0: u8,     // C0
    pub velocity: i16,         // D0/D1
    pub acceleration: u16,     // D2/D3
    pub target_position_l: u8, // D4
    pub target_position_m: u8, // D5
    pub target_position_h: u8, // D6
    pub control_byte3: u8,     // C3
    pub control_byte2: u8,     // C2
    pub control_byte1: u8,     // C1
}

#[derive(Clone, Debug, Default)]
pub struct Wago750_671TxPdo {
    pub status_byte0: u8,     // S0
    pub actual_velocity: i16, // D0/D1
    pub position_l: u8,       // D4
    pub position_m: u8,       // D5
    pub position_h: u8,       // D6
    pub status_byte3: u8,     // S3
    pub status_byte2: u8,     // S2
    pub status_byte1: u8,     // S1
}
#[derive(Clone)]
pub enum Wago750_671InputPort {
    DI1,
    DI2,
}

impl DigitalInputDevice<Wago750_671InputPort> for Wago750_671 {
    fn get_input(&self, port: Wago750_671InputPort) -> Result<DigitalInputInput, anyhow::Error> {
        let s3 = StatusByteS3::from_bits(self.txpdo.status_byte3);
        Ok(DigitalInputInput {
            value: match port {
                Wago750_671InputPort::DI1 => s3.has_flag(S3Flag::Input1),
                Wago750_671InputPort::DI2 => s3.has_flag(S3Flag::Input2),
            },
        })
    }
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
            status_byte0: b[0],
            actual_velocity: i16::from_le_bytes([b[2], b[3]]),
            position_l: b[6],
            position_m: b[7],
            position_h: b[8],
            status_byte3: b[9],
            status_byte2: b[10],
            status_byte1: b[11],
        };

        let s1 = StatusByteS1::from_bits(self.txpdo.status_byte1);
        let s2 = StatusByteS2::from_bits(self.txpdo.status_byte2);
        let s3 = StatusByteS3::from_bits(self.txpdo.status_byte3);
        // Mailbox status toggle is in input byte 3 / MB1 status, bit 7.
        // In the current txpdo struct that byte overlaps with actual_velocity MSB,
        // so inspect the raw input bytes directly.
        let mailbox_status_toggle = (b[3] & 0x80) != 0;

        if self.mailbox_in_flight && mailbox_status_toggle == self.mailbox_toggle {
            self.mailbox_in_flight = false;
            self.mailbox_pending = None;
            self.mailbox_active = false;
            self.set_mailbox_bit(false);

            if self.mailbox_return_to_speed_mode {
                self.mailbox_return_to_speed_mode = false;
                self.desired_mode = C1Mode::SpeedControl;
                self.desired_control_byte3 = 0;
                self.start_requested = false;
                self.initialized = false;
                self.state = InitState::SetMode;
            }
        }

        self.mailbox_last_status_toggle = mailbox_status_toggle;

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
            }
            InitState::Enable => {
                // set the specific bits of Control Byte C1
                let mut c1 = ControlByteC1::new().with_flag(C1Flag::Enable);
                if self.desired_stop2_n {
                    c1 = c1.with_flag(C1Flag::Stop2N);
                }
                let c1 = c1.bits();
                self.rxpdo.control_byte1 = c1;

                // Switch state once the terminal is ready. Stop2_N ack is not
                // guaranteed to mirror C1 bytewise on every module config.
                if s1.has_flag(S1Flag::Ready) {
                    self.state = InitState::SetMode;
                }
            }
            InitState::SetMode => {
                // set the specific bits of Control Byte C1 and also the mode
                let mut c1 = ControlByteC1::new().with_flag(C1Flag::Enable);
                if self.desired_stop2_n {
                    c1 = c1.with_flag(C1Flag::Stop2N);
                }
                let c1 = c1.with_mode(self.desired_mode).bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte3 = self.desired_control_byte3 & !(C3Flag::ResetQuit as u8);

                // Switch state once primary application ack is present.
                if s1.has_flag(S1Flag::Ready) && (s1.bits() & (self.desired_mode as u8) != 0) {
                    self.initialized = true;
                    if self.start_requested {
                        self.state = InitState::StartPulseStart;
                    } else {
                        self.state = InitState::Running;
                    }
                }
            }
            InitState::StartPulseStart => {
                let mut c1 = ControlByteC1::new().with_flag(C1Flag::Enable);
                if self.desired_stop2_n {
                    c1 = c1.with_flag(C1Flag::Stop2N);
                }
                let c1 = c1
                    .with_flag(C1Flag::Start)
                    .with_mode(self.desired_mode)
                    .bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte3 = self.desired_control_byte3 & !(C3Flag::ResetQuit as u8);

                // Switch state after Start pulse is acknowledged.
                if s1.has_flag(S1Flag::StartAck) {
                    self.state = InitState::StartPulseEnd;
                }
            }
            InitState::StartPulseEnd => {
                let mut c1 = ControlByteC1::new().with_flag(C1Flag::Enable);
                if self.desired_stop2_n {
                    c1 = c1.with_flag(C1Flag::Stop2N);
                }
                let c1 = c1.with_mode(self.desired_mode).bits();
                self.rxpdo.control_byte1 = c1;
                self.rxpdo.control_byte3 = self.desired_control_byte3 & !(C3Flag::ResetQuit as u8);

                // Switch state once the start ack has dropped again.
                if !s1.has_flag(S1Flag::StartAck) {
                    self.start_requested = false;
                    self.state = InitState::Running;
                }
            }
            InitState::Running => {
                let mut c1 = ControlByteC1::new().with_flag(C1Flag::Enable);
                if self.desired_stop2_n {
                    c1 = c1.with_flag(C1Flag::Stop2N);
                }
                self.rxpdo.control_byte1 = c1.with_mode(self.desired_mode).bits();
                self.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
                self.rxpdo.control_byte3 = self.desired_control_byte3 & !(C3Flag::ResetQuit as u8);
                self.last_error_snapshot = None;

                if self.mailbox_active {
                    if let Some(bytes) = self.mailbox_pending {
                        if !self.mailbox_in_flight {
                            self.set_mailbox_bit(true);
                            self.apply_mailbox_bytes(bytes);
                            self.mailbox_in_flight = true;
                            if !self.mailbox_logged_dispatch {
                                tracing::info!(
                                    "750-671 mailbox dispatch | opcode=0x{:02X} arg0=0x{:02X} arg1=0x{:02X} arg2=0x{:02X} arg3=0x{:02X} return_to_speed_mode={}",
                                    bytes[0],
                                    bytes[2],
                                    bytes[3],
                                    bytes[4],
                                    bytes[5],
                                    self.mailbox_return_to_speed_mode,
                                );
                                self.mailbox_logged_dispatch = true;
                            }
                        }
                    } else {
                        self.set_mailbox_bit(false);
                    }
                } else {
                    self.set_mailbox_bit(false);
                }

                if s2.has_flag(S2Flag::Error) {
                    self.state = InitState::ErrorQuit;
                }

                if s3.has_flag(S3Flag::Reset) {
                    self.state = InitState::ResetQuit;
                }

                if s1.has_flag(S1Flag::Ready) && (s1.bits() & (self.desired_mode as u8) == 0) {
                    self.initialized = false;
                    self.state = InitState::SetMode;
                } else if self.start_requested {
                    self.state = InitState::StartPulseStart;
                }
            }
            InitState::ErrorQuit => {
                self.rxpdo.control_byte2 |= C2Flag::ErrorQuit as u8;
                self.rxpdo.control_byte3 = self.desired_control_byte3 & !(C3Flag::ResetQuit as u8);
                if s2.has_flag(S2Flag::Error) {
                    let snapshot = [
                        self.rxpdo.control_byte1,
                        self.rxpdo.control_byte2,
                        self.rxpdo.control_byte3,
                        self.txpdo.status_byte1,
                        self.txpdo.status_byte2,
                        self.txpdo.status_byte3,
                    ];
                    if self.last_error_snapshot != Some(snapshot) {
                        self.last_error_snapshot = Some(snapshot);
                        tracing::error!(
                            "750-671 error recovery active | c1=0x{:02X} c2=0x{:02X} c3=0x{:02X} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X} pos={} vel={} initialized={}",
                            self.rxpdo.control_byte1,
                            self.rxpdo.control_byte2,
                            self.rxpdo.control_byte3,
                            self.txpdo.status_byte1,
                            self.txpdo.status_byte2,
                            self.txpdo.status_byte3,
                            decode_i24(
                                self.txpdo.position_l,
                                self.txpdo.position_m,
                                self.txpdo.position_h
                            ),
                            self.txpdo.actual_velocity,
                            self.initialized,
                        );
                    }
                } else {
                    self.last_error_snapshot = None;
                    self.initialized = false;
                    self.start_requested = false;
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
                    self.start_requested = false;
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

        let b = [
            self.rxpdo.control_byte0,
            0,
            self.rxpdo.velocity.to_le_bytes()[0],
            self.rxpdo.velocity.to_le_bytes()[1],
            self.rxpdo.acceleration.to_le_bytes()[0],
            self.rxpdo.acceleration.to_le_bytes()[1],
            self.rxpdo.target_position_l,
            self.rxpdo.target_position_m,
            self.rxpdo.target_position_h,
            self.rxpdo.control_byte3,
            self.rxpdo.control_byte2,
            self.rxpdo.control_byte1,
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
            state: InitState::Off,
            initialized: false,
            last_error_snapshot: None,
            desired_mode: C1Mode::SpeedControl,
            desired_stop2_n: true,
            desired_control_byte3: 0,
            start_requested: false,

            mailbox_active: false,
            mailbox_toggle: false,
            mailbox_pending: None,
            mailbox_in_flight: false,
            mailbox_return_to_speed_mode: false,
            mailbox_last_status_toggle: false,
            mailbox_logged_dispatch: false,
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
