use super::{
    EK1100_ROLE, EL2002_ROLE, PULLER_ROLE, PullerMode, RewindPhase, Rewinder, RewinderMode,
    SOURCE_SPOOL_ROLE, SourceSpoolMode, TAKEUP_SPOOL_ROLE, TRAVERSE_ROLE, TakeupSpoolMode,
    TraverseMode,
};
use crate::{MachineHardware, MachineNew};
use control_core::converters::{
    angular_step_converter::AngularStepConverter, linear_step_converter::LinearStepConverter,
};
use qitech_lib::{
    ethercat_hal::{
        EtherCATThreadChannel,
        coe::ConfigurableDevice,
        devices::{
            ek1100::EK1100,
            el2002::EL2002,
            el7031::{EL7031, coe::EL7031Configuration, pdo::EL7031PredefinedPdoAssignment},
            el7031_0030::{
                self, EL7031_0030, coe::EL7031_0030Configuration,
                pdo::EL7031_0030PredefinedPdoAssignment,
            },
            el7041_0052::{EL7041_0052, coe::EL7041_0052Configuration},
        },
        shared_config,
        shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration},
    },
    units::{
        angle::degree,
        angular_velocity::revolution_per_minute,
        f64::*,
        length::{centimeter, millimeter},
        velocity::meter_per_minute,
    },
};
use std::time::Instant;

impl MachineNew for Rewinder {
    fn new(hw: MachineHardware) -> Result<Self, anyhow::Error> {
        let _ek1100 = hw.try_get_ethercat_device_and_addr_by_role::<EK1100>(EK1100_ROLE)?;
        let el2002 = hw.try_get_ethercat_device_and_addr_by_role::<EL2002>(EL2002_ROLE)?;
        let takeup_spool =
            hw.try_get_ethercat_device_and_addr_by_role::<EL7041_0052>(TAKEUP_SPOOL_ROLE)?;
        let traverse = hw.try_get_ethercat_device_and_addr_by_role::<EL7031>(TRAVERSE_ROLE)?;
        let puller = hw.try_get_ethercat_device_and_addr_by_role::<EL7031_0030>(PULLER_ROLE)?;
        let source_spool =
            hw.try_get_ethercat_device_and_addr_by_role::<EL7031_0030>(SOURCE_SPOOL_ROLE)?;

        let interface: EtherCATThreadChannel = hw
            .ethercat_interface
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Rewinder: No EtherCAT interface was supplied"))?;

        let el7031_0030_config = EL7031_0030Configuration {
            stm_features: el7031_0030::coe::StmFeatures {
                operation_mode: EL70x1OperationMode::DirectVelocity,
                speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                ..Default::default()
            },
            stm_motor: StmMotorConfiguration {
                max_current: 2700,
                ..Default::default()
            },
            pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
            ..Default::default()
        };

        for (device, address) in [&puller, &source_spool] {
            let mut device_ref = device.borrow_mut();
            (&mut *device_ref).write_config(interface.clone(), *address, &el7031_0030_config)?;
            drop(device_ref);
        }

        let el7031_config = EL7031Configuration {
            stm_features: shared_config::el70x1::StmFeatures {
                operation_mode: EL70x1OperationMode::DirectVelocity,
                speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                ..Default::default()
            },
            stm_motor: StmMotorConfiguration {
                max_current: 1500,
                ..Default::default()
            },
            pdo_assignment: EL7031PredefinedPdoAssignment::VelocityControlCompact,
            ..Default::default()
        };

        {
            let mut traverse_ref = traverse.0.borrow_mut();
            (&mut *traverse_ref).write_config(interface.clone(), traverse.1, &el7031_config)?;
        }

        let el7041_config = EL7041_0052Configuration {
            stm_features: shared_config::el70x1::StmFeatures {
                operation_mode: EL70x1OperationMode::DirectVelocity,
                ..Default::default()
            },
            stm_motor: StmMotorConfiguration {
                max_current: 2800,
                ..Default::default()
            },
            ..Default::default()
        };

        {
            let mut takeup_ref = takeup_spool.0.borrow_mut();
            (&mut *takeup_ref).write_config(interface.clone(), takeup_spool.1, &el7041_config)?;
        }

        let (api_sender, api_receiver) = tokio::sync::mpsc::channel(2);

        let mut source_spool_speed_controller =
            super::SpoolSpeedController::new_with_tension_range_and_response(
                Angle::new::<degree>(super::rewind_control::ArmConfig::SOURCE.hard_max_deg),
                Angle::new::<degree>(super::rewind_control::ArmConfig::SOURCE.hard_min_deg),
                crate::winder2::spool_speed_controller::SpoolTensionResponse::Source,
            );
        source_spool_speed_controller.set_adaptive_tension_target(0.70);
        source_spool_speed_controller.set_adaptive_radius_learning_rate(0.15);
        source_spool_speed_controller.set_adaptive_max_speed_multiplier(2.0);
        source_spool_speed_controller.set_adaptive_acceleration_factor(0.05);
        source_spool_speed_controller.set_adaptive_deacceleration_urgency_multiplier(20.0);

        let mut takeup_spool_speed_controller = super::SpoolSpeedController::new();
        let _ = takeup_spool_speed_controller
            .set_minmax_min_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
        let _ = takeup_spool_speed_controller
            .set_minmax_max_speed(AngularVelocity::new::<revolution_per_minute>(90.0));

        let mut rewinder = Self {
            api_receiver,
            api_sender,
            digital_outputs: el2002.0,
            traverse: traverse.0,
            takeup_spool: takeup_spool.0,
            puller: puller.0.clone(),
            source_spool: source_spool.0.clone(),
            takeup_tension_arm: super::TensionArm::new(puller.0.clone()),
            source_tension_arm: super::TensionArm::new(source_spool.0.clone()),
            namespace: super::api::RewinderNamespace { namespace: None },
            last_measurement_emit: Instant::now(),
            last_rewind_diagnostics_log: Instant::now(),
            machine_identification_unique: hw.identification,
            mode: RewinderMode::Standby,
            takeup_spool_mode: TakeupSpoolMode::Standby,
            source_spool_mode: SourceSpoolMode::Standby,
            traverse_mode: TraverseMode::Standby,
            puller_mode: PullerMode::Standby,
            puller_speed_controller: super::PullerSpeedController::new(
                Velocity::new::<meter_per_minute>(1.0),
                LinearStepConverter::from_diameter(200, Length::new::<centimeter>(8.0)),
            ),
            takeup_spool_speed_controller,
            source_spool_speed_controller,
            takeup_spool_step_converter: AngularStepConverter::new(200),
            source_spool_step_converter: AngularStepConverter::new(200),
            traverse_controller: super::TraverseController::new(
                Length::new::<millimeter>(22.0),
                Length::new::<millimeter>(92.0),
                64,
            ),
            rewind_phase: RewindPhase::Idle,
            rewind_control: super::rewind_control::RewindControlState::new(
                super::rewind_control::RewindControlConfig::default(),
            ),
            rewind_automatic_action: super::auto_stop::RewindAutomaticAction::default(),
            emitted_default_state: false,
            last_can_rewind: false,
        };

        rewinder.emit_state();
        Ok(rewinder)
    }
}
