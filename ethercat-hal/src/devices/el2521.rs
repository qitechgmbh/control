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

/// EL2521 1-channel pulse train output terminal
#[derive(EthercatDevice)]
pub struct EL2521 {
    pub configuration: EL2521Configuration,
    pub txpdo: EL2521TxPdo,
    pub rxpdo: EL2521RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL2521 {}

impl std::fmt::Debug for EL2521 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2521")
    }
}

impl NewEthercatDevice for EL2521 {
    /// Create a new EL2521 device with default configuration
    fn new() -> Self {
        let configuration = EL2521Configuration::default();
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

impl PulseTrainOutputDevice<EL2521Port> for EL2521 {
    fn set_output(&mut self, _port: EL2521Port, value: PulseTrainOutputOutput) {
        self.rxpdo.pto_control.as_mut().unwrap().disble_ramp = value.disble_ramp;
        self.rxpdo.pto_control.as_mut().unwrap().frequency_value = value.frequency_value;
        self.rxpdo.pto_target.as_mut().unwrap().target_counter_value = value.target_counter_value;
        self.rxpdo.enc_control.as_mut().unwrap().set_counter = value.set_counter;
        self.rxpdo.enc_control.as_mut().unwrap().set_counter_value = value.set_counter_value;
    }

    fn get_output(&self, _port: EL2521Port) -> PulseTrainOutputOutput {
        PulseTrainOutputOutput {
            disble_ramp: self.rxpdo.pto_control.as_ref().unwrap().disble_ramp,
            frequency_value: self.rxpdo.pto_control.as_ref().unwrap().frequency_value,
            target_counter_value: self.rxpdo.pto_target.as_ref().unwrap().target_counter_value,
            set_counter: self.rxpdo.enc_control.as_ref().unwrap().set_counter,
            set_counter_value: self.rxpdo.enc_control.as_ref().unwrap().set_counter_value,
        }
    }

    fn get_input(&self, _port: EL2521Port) -> PulseTrainOutputInput {
        PulseTrainOutputInput {
            select_end_counter: self.txpdo.pto_status.as_ref().unwrap().select_end_counter,
            ramp_active: self.txpdo.pto_status.as_ref().unwrap().ramp_active,
            input_t: self.txpdo.pto_status.as_ref().unwrap().input_t,
            input_z: self.txpdo.pto_status.as_ref().unwrap().input_z,
            error: self.txpdo.pto_status.as_ref().unwrap().error,
            sync_error: self.txpdo.pto_status.as_ref().unwrap().sync_error,
            counter_underflow: self.txpdo.enc_status.as_ref().unwrap().counter_underflow,
            counter_overflow: self.txpdo.enc_status.as_ref().unwrap().counter_overflow,
            counter_value: self.txpdo.enc_status.as_ref().unwrap().counter_value,
            set_counter_done: self.txpdo.enc_status.as_ref().unwrap().set_counter_done,
        }
    }
}

impl ConfigurableDevice<EL2521Configuration> for EL2521 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL2521Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL2521Configuration {
        self.configuration.clone()
    }
}

#[derive(Debug, Clone, Copy)]
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
    /// Either the manufacturer's or the user's switch-on value
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
    pub pdo_assignment: EL2521PredefinedPdoAssignment,
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
            pdo_assignment: EL2521PredefinedPdoAssignment::EnhancedOperatingMode32Bit,
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
        device: &EthercrabSubDevicePreoperational<'a>,
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
pub enum EL2521PredefinedPdoAssignment {
    EnhancedOperatingMode32Bit,
}

impl PredefinedPdoAssignment<EL2521TxPdo, EL2521RxPdo> for EL2521PredefinedPdoAssignment {
    fn txpdo_assignment(&self) -> EL2521TxPdo {
        match self {
            Self::EnhancedOperatingMode32Bit => EL2521TxPdo {
                pto_status: Some(PtoStatus::default()),
                enc_status: Some(EncStatus::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL2521RxPdo {
        match self {
            Self::EnhancedOperatingMode32Bit => EL2521RxPdo {
                pto_control: Some(PtoControl::default()),
                pto_target: Some(PtoTarget::default()),
                enc_control: Some(EncControl::default()),
            },
        }
    }
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL2521TxPdo {
    #[pdo_object_index(0x1A01)]
    pub pto_status: Option<PtoStatus>,

    #[pdo_object_index(0x1A05)]
    pub enc_status: Option<EncStatus>,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL2521RxPdo {
    #[pdo_object_index(0x1601)]
    pub pto_control: Option<PtoControl>,

    #[pdo_object_index(0x1607)]
    pub pto_target: Option<PtoTarget>,

    #[pdo_object_index(0x1605)]
    pub enc_control: Option<EncControl>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_rx_pdo() {
        let mut buffer = [0u8; 14];
        let rxpdo = EL2521RxPdo {
            pto_control: Some(PtoControl {
                frequency_select: true,
                disble_ramp: true,
                go_counter: true,
                frequency_value: 1000,
            }),
            pto_target: Some(PtoTarget {
                target_counter_value: 1000000,
            }),
            enc_control: Some(EncControl {
                set_counter: true,
                set_counter_value: 1000000,
            }),
        };
        let bits = buffer.view_bits_mut::<Lsb0>();
        rxpdo.write(bits).unwrap();
        assert_eq!(buffer[0], 0b0000_0111);
        assert_eq!(u16::from_le_bytes([buffer[2], buffer[3]]), 1000);
        assert_eq!(
            u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            1000000
        );
        assert_eq!(buffer[8], 0b0000_0100);
        assert_eq!(
            u32::from_le_bytes([buffer[10], buffer[11], buffer[12], buffer[13]]),
            1000000
        );
    }
}

pub const EL2521_VENDOR_ID: u32 = 0x2;
pub const EL2521_PRODUCT_ID: u32 = 0x09d93052;
pub const EL2521_REVISION_0000_A: u32 = 0x03fe0000;
pub const EL2521_REVISION_0000_B: u32 = 0x03f90000;
pub const EL2521_REVISION_0024_A: u32 = 0x03f80018;
pub const EL2521_IDENTITY_0000_A: SubDeviceIdentityTuple =
    (EL2521_VENDOR_ID, EL2521_PRODUCT_ID, EL2521_REVISION_0000_A);
pub const EL2521_IDENTITY_0000_B: SubDeviceIdentityTuple =
    (EL2521_VENDOR_ID, EL2521_PRODUCT_ID, EL2521_REVISION_0000_B);
pub const EL2521_IDENTITY_0024_A: SubDeviceIdentityTuple =
    (EL2521_VENDOR_ID, EL2521_PRODUCT_ID, EL2521_REVISION_0024_A);
