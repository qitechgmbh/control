use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::pdo::PredefinedPdoAssignment;
use crate::pdo::el70x1::{
    EncControl, EncControlCompact, EncStatus, EncStatusCompact, EncTimestampCompact,
    PosActualPositionLag, PosControl, PosControl2, PosControlCompact, PosStatus, PosStatusCompact,
    StmControl, StmExternalPosition, StmInternalPosition, StmPosition, StmStatus,
    StmSynchronInfoData, StmVelocity,
};
use ethercat_hal_derive::{RxPdo, TxPdo};

#[derive(Debug, Clone, TxPdo)]
pub struct EL7041_0052TxPdo {
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
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL7041_0052RxPdo {
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
pub enum EL7041_0052PredefinedPdoAssignment {
    #[default]
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

impl PredefinedPdoAssignment<EL7041_0052TxPdo, EL7041_0052RxPdo>
    for EL7041_0052PredefinedPdoAssignment
{
    fn txpdo_assignment(&self) -> EL7041_0052TxPdo {
        match self {
            Self::VelocityControlCompact => EL7041_0052TxPdo {
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
            },
            Self::VelocityControlCompactWithInfoData => EL7041_0052TxPdo {
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
            },
            Self::VelocityControl => EL7041_0052TxPdo {
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
            },
            Self::PositionControl => EL7041_0052TxPdo {
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
            },
            Self::PositionInterfaceCompact => EL7041_0052TxPdo {
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
            },
            Self::PositionInterface => EL7041_0052TxPdo {
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
            },
            Self::PositionInterfaceWithInfoData => EL7041_0052TxPdo {
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
            },
            Self::PositionInterfaceAutoStart => EL7041_0052TxPdo {
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
            },
            Self::PositionInterfaceAutoStartWithInfoData => EL7041_0052TxPdo {
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
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL7041_0052RxPdo {
        match self {
            Self::VelocityControlCompact | Self::VelocityControlCompactWithInfoData => {
                EL7041_0052RxPdo {
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
            Self::VelocityControl => EL7041_0052RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: None,
                stm_velocity: Some(StmVelocity::default()),
                pos_control_compact: None,
                pos_control: None,
                pos_control_2: Some(PosControl2::default()),
            },
            Self::PositionControl => EL7041_0052RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: Some(StmPosition::default()),
                stm_velocity: None,
                pos_control_compact: None,
                pos_control: None,
                pos_control_2: Some(PosControl2::default()),
            },
            Self::PositionInterfaceCompact => EL7041_0052RxPdo {
                enc_control_compact: None,
                enc_control: Some(EncControl::default()),
                stm_control: Some(StmControl::default()),
                stm_position: None,
                stm_velocity: None,
                pos_control_compact: Some(PosControlCompact::default()),
                pos_control: None,
                pos_control_2: Some(PosControl2::default()),
            },
            Self::PositionInterface | Self::PositionInterfaceWithInfoData => EL7041_0052RxPdo {
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
                EL7041_0052RxPdo {
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
