use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::{
    coe::{ConfigurableDevice, Configuration},
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    io::pulse_train_output::{
        PulseTrainOutputDevice, PulseTrainOutputInput, PulseTrainOutputOutput,
    },
    pdo::{
        PredefinedPdoAssignment, RxPdo, TxPdo,
        el252x::{EncControl, EncStatus, PtoControl, PtoStatus, PtoTarget},
    },
};
use anyhow::Ok;
use ethercat_hal_derive::{EthercatDevice, RxPdo, TxPdo};

/// EL2521 2-channel pulse train output terminal
#[derive(EthercatDevice)]
pub struct EL2522 {
    pub configuration: EL2522Configuration,
    pub txpdo: EL2522TxPdo,
    pub rxpdo: EL2522RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL2522 {}

impl std::fmt::Debug for EL2522 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2522")
    }
}

impl NewEthercatDevice for EL2522 {
    /// Create a new EL2522 device with default configuration
    fn new() -> Self {
        let configuration = EL2522Configuration::default();
        let txpdo = configuration.pdo_assignment.txpdo_assignment();
        let rxpdo = configuration.pdo_assignment.rxpdo_assignment();
        Self {
            configuration,
            txpdo,
            rxpdo,
            is_used: false,
        }
    }
}

impl ConfigurableDevice<EL2522Configuration> for EL2522 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL2522Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL2522Configuration {
        self.configuration.clone()
    }
}

impl PulseTrainOutputDevice<EL2522Port> for EL2522 {
    fn set_output(&mut self, port: EL2522Port, value: PulseTrainOutputOutput) {
        let (pto_control, pto_target, enc_control) = self.get_rxpdo_mut(port);

        pto_control.disble_ramp = value.disble_ramp;
        pto_control.frequency_value = value.frequency_value;
        pto_target.target_counter_value = value.target_counter_value;
        enc_control.set_counter = value.set_counter;
        enc_control.set_counter_value = value.set_counter_value;
    }

    fn get_output(&self, port: EL2522Port) -> PulseTrainOutputOutput {
        let (pto_control, pto_target, enc_control) = self.get_rxpdo(port);

        PulseTrainOutputOutput {
            disble_ramp: pto_control.disble_ramp,
            frequency_value: pto_control.frequency_value,
            target_counter_value: pto_target.target_counter_value,
            set_counter: enc_control.set_counter,
            set_counter_value: enc_control.set_counter_value,
        }
    }
    fn get_input(&self, port: EL2522Port) -> PulseTrainOutputInput {
        let (pto_status, enc_status) = self.get_txpdo(port);

        PulseTrainOutputInput {
            select_end_counter: pto_status.select_end_counter,
            ramp_active: pto_status.ramp_active,
            input_t: pto_status.input_t,
            input_z: pto_status.input_z,
            error: pto_status.error,
            sync_error: pto_status.sync_error,
            counter_underflow: enc_status.counter_underflow,
            counter_overflow: enc_status.counter_overflow,
            counter_value: enc_status.counter_value,
            set_counter_done: enc_status.set_counter_done,
        }
    }
}

impl EL2522 {
    const fn get_txpdo(&self, port: EL2522Port) -> (&PtoStatus, &EncStatus) {
        match port {
            EL2522Port::PTO1 => (
                self.txpdo.pto_status_channel1.as_ref().unwrap(),
                self.txpdo.enc_status_channel1.as_ref().unwrap(),
            ),
            EL2522Port::PTO2 => (
                self.txpdo.pto_status_channel2.as_ref().unwrap(),
                self.txpdo.enc_status_channel2.as_ref().unwrap(),
            ),
        }
    }

    const fn get_rxpdo(&self, port: EL2522Port) -> (&PtoControl, &PtoTarget, &EncControl) {
        match port {
            EL2522Port::PTO1 => (
                self.rxpdo.pto_control_channel1.as_ref().unwrap(),
                self.rxpdo.pto_target_channel1.as_ref().unwrap(),
                self.rxpdo.enc_control_channel1.as_ref().unwrap(),
            ),
            EL2522Port::PTO2 => (
                self.rxpdo.pto_control_channel2.as_ref().unwrap(),
                self.rxpdo.pto_target_channel2.as_ref().unwrap(),
                self.rxpdo.enc_control_channel2.as_ref().unwrap(),
            ),
        }
    }
    const fn get_rxpdo_mut(
        &mut self,
        port: EL2522Port,
    ) -> (&mut PtoControl, &mut PtoTarget, &mut EncControl) {
        match port {
            EL2522Port::PTO1 => (
                self.rxpdo.pto_control_channel1.as_mut().unwrap(),
                self.rxpdo.pto_target_channel1.as_mut().unwrap(),
                self.rxpdo.enc_control_channel1.as_mut().unwrap(),
            ),
            EL2522Port::PTO2 => (
                self.rxpdo.pto_control_channel2.as_mut().unwrap(),
                self.rxpdo.pto_target_channel2.as_mut().unwrap(),
                self.rxpdo.enc_control_channel2.as_mut().unwrap(),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL2522Port {
    PTO1,
    PTO2,
}

#[derive(Debug, Clone)]
pub struct EL2522Configuration {
    pub pdo_assignment: EL2522PredefinedPdoAssignment,
    pub channel1_configuration: EL2522ChannelConfiguration,
    pub channel2_configuration: EL2522ChannelConfiguration,
}

#[derive(Debug, Clone)]
pub struct EL2522ChannelConfiguration {
    // PTO Settings
    /// # 0x8000:01 (Ch.1) / 0x8010:01 (Ch.2)
    /// If the counter value is set to "0", the C-track goes into the "high" state
    /// Default: false (0x00)
    pub adapt_a_b_on_position_set: bool,

    /// # 0x8000:02 (Ch.1) / 0x8010:02 (Ch.2)
    /// If the watchdog timer responds, the terminal ramps with the time constant set in object 0x8000:18
    /// Default: false (0x00)
    pub emergency_ramp_active: bool,

    /// # 0x8000:03 (Ch.1) / 0x8010:03 (Ch.2)
    /// The watchdog timer is deactivated
    /// Default: false (0x00)
    pub watchdog_timer_deactive: bool,

    /// # 0x8000:04 (Ch.1) / 0x8010:04 (Ch.2)
    /// TRUE: The output value is displayed in magnitude-sign format
    /// FALSE: The output value is output as a signed integer in two's complement
    /// Default: false (0x00)
    pub sign_amount_representation: bool,

    /// # 0x8000:06 (Ch.1) / 0x8010:06 (Ch.2)
    /// Ramp function activated
    /// Default: true (0x01)
    pub ramp_function_active: bool,

    /// # 0x8000:07 (Ch.1) / 0x8010:07 (Ch.2)
    /// 0: 10 Hz, 1: 1 kHz
    /// Default: false (0x00) - 10 Hz
    pub ramp_base_frequency: bool,

    /// # 0x8000:08 (Ch.1) / 0x8010:08 (Ch.2)
    /// TRUE: Direct input mode, FALSE: Relative input mode
    /// Default: false (0x00)
    pub direct_input_mode: bool,

    /// # 0x8000:09 (Ch.1) / 0x8010:09 (Ch.2)
    /// TRUE: User's switch-on value, FALSE: Manufacturer's switch-on value
    /// Default: false (0x00)
    pub user_switch_on_value_on_watchdog: bool,

    /// # 0x8000:0A (Ch.1) / 0x8010:0A (Ch.2)
    /// TRUE: Travel distance control activated, FALSE: Travel distance control deactivated
    /// Default: false (0x00)
    pub travel_distance_control: bool,

    /// # 0x8000:0E (Ch.1) / 0x8010:0E (Ch.2)
    /// 0: Frequency mod., 1: Pulse-dir, 2: Incremental enc.
    /// Default: FrequencyModulation (0x00)
    pub operating_mode: EL2522OperatingMode,

    /// # 0x8000:10 (Ch.1) / 0x8010:10 (Ch.2)
    /// TRUE: Negative logic, FALSE: Positive logic
    /// Default: false (0x00)
    pub negative_logic: bool,

    /// # 0x8000:11 (Ch.1) / 0x8010:11 (Ch.2)
    /// User switch-on value (frequency)
    /// Default: 0x0000 (0)
    pub user_switch_on_value: u16,

    /// # 0x8000:12 (Ch.1) / 0x8010:12 (Ch.2)
    /// Base frequency 1 = 50000 Hz
    /// Default: 0x0000C350 (50000)
    pub base_frequency_1: u32,

    /// # 0x8000:13 (Ch.1) / 0x8010:13 (Ch.2)
    /// Base frequency 2 = 100000 Hz
    /// Default: 0x000186A0 (100000)
    pub base_frequency_2: u32,

    /// # 0x8000:14 (Ch.1) / 0x8010:14 (Ch.2)
    /// Ramp time constant (rising)
    /// Default: 0x03E8 (1000)
    pub ramp_time_constant_rising: u16,

    /// # 0x8000:15 (Ch.1) / 0x8010:15 (Ch.2)
    /// Ramp time constant (falling)
    /// Default: 0x03E8 (1000)
    pub ramp_time_constant_falling: u16,

    /// # 0x8000:16 (Ch.1) / 0x8010:16 (Ch.2)
    /// Frequency factor (direct input, digit x 10 mHz)
    /// Default: 0x0064 (100)
    pub frequency_factor: u16,

    /// # 0x8000:17 (Ch.1) / 0x8010:17 (Ch.2)
    /// Slowing down frequency, travel distance control
    /// Default: 0x0032 (50)
    pub slowing_down_frequency: u16,

    /// # 0x8000:18 (Ch.1) / 0x8010:18 (Ch.2)
    /// Ramp time constant for controlled switch-off; User switch-on value is driven to (object 0x8000:11)
    /// Default: 0x03E8 (1000)
    pub ramp_time_constant_emergency: u16,

    // ENC Settings
    /// # 0x8020:01 (Ch.1) / 0x8030:01 (Ch.2)
    /// The counter is reset via the C-track
    /// Default: false (0x00)
    pub enable_c_reset: bool,

    /// # 0x8020:0A (Ch.1) / 0x8030:0A (Ch.2)
    /// The counter is more highly resolved with the bits specified in 0x8pp0:16
    /// Default: false (0x00)
    pub enable_micro_increments: bool,

    /// # 0x8020:18 (Ch.1) / 0x8030:18 (Ch.2)
    /// If 0x8pp0:0A is enabled: Number of micro increment bits
    /// Default: 0x0008 (8)
    pub micro_increment_bits: u16,

    /// # 0x8020:19 (Ch.1) / 0x8030:19 (Ch.2)
    /// If C-reset active: Number of increments "per revolution". At 1024 the counter counts
    /// Default: 0x00000400 (1024)
    pub pulses_per_revolution: u32,

    /// # 0x8020:1A (Ch.1) / 0x8030:1A (Ch.2)
    /// If the difference between "Target counter value" and "Counter value" exceeds this threshold, no output takes place.
    /// 0: function for automatic setting is inactive
    /// Default: 0x00000000 (0)
    pub autoset_threshold: u32,
}

impl Default for EL2522ChannelConfiguration {
    fn default() -> Self {
        Self {
            // PTO Settings
            adapt_a_b_on_position_set: false,
            emergency_ramp_active: false,
            watchdog_timer_deactive: false,
            sign_amount_representation: false,
            ramp_function_active: true,
            ramp_base_frequency: false,
            direct_input_mode: false,
            user_switch_on_value_on_watchdog: false,
            travel_distance_control: false,
            operating_mode: EL2522OperatingMode::FrequencyModulation,
            negative_logic: false,
            user_switch_on_value: 0x0000,
            base_frequency_1: 0x0000C350,         // 50000 Hz
            base_frequency_2: 0x000186A0,         // 100000 Hz
            ramp_time_constant_rising: 0x03E8,    // 1000
            ramp_time_constant_falling: 0x03E8,   // 1000
            frequency_factor: 0x0064,             // 100
            slowing_down_frequency: 0x0032,       // 50
            ramp_time_constant_emergency: 0x03E8, // 1000

            // ENC Settings
            enable_c_reset: false,
            enable_micro_increments: false,
            micro_increment_bits: 0x08,        // 8
            pulses_per_revolution: 0x00000400, // 1024
            autoset_threshold: 0x00000000,     // 0
        }
    }
}

impl Default for EL2522Configuration {
    fn default() -> Self {
        Self {
            pdo_assignment: EL2522PredefinedPdoAssignment::Standart32Bit,
            channel1_configuration: EL2522ChannelConfiguration::default(),
            channel2_configuration: EL2522ChannelConfiguration::default(),
        }
    }
}

impl Configuration for EL2522Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        // Write configuration for Channel 1
        self.write_channel_config(device, 0x8000, 0x8020, &self.channel1_configuration)
            .await?;

        // Write configuration for Channel 2
        self.write_channel_config(device, 0x8010, 0x8030, &self.channel2_configuration)
            .await?;

        // Write PDO assignments
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

impl EL2522Configuration {
    async fn write_channel_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
        pto_base_index: u16,
        enc_base_index: u16,
        config: &EL2522ChannelConfiguration,
    ) -> Result<(), anyhow::Error> {
        // Write PTO settings
        device
            .sdo_write(pto_base_index, 0x01, config.adapt_a_b_on_position_set)
            .await?;
        device
            .sdo_write(pto_base_index, 0x02, config.emergency_ramp_active)
            .await?;
        device
            .sdo_write(pto_base_index, 0x03, config.watchdog_timer_deactive)
            .await?;
        device
            .sdo_write(pto_base_index, 0x04, config.sign_amount_representation)
            .await?;
        device
            .sdo_write(pto_base_index, 0x06, config.ramp_function_active)
            .await?;
        device
            .sdo_write(pto_base_index, 0x07, config.ramp_base_frequency)
            .await?;
        device
            .sdo_write(pto_base_index, 0x08, config.direct_input_mode)
            .await?;
        device
            .sdo_write(
                pto_base_index,
                0x09,
                config.user_switch_on_value_on_watchdog,
            )
            .await?;
        device
            .sdo_write(pto_base_index, 0x0A, config.travel_distance_control)
            .await?;
        device
            .sdo_write(pto_base_index, 0x0E, u8::from(config.operating_mode))
            .await?;
        device
            .sdo_write(pto_base_index, 0x10, config.negative_logic)
            .await?;
        device
            .sdo_write(pto_base_index, 0x11, config.user_switch_on_value)
            .await?;
        device
            .sdo_write(pto_base_index, 0x12, config.base_frequency_1)
            .await?;
        device
            .sdo_write(pto_base_index, 0x13, config.base_frequency_2)
            .await?;
        device
            .sdo_write(pto_base_index, 0x14, config.ramp_time_constant_rising)
            .await?;
        device
            .sdo_write(pto_base_index, 0x15, config.ramp_time_constant_falling)
            .await?;
        device
            .sdo_write(pto_base_index, 0x16, config.frequency_factor)
            .await?;
        device
            .sdo_write(pto_base_index, 0x17, config.slowing_down_frequency)
            .await?;
        device
            .sdo_write(pto_base_index, 0x18, config.ramp_time_constant_emergency)
            .await?;

        // Write ENC settings
        device
            .sdo_write(enc_base_index, 0x01, config.enable_c_reset)
            .await?;
        device
            .sdo_write(enc_base_index, 0x0A, config.enable_micro_increments)
            .await?;
        device
            .sdo_write(enc_base_index, 0x18, config.micro_increment_bits)
            .await?;
        device
            .sdo_write(enc_base_index, 0x19, config.pulses_per_revolution)
            .await?;
        device
            .sdo_write(enc_base_index, 0x1A, config.autoset_threshold)
            .await?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EL2522OperatingMode {
    FrequencyModulation,
    PulseDirectionSpecification,
    PulseWidthModulation,
}

impl From<EL2522OperatingMode> for u8 {
    fn from(value: EL2522OperatingMode) -> Self {
        match value {
            EL2522OperatingMode::FrequencyModulation => 0,
            EL2522OperatingMode::PulseDirectionSpecification => 1,
            EL2522OperatingMode::PulseWidthModulation => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL2522PredefinedPdoAssignment {
    Standart32Bit,
}

impl PredefinedPdoAssignment<EL2522TxPdo, EL2522RxPdo> for EL2522PredefinedPdoAssignment {
    fn txpdo_assignment(&self) -> EL2522TxPdo {
        match self {
            Self::Standart32Bit => EL2522TxPdo {
                pto_status_channel1: Some(PtoStatus::default()),
                pto_status_channel2: Some(PtoStatus::default()),
                enc_status_channel1: Some(EncStatus::default()),
                enc_status_channel2: Some(EncStatus::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL2522RxPdo {
        match self {
            Self::Standart32Bit => EL2522RxPdo {
                pto_control_channel1: Some(PtoControl::default()),
                pto_target_channel1: Some(PtoTarget::default()),
                enc_control_channel1: Some(EncControl::default()),
                pto_control_channel2: Some(PtoControl::default()),
                pto_target_channel2: Some(PtoTarget::default()),
                enc_control_channel2: Some(EncControl::default()),
            },
        }
    }
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL2522TxPdo {
    #[pdo_object_index(0x1A00)]
    pub pto_status_channel1: Option<PtoStatus>,

    #[pdo_object_index(0x1A01)]
    pub pto_status_channel2: Option<PtoStatus>,

    #[pdo_object_index(0x1A03)]
    pub enc_status_channel1: Option<EncStatus>,

    #[pdo_object_index(0x1A05)]
    pub enc_status_channel2: Option<EncStatus>,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL2522RxPdo {
    #[pdo_object_index(0x1600)]
    pub pto_control_channel1: Option<PtoControl>,

    #[pdo_object_index(0x1603)]
    pub pto_target_channel1: Option<PtoTarget>,

    #[pdo_object_index(0x1605)]
    pub pto_control_channel2: Option<PtoControl>,

    #[pdo_object_index(0x1608)]
    pub pto_target_channel2: Option<PtoTarget>,

    #[pdo_object_index(0x160B)]
    pub enc_control_channel1: Option<EncControl>,

    #[pdo_object_index(0x160D)]
    pub enc_control_channel2: Option<EncControl>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_rx_pdo() {
        let mut buffer = [0u8; 28];
        let rxpdo = EL2522RxPdo {
            pto_control_channel1: Some(PtoControl {
                frequency_select: true,
                disble_ramp: true,
                go_counter: true,
                frequency_value: 1000,
            }),
            pto_target_channel1: Some(PtoTarget {
                target_counter_value: 1000001,
            }),

            pto_control_channel2: Some(PtoControl {
                frequency_select: true,
                disble_ramp: true,
                go_counter: true,
                frequency_value: 2000,
            }),
            pto_target_channel2: Some(PtoTarget {
                target_counter_value: 2000001,
            }),
            enc_control_channel1: Some(EncControl {
                set_counter: true,
                set_counter_value: 10001,
            }),
            enc_control_channel2: Some(EncControl {
                set_counter: true,
                set_counter_value: 10002,
            }),
        };
        let bits = buffer.view_bits_mut::<Lsb0>();
        rxpdo.write(bits).unwrap();
        // pto_control_channel1
        assert_eq!(buffer[0], 0b0000_0111);
        assert_eq!(u16::from_le_bytes([buffer[2], buffer[3]]), 1000);
        // pto_target_channel1
        assert_eq!(
            u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            1000001
        );
        // pto_control_channel2
        assert_eq!(buffer[8], 0b0000_0111);
        assert_eq!(u16::from_le_bytes([buffer[10], buffer[11]]), 2000);
        // pto_target_channel2
        assert_eq!(
            u32::from_le_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
            2000001
        );
        // enc_control_channel1
        assert_eq!(buffer[16], 0b0000_0100);
        assert_eq!(
            u32::from_le_bytes([buffer[18], buffer[19], buffer[20], buffer[21]]),
            10001
        );
        // enc_control_channel2
        assert_eq!(buffer[22], 0b0000_0100);
        assert_eq!(
            u32::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
            10002
        );
    }
}

pub const EL2522_VENDOR_ID: u32 = 0x2;
pub const EL2522_PRODUCT_ID: u32 = 0x09da3052;
pub const EL2522_REVISION_A: u32 = 0x00160000;
pub const EL2522_IDENTITY_A: SubDeviceIdentityTuple =
    (EL2522_VENDOR_ID, EL2522_PRODUCT_ID, EL2522_REVISION_A);
