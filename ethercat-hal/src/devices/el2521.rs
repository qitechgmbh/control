use super::Device;
use crate::{
    coe::{Configuration, RX_PDO_ASSIGNMENT_REG, TX_PDO_ASSIGNMENT_REG},
    io::pulse_train_output::{
        PulseTrainOutputDevice, PulseTrainOutputState, PulseTrainOutputWrite,
    },
    pdo::{
        PdoObject, PdoPreset, RxPdo, RxPdoObject, TxPdo, TxPdoObject,
        EL252X::{EncControl, EncStatus, PtoControl, PtoStatus, PtoTarget},
    },
    types::EthercrabSubDevice,
};
use anyhow::Ok;
use ethercat_hal_derive::{RxPdo, TxPdo};
use std::any::Any;

/// EL2521 8-channel digital output device
///   
/// 24V DC, 0.5A per channel
#[derive(Debug)]
pub struct EL2521 {
    pub configuration: EL2521Configuration,
    pub txpdo: EL2521TxPdo,
    pub rxpdo: EL2521RxPdo,
    pub output_ts: u64,
}

impl EL2521 {
    /// Create a new EL2521 device with default configuration
    pub fn new() -> Self {
        let configuration = EL2521Configuration::default();
        let txpdo = configuration.pdo_assignment.txpdo_assignment();
        let rxpdo = configuration.pdo_assignment.rxpdo_assignment();
        Self {
            configuration: configuration,
            txpdo,
            rxpdo,
            output_ts: 0,
        }
    }

    pub fn set_configuration(&mut self, configuration: &EL2521Configuration) {
        self.configuration = configuration.clone();
        self.txpdo = configuration.pdo_assignment.txpdo_assignment();
        self.rxpdo = configuration.pdo_assignment.rxpdo_assignment();
    }
}

impl Device for EL2521 {
    fn output(&self, output: &mut [u8]) {
        // log::info!("EL2521 output {:?}", self.output_pdus);
        self.rxpdo.write(output);
    }
    fn output_len(&self) -> usize {
        self.rxpdo.size()
    }
    fn input(&mut self, _input: &[u8]) {
        log::info!("EL2521 input {:?}", _input);
        self.txpdo.read(_input);
    }
    fn input_len(&self) -> usize {
        self.txpdo.size()
    }
    fn ts(&mut self, _input_ts: u64, output_ts: u64) {
        self.output_ts = output_ts;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PulseTrainOutputDevice<EL2521Port> for EL2521 {
    fn pulse_train_output_write(&mut self, port: EL2521Port, value: PulseTrainOutputWrite) {
        match self.configuration.pdo_assignment {
            EL2521PdoPreset::EnhancedOperatingMode32Bit => {}
            _ => {
                panic!("Only EnhancedOperatingMode32Bit is supported");
            }
        }
    }

    fn pulse_train_output_state(&self, port: EL2521Port) -> PulseTrainOutputState {
        match self.configuration.pdo_assignment {
            EL2521PdoPreset::EnhancedOperatingMode32Bit => {}
            _ => {
                panic!("Only EnhancedOperatingMode32Bit is supported");
            }
        }

        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub enum EL2521Port {
    PTO1,
}

/// 0x8000 CoE
#[derive(Debug, Clone)]
pub struct EL2521Configuration {
    /// # 0x8010:02
    /// - `true` = If the watchdog timer responds, the terminal ramps with the time constant set in object 8001:08
    /// - `false` = The function is deactivated
    ///
    /// default: `false`
    pub emergency_ramp_active: bool,

    /// # 0x8010:03
    /// - `true` = The watchdog timer is deactivated
    ///
    /// The watchdog timer is activated in the delivery state.
    /// Either the manufacturer's of the user's switch-on value
    /// is output if the watchdog overflows
    ///
    /// default: `false`
    pub watchdog_timer_deactive: bool,

    /// # 0x8010:04
    pub sign_amount_representation: bool,

    // /// # 0x8010:05
    // pub rising_edge_clears_counter: bool,
    /// 0x8010:06
    pub ramp_function_active: bool,

    /// # 0x8010:07
    /// - `true` = 1kHz
    /// - `false` = 10kHz
    ///
    /// default: `false`
    pub ramp_base_frequency: bool,

    /// # 0x8010:08
    /// -`true` = direct input mode
    /// -`false` = relative input mode
    ///
    /// default: `false`
    pub direct_input_mode: bool,

    /// # 0x8010:09
    /// -`true` = Behavior with triggered watchdog timer: User switch-on value
    /// -`false` = Behavior with triggered watchdog timer: Manufacturer switch-on value
    ///
    /// default: `false`
    pub user_switch_on_value_on_watchdog: bool,

    /// # 0x8010:0A
    /// -`true` = Travel distance control activated
    /// -`false` = Travel distance control deactivated
    ///
    /// default: `false`
    pub travel_distance_control: bool,

    // /// # 0x8010:0B
    // /// Inversion of the output logic 24V
    // /// -`false` = high level in switched state
    // /// -`true` = low level in switched state
    // ///
    // /// default: `false`
    // pub output_set_active_low: bool,
    /// # 0x8010:0E
    /// - `0` = Frequency modulation operating mode (pull-down menu)
    /// - `1` = Pulse direction specification operating mode (pull-down menu)
    /// - `2` = Pulse width modulation operating mode (pull-down menu)
    ///
    /// default: `FrequencyModulation`
    pub operating_mode: EL2521OperatingMode,

    /// # 0x8010:10
    pub negative_logic: bool,

    /// # 0x8010:11
    /// User switch-on value (frequency)
    ///
    /// default: `0x0000`
    pub user_switch_on_value: u16,

    /// # 0x8010:12
    /// Base frequency 1 = 50kHz
    ///
    /// default: `0x0000C350` (50kHz)
    pub base_frequency_1: u32,

    /// # 0x8010:13
    /// Base frequency 2 = 100kHz
    ///
    /// default: `0x000186A0` (100kHz)
    pub base_frequency_2: u32,

    /// # 0x8010:14
    /// Ramp time constant (rising)
    ///
    /// default: `0x03E8` (1000)
    pub ramp_time_constant_rising: u16,

    /// # 0x8010:15
    /// Ramp time constant (falling)
    ///
    /// default: `0x03E8` (1000)
    pub ramp_time_constant_falling: u16,

    /// # 0x8010:16
    /// Frequency factor (direct input, digit x 10 mHz)
    ///
    /// default: `0x0064` (100)
    pub frequency_factor: u16,

    /// # 0x8010:17
    /// Slowing down frequency, travel distance control
    ///
    /// default: `0x0032` (50)
    pub slowing_down_frequency: u16,

    /// # 0x8010:18
    /// Ramp time constant for controlled switch-off;
    ///
    /// User switch-on value is driven to (object 0x8000:11)
    ///
    /// default: `0x03E8` (1000)
    pub ramp_time_constant_emergency: u16,

    /// # 0x1400 & 0x1600
    pub pdo_assignment: EL2521PdoPreset,
}

impl Default for EL2521Configuration {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            emergency_ramp_active: false,
            watchdog_timer_deactive: false,
            sign_amount_representation: false,
            // rising_edge_clears_counter: false,
            ramp_function_active: true,
            ramp_base_frequency: false,
            direct_input_mode: false,
            user_switch_on_value_on_watchdog: false,
            travel_distance_control: false,
            // output_set_active_low: false,
            operating_mode: EL2521OperatingMode::FrequencyModulation,
            negative_logic: false,
            user_switch_on_value: 0x0000,
            base_frequency_1: 0x0000C350,         // 50kHz
            base_frequency_2: 0x000186A0,         // 100kHz
            ramp_time_constant_rising: 0x03E8,    // 1000
            ramp_time_constant_falling: 0x03E8,   // 1000
            frequency_factor: 0x0064,             // 100
            slowing_down_frequency: 0x0032,       // 50
            ramp_time_constant_emergency: 0x03E8, // 1000
            pdo_assignment: EL2521PdoPreset::EnhancedOperatingMode32Bit,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EL2521OperatingMode {
    FrequencyModulation,
    PulseDirectionSpecification,
    PulseWidthModulation,
}

impl From<EL2521OperatingMode> for u8 {
    fn from(value: EL2521OperatingMode) -> Self {
        match value {
            EL2521OperatingMode::FrequencyModulation => 0,
            EL2521OperatingMode::PulseDirectionSpecification => 1,
            EL2521OperatingMode::PulseWidthModulation => 2,
        }
    }
}

impl Configuration for EL2521Configuration {
    async fn write_config<'a>(
        &self,
        device: &'a EthercrabSubDevice<'a>,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(0x8010, 0x02, self.emergency_ramp_active)
            .await?;
        device
            .sdo_write(0x8010, 0x03, self.watchdog_timer_deactive)
            .await?;
        device
            .sdo_write(0x8010, 0x04, self.sign_amount_representation)
            .await?;
        device
            .sdo_write(0x8010, 0x06, self.ramp_function_active)
            .await?;
        device
            .sdo_write(0x8010, 0x07, self.ramp_base_frequency)
            .await?;
        device
            .sdo_write(0x8010, 0x08, self.direct_input_mode)
            .await?;
        device
            .sdo_write(0x8010, 0x09, self.user_switch_on_value_on_watchdog)
            .await?;
        device
            .sdo_write(0x8010, 0x0A, self.travel_distance_control)
            .await?;
        device
            .sdo_write(0x8010, 0x0E, u8::from(self.operating_mode))
            .await?;
        device.sdo_write(0x8010, 0x10, self.negative_logic).await?;
        device
            .sdo_write(0x8010, 0x11, self.user_switch_on_value)
            .await?;
        device
            .sdo_write(0x8010, 0x12, self.base_frequency_1)
            .await?;
        device
            .sdo_write(0x8010, 0x13, self.base_frequency_2)
            .await?;
        device
            .sdo_write(0x8010, 0x14, self.ramp_time_constant_rising)
            .await?;
        device
            .sdo_write(0x8010, 0x15, self.ramp_time_constant_falling)
            .await?;
        device
            .sdo_write(0x8010, 0x16, self.frequency_factor)
            .await?;
        device
            .sdo_write(0x8010, 0x17, self.slowing_down_frequency)
            .await?;
        device
            .sdo_write(0x8010, 0x18, self.ramp_time_constant_emergency)
            .await?;
        self.pdo_assignment
            .txpdo_assignment()
            .write_config(device)
            .await?;
        self.pdo_assignment
            .rxpdo_assignment()
            .write_config(device)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum EL2521PdoPreset {
    EnhancedOperatingMode32Bit,
}

impl PdoPreset<EL2521TxPdo, EL2521RxPdo> for EL2521PdoPreset {
    fn txpdo_assignment(&self) -> EL2521TxPdo {
        match self {
            EL2521PdoPreset::EnhancedOperatingMode32Bit => EL2521TxPdo {
                pto_status: Some(PtoStatus::default()),
                enc_status: Some(EncStatus::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL2521RxPdo {
        match self {
            EL2521PdoPreset::EnhancedOperatingMode32Bit => EL2521RxPdo {
                pto_control: Some(PtoControl::default()),
                pto_target: Some(PtoTarget::default()),
                enc_control: Some(EncControl::default()),
            },
        }
    }
}

#[derive(Debug, Clone, TxPdo)]
struct EL2521TxPdo {
    /// # `0x1A01` PTO Status
    #[pdo_object_index(0x1A01)]
    pub pto_status: Option<PtoStatus>,

    /// # `0x1A02` Encoder Status
    #[pdo_object_index(0x1A02)]
    pub enc_status: Option<EncStatus>,
}

#[derive(Debug, Clone, RxPdo)]
struct EL2521RxPdo {
    /// # `0x1601` PTO Control
    #[pdo_object_index(0x1601)]
    pub pto_control: Option<PtoControl>,

    /// # `0x1607` PTO Target
    #[pdo_object_index(0x1607)]
    pub pto_target: Option<PtoTarget>,

    /// # `0x1605` Encoder Control
    #[pdo_object_index(0x1605)]
    pub enc_control: Option<EncControl>,
}
