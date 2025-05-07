use crate::pdo::PredefinedPdoAssignment;
use crate::pdo::el30xx::{AiCompact, AiStandard};
use crate::pdo::el70x1::{
    EncControl, EncControlCompact, EncStatus, EncStatusCompact, EncTimestampCompact,
    PosActualPositionLag, PosControl, PosControl2, PosControlCompact, PosStatus, PosStatusCompact,
    StmControl, StmExternalPosition, StmInternalPosition, StmPosition, StmStatus,
    StmSynchronInfoData, StmVelocity,
};
use crate::types::EthercrabSubDevicePreoperational;
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

#[derive(Debug, Clone)]
pub enum EL7031_0030PredefinedPdoAssignment {
    VelocityControlCompact,
    VelocityControlCompactWithInfoData,
    VelocityControl,
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
            EL7031_0030PredefinedPdoAssignment::VelocityControlCompact => EL7031_0030TxPdo {
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
                ai_standard_channel_1: None,
                ai_compact_channel_1: Some(AiCompact::default()),
                ai_standard_channel_2: None,
                ai_compact_channel_2: Some(AiCompact::default()),
            },
            EL7031_0030PredefinedPdoAssignment::VelocityControlCompactWithInfoData => {
                EL7031_0030TxPdo {
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
                    ai_standard_channel_1: None,
                    ai_compact_channel_1: Some(AiCompact::default()),
                    ai_standard_channel_2: None,
                    ai_compact_channel_2: Some(AiCompact::default()),
                }
            }
            EL7031_0030PredefinedPdoAssignment::VelocityControl => EL7031_0030TxPdo {
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
            EL7031_0030PredefinedPdoAssignment::PositionControl => EL7031_0030TxPdo {
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
            EL7031_0030PredefinedPdoAssignment::PositionInterfaceCompact => EL7031_0030TxPdo {
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
            EL7031_0030PredefinedPdoAssignment::PositionInterface => EL7031_0030TxPdo {
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
            EL7031_0030PredefinedPdoAssignment::PositionInterfaceWithInfoData => EL7031_0030TxPdo {
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
            EL7031_0030PredefinedPdoAssignment::PositionInterfaceAutoStart => EL7031_0030TxPdo {
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
            EL7031_0030PredefinedPdoAssignment::PositionInterfaceAutoStartWithInfoData => {
                EL7031_0030TxPdo {
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
                }
            }
        }
    }

    fn rxpdo_assignment(&self) -> EL7031_0030RxPdo {
        match self {
            EL7031_0030PredefinedPdoAssignment::VelocityControlCompact
            | EL7031_0030PredefinedPdoAssignment::VelocityControlCompactWithInfoData => {
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
            EL7031_0030PredefinedPdoAssignment::VelocityControl => EL7031_0030RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: None,
                stm_velocity: Some(StmVelocity::default()),
                pos_control_compact: None,
                pos_control: None,
                pos_control_2: None,
            },
            EL7031_0030PredefinedPdoAssignment::PositionControl => EL7031_0030RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: Some(StmPosition::default()),
                stm_velocity: None,
                pos_control_compact: None,
                pos_control: None,
                pos_control_2: None,
            },
            EL7031_0030PredefinedPdoAssignment::PositionInterfaceCompact => EL7031_0030RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: None,
                stm_velocity: None,
                pos_control_compact: Some(PosControlCompact::default()),
                pos_control: None,
                pos_control_2: Some(PosControl2::default()),
            },
            EL7031_0030PredefinedPdoAssignment::PositionInterface
            | EL7031_0030PredefinedPdoAssignment::PositionInterfaceWithInfoData => {
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
            EL7031_0030PredefinedPdoAssignment::PositionInterfaceAutoStart
            | EL7031_0030PredefinedPdoAssignment::PositionInterfaceAutoStartWithInfoData => {
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

impl Default for EL7031_0030PredefinedPdoAssignment {
    fn default() -> Self {
        EL7031_0030PredefinedPdoAssignment::VelocityControlCompact
    }
}
