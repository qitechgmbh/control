use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::pdo::PredefinedPdoAssignment;
use crate::pdo::analog_input::{AiCompact, AiStandard};
use crate::pdo::el70x1::{
    EncControl, EncControlCompact, EncStatus, EncStatusCompact, EncTimestampCompact,
    PosActualPositionLag, PosControl, PosControl2, PosControlCompact, PosStatus, PosStatusCompact,
    StmControl, StmExternalPosition, StmInternalPosition, StmPosition, StmStatus,
    StmSynchronInfoData, StmVelocity,
};
use ethercat_hal_derive::{RxPdo, TxPdo};

#[derive(Debug, Clone, TxPdo)]
pub struct EL7031_0030TxPdo {
    #[pdo_object_index(0x1A00)]
    pub enc_status_compact: Option<EncStatusCompact>,

    #[pdo_object_index(0x1A01)]
    pub enc_status: Option<EncStatus>,

    #[pdo_object_index(0x1A02)]
    pub enc_timestamp_compact: Option<EncTimestampCompact>,

    #[pdo_object_index(0x1A03)]
    pub stm_status: Option<StmStatus>,

    #[pdo_object_index(0x1A04)]
    pub stm_synchron_info_data: Option<StmSynchronInfoData>,

    #[pdo_object_index(0x1A05)]
    pub pos_status_compact: Option<PosStatusCompact>,

    #[pdo_object_index(0x1A06)]
    pub pos_status: Option<PosStatus>,

    #[pdo_object_index(0x1A07)]
    pub stm_internal_position: Option<StmInternalPosition>,

    #[pdo_object_index(0x1A08)]
    pub stm_external_position: Option<StmExternalPosition>,

    #[pdo_object_index(0x1A09)]
    pub pos_actual_position_lag: Option<PosActualPositionLag>,

    #[pdo_object_index(0x1A0A)]
    pub ai_standard_channel_1: Option<AiStandard>,

    #[pdo_object_index(0x1A0B)]
    pub ai_compact_channel_1: Option<AiCompact>,

    #[pdo_object_index(0x1A0C)]
    pub ai_standard_channel_2: Option<AiStandard>,

    #[pdo_object_index(0x1A0D)]
    pub ai_compact_channel_2: Option<AiCompact>,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL7031_0030RxPdo {
    #[pdo_object_index(0x1600)]
    pub enc_control_compact: Option<EncControlCompact>,

    #[pdo_object_index(0x1601)]
    pub enc_control: Option<EncControl>,

    #[pdo_object_index(0x1602)]
    pub stm_control: Option<StmControl>,

    #[pdo_object_index(0x1603)]
    pub stm_position: Option<StmPosition>,

    #[pdo_object_index(0x1604)]
    pub stm_velocity: Option<StmVelocity>,

    #[pdo_object_index(0x1605)]
    pub pos_control_compact: Option<PosControlCompact>,

    #[pdo_object_index(0x1606)]
    pub pos_control: Option<PosControl>,

    #[pdo_object_index(0x1607)]
    pub pos_control_2: Option<PosControl2>,
}

#[derive(Debug, Clone, Default)]
pub enum EL7031_0030PredefinedPdoAssignment {
    VelocityControlCompact,
    VelocityControlCompactWithInfoData,
    VelocityControl,
    #[default]
    PositionControl,
    PositionInterfaceCompact,
    PositionInterface,
    PositionInterfaceWithInfoData,
    PositionInterfaceAutoStart,
    PositionInterfaceAutoStartWithInfoData,
}

impl PredefinedPdoAssignment<EL7031_0030TxPdo, EL7031_0030RxPdo>
    for EL7031_0030PredefinedPdoAssignment
{
    fn txpdo_assignment(&self) -> EL7031_0030TxPdo {
        match self {
            Self::VelocityControlCompact => EL7031_0030TxPdo {
                enc_status_compact: Some(EncStatusCompact::default()),
                enc_status: None,
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: None,
                pos_status_compact: None,
                pos_status: None,
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::VelocityControlCompactWithInfoData => EL7031_0030TxPdo {
                enc_status_compact: Some(EncStatusCompact::default()),
                enc_status: None,
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: Some(StmSynchronInfoData::default()),
                pos_status_compact: None,
                pos_status: None,
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::VelocityControl => EL7031_0030TxPdo {
                enc_status_compact: None,
                enc_status: Some(EncStatus::default()),
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: None,
                pos_status_compact: None,
                pos_status: None,
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::PositionControl => EL7031_0030TxPdo {
                enc_status_compact: None,
                enc_status: Some(EncStatus::default()),
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: None,
                pos_status_compact: None,
                pos_status: None,
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::PositionInterfaceCompact => EL7031_0030TxPdo {
                enc_status_compact: None,
                enc_status: Some(EncStatus::default()),
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: None,
                pos_status_compact: Some(PosStatusCompact::default()),
                pos_status: None,
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::PositionInterface => EL7031_0030TxPdo {
                enc_status_compact: None,
                enc_status: Some(EncStatus::default()),
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: None,
                pos_status_compact: None,
                pos_status: Some(PosStatus::default()),
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::PositionInterfaceWithInfoData => EL7031_0030TxPdo {
                enc_status_compact: None,
                enc_status: Some(EncStatus::default()),
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: Some(StmSynchronInfoData::default()),
                pos_status_compact: None,
                pos_status: Some(PosStatus::default()),
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::PositionInterfaceAutoStart => EL7031_0030TxPdo {
                enc_status_compact: None,
                enc_status: Some(EncStatus::default()),
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: None,
                pos_status_compact: None,
                pos_status: Some(PosStatus::default()),
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
            Self::PositionInterfaceAutoStartWithInfoData => EL7031_0030TxPdo {
                enc_status_compact: None,
                enc_status: Some(EncStatus::default()),
                enc_timestamp_compact: None,
                stm_status: Some(StmStatus::default()),
                stm_synchron_info_data: Some(StmSynchronInfoData::default()),
                pos_status_compact: None,
                pos_status: Some(PosStatus::default()),
                stm_internal_position: None,
                stm_external_position: None,
                pos_actual_position_lag: None,
                ai_standard_channel_1: Some(AiStandard::default()),
                ai_compact_channel_1: None,
                ai_standard_channel_2: Some(AiStandard::default()),
                ai_compact_channel_2: None,
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL7031_0030RxPdo {
        match self {
            Self::VelocityControlCompact | Self::VelocityControlCompactWithInfoData => {
                EL7031_0030RxPdo {
                    enc_control_compact: Some(EncControlCompact::default()),
                    enc_control: None,
                    stm_control: Some(StmControl::default()),
                    stm_position: None,
                    stm_velocity: Some(StmVelocity::default()),
                    pos_control_compact: None,
                    pos_control: None,
                    pos_control_2: None,
                }
            }
            Self::VelocityControl => EL7031_0030RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: None,
                stm_velocity: Some(StmVelocity::default()),
                pos_control_compact: None,
                pos_control: None,
                pos_control_2: None,
            },
            Self::PositionControl => EL7031_0030RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: Some(StmPosition::default()),
                stm_velocity: None,
                pos_control_compact: None,
                pos_control: None,
                pos_control_2: None,
            },
            Self::PositionInterfaceCompact => EL7031_0030RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: None,
                stm_velocity: None,
                pos_control_compact: Some(PosControlCompact::default()),
                pos_control: None,
                pos_control_2: None,
            },
            Self::PositionInterface | Self::PositionInterfaceWithInfoData => EL7031_0030RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: None,
                stm_velocity: None,
                pos_control_compact: None,
                pos_control: Some(PosControl::default()),
                pos_control_2: None,
            },
            Self::PositionInterfaceAutoStart | Self::PositionInterfaceAutoStartWithInfoData => {
                EL7031_0030RxPdo {
                    enc_control_compact: None,
                    enc_control: Some(EncControl::default()),
                    stm_control: Some(StmControl::default()),
                    stm_position: None,
                    stm_velocity: None,
                    pos_control_compact: None,
                    pos_control: Some(PosControl::default()),
                    pos_control_2: Some(PosControl2::default()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdo::{RxPdo, TxPdo};

    #[test]
    fn test_pdo_assignment_velocity_control_compact() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::VelocityControlCompact;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 16 * 8);
        assert_eq!(rxpdo.size(), 8 * 8);
    }

    #[test]
    fn test_pdo_assignment_velocity_control_compact_with_info_data() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::VelocityControlCompactWithInfoData;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 20 * 8);
        assert_eq!(rxpdo.size(), 8 * 8);
    }

    #[test]
    fn test_pdo_assignment_velocity_control() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::VelocityControl;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 20 * 8);
        assert_eq!(rxpdo.size(), 10 * 8);
    }

    #[test]
    fn test_pdo_assignment_position_control() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::PositionControl;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 20 * 8);
        assert_eq!(rxpdo.size(), 12 * 8);
    }

    #[test]
    fn test_pdo_assignment_position_interface_compact() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::PositionInterfaceCompact;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 22 * 8);
        assert_eq!(rxpdo.size(), 14 * 8);
    }

    #[test]
    fn test_pdo_assignment_position_interface() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::PositionInterface;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 32 * 8);
        assert_eq!(rxpdo.size(), 22 * 8);
    }

    #[test]
    fn test_pdo_assignment_position_interface_with_info_data() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::PositionInterfaceWithInfoData;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 36 * 8);
        assert_eq!(rxpdo.size(), 22 * 8);
    }

    #[test]
    fn test_pdo_assignment_position_interface_auto_start() {
        let pdo_assignment = EL7031_0030PredefinedPdoAssignment::PositionInterfaceAutoStart;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 32 * 8);
        assert_eq!(rxpdo.size(), 36 * 8);
    }

    #[test]
    fn test_pdo_assignment_position_interface_auto_start_with_info_data() {
        let pdo_assignment =
            EL7031_0030PredefinedPdoAssignment::PositionInterfaceAutoStartWithInfoData;
        let txpdo = pdo_assignment.txpdo_assignment();
        let rxpdo = pdo_assignment.rxpdo_assignment();

        assert_eq!(txpdo.size(), 36 * 8);
        assert_eq!(rxpdo.size(), 36 * 8);
    }
}
