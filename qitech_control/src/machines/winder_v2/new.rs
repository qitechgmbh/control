mod winder2_imports {
    pub use super::super::tension_arm::TensionArm;
    pub use crate::converters::angular_step_converter::AngularStepConverter;
    pub use crate::converters::linear_step_converter::LinearStepConverter;

    pub use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::ek1100::EK1100;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el2002::EL2002;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::EL7031;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::coe::EL7031Configuration;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::pdo::EL7031PredefinedPdoAssignment;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::coe::EL7031_0030Configuration;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::{self, EL7031_0030};
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::EL7041_0052;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::coe::EL7041_0052Configuration;

    pub use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;

    pub use qitech_lib::ethercat_hal::shared_config;
    pub use qitech_lib::ethercat_hal::shared_config::el70x1::{
        EL70x1OperationMode, StmMotorConfiguration,
    };
    pub use qitech_lib::units::ConstZero;
    pub use qitech_lib::units::f64::*;
    pub use qitech_lib::units::length::{centimeter, meter, millimeter};
    pub use qitech_lib::units::velocity::meter_per_minute;
    pub use std::time::Instant;
}

use std::cell::RefCell;
use std::rc::Rc;

use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::angle::degree;
use qitech_lib::units::angular_velocity::revolution_per_minute;
pub use winder2_imports::*;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::MachineBuild;

use crate::machines::winder_v2::SpoolAutomaticAction;
use crate::machines::winder_v2::VARIANT_7031_SPOOL;
use crate::machines::winder_v2::VARIANT_REGULAR;
use crate::machines::winder_v2::WinderV1;
use crate::machines::winder_v2::adaptive_spool_speed_controller::AdaptiveSpoolSpeedController;
use crate::machines::winder_v2::api::ConfigProperties;
use crate::machines::winder_v2::api::GearRatio;
use crate::machines::winder_v2::api::Measurements;
use crate::machines::winder_v2::api::ModeStateProperties;
use crate::machines::winder_v2::api::PullerRegulationMode;
use crate::machines::winder_v2::api::PullerStateProperties;
use crate::machines::winder_v2::api::SpoolSpeedControllerStateProperties;
use crate::machines::winder_v2::api::StateProperties;
use crate::machines::winder_v2::api::TensionArmStateProperties;
use crate::machines::winder_v2::api::TraverseStateProperties;
use crate::machines::winder_v2::puller_speed_controller::AdaptiveSpeedAlgorithm;
use crate::machines::winder_v2::puller_speed_controller::PullerSpeedController;
use crate::machines::winder_v2::spool_speed_controller::SpoolSpeedController;
use crate::machines::winder_v2::spool_speed_controller::SpoolSpeedControllerType;
use crate::machines::winder_v2::traverse_controller::TraverseController;
use crate::machines::winder_v2::types::Mode;
use crate::machines::winder_v2::types::PullerMode;
use crate::machines::winder_v2::types::SpoolAutomaticActionMode;
use crate::machines::winder_v2::types::SpoolMode;
use crate::machines::winder_v2::types::TraverseMode;
use crate::machines::winder_v2::types::Winder2Mode;

impl MachineBuild for WinderV1<VARIANT_REGULAR> {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let interface = ctx.get_ethercat_interface()?;

        init_ek1100(ctx)?;
        let el2002 = init_el2002(ctx)?;
        let el7041 = init_el7041(ctx, interface.clone())?;
        let el7031 = init_el7031(ctx, interface.clone())?;
        let el7031_0030 = init_el7031_0030(ctx, interface.clone())?;

        Self::new(
            ctx,
            el7031,
            el7031_0030.clone(),
            el7041,
            TensionArm::new(el7031_0030.clone()),
            el2002,
        )
    }
}

impl MachineBuild for WinderV1<VARIANT_7031_SPOOL> {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let interface = ctx.get_ethercat_interface()?;

        init_ek1100(ctx)?;
        let el2002 = init_el2002(ctx)?;
        let el7031_0030 = init_el7031_0030(ctx, interface.clone())?;
        let el7031 = init_el7031(ctx, interface.clone())?;
        let el7031_0030_spool = init_el7031_0030_spool(ctx, interface.clone())?;

        Self::new(
            ctx,
            el7031,
            el7031_0030.clone(),
            el7031_0030_spool,
            TensionArm::new(el7031_0030.clone()),
            el2002,
        )
    }
}

impl<const VARIANT: usize> WinderV1<VARIANT> {
    fn new(
        ctx: &mut BuildContext,
        traverse: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        puller: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        spool: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        tension_arm: TensionArm,
        laser: Rc<RefCell<dyn DigitalOutputDevice>>,
    ) -> BuildResult<Self> {
        Self::init_commands(ctx)?;

        let adaptive_spool_speed_controller = AdaptiveSpoolSpeedController::new(
            ctx.config::<f64>("spool.adaptive.tension_target")
                .minimum(0.0)
                .maximum(1.0)
                .default(0.7)
                .build()?,
            ctx.config::<f64>("spool.adaptive.radius_learning_rate")
                .minimum(0.0)
                .default(0.5)
                .build()?,
            ctx.config::<f64>("spool.adaptive.max_speed_multiplier")
                .minimum(0.1)
                .default(4.0)
                .build()?,
            ctx.config::<f64>("spool.adaptive.acceleration_factor")
                .minimum(0.01)
                .maximum(1.0)
                .default(0.2)
                .build()?,
            ctx.config::<f64>("spool.adaptive.deacceleration_urgency_multiplier")
                .minimum(1.0)
                .default(15.0)
                .build()?,
        );

        let adapative_puller_algorithm = AdaptiveSpeedAlgorithm::new(
            ctx
                .config::<f64>("puller.adapative.speed_delta_max")
                .minimum(0.0)
                .default(0.33)
                .build()?,
            ctx
                .config::<f64>("puller.adapative.increase_per_step")
                .minimum(0.0)
                .maximum(1.0)
                .default(0.033)
                .build()?,
            ctx
                .config::<meter>("puller.adapative.adjustment_interval")
                .default(0.01)
                .build()?,
            ctx
                .config::<millimeter>("puller.adapative.accepted_difference")
                .default(0.5)
                .build()?,
        );

        Ok(Self {
            traverse,
            puller,
            spool,
            tension_arm,
            laser,
            laser_enabled: false,
            traverse_controller: TraverseController::new(
                Length::new::<millimeter>(22.0), // Default inner limit
                Length::new::<millimeter>(92.0), // Default outer limit
                64,                              // Microsteps
                ctx.config::<millimeter>("traverse.step_size")
                    .default(1.75)
                    .build()?,
                ctx.config::<millimeter>("traverse.padding")
                    .default(0.88)
                    .build()?,
            ),
            mode: Winder2Mode::Standby,
            spool_mode: SpoolMode::Standby,
            traverse_mode: TraverseMode::Standby,
            puller_mode: PullerMode::Standby,
            spool_speed_controller: SpoolSpeedController::new(adaptive_spool_speed_controller),
            spool_step_converter: AngularStepConverter::new(200),
            spool_automatic_action: SpoolAutomaticAction {
                progress: Length::ZERO,
                progress_last_check: Instant::now(),
                target_length: ctx
                    .config::<meter>("spool_automatic.required_meters")
                    .default(250.0)
                    .build()?,
                mode: ctx
                    .config::<SpoolAutomaticActionMode>("spool_automatic.action")
                    .default(SpoolAutomaticActionMode::NoAction)
                    .build()?,
            },
            puller_speed_controller: PullerSpeedController::new(
                ctx.config::<meter_per_minute>("puller.target_speed")
                    .default(1.0)
                    .build()?,
                LinearStepConverter::from_diameter(
                    200,                            // Assuming 200 steps per revolution for the puller stepper,
                    Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                ),
                ctx.config::<bool>("puller.forward").default(true).build()?,
                adapative_puller_algorithm,
            ),
            config_props: Self::init_config_properties(ctx)?,
            state_props: Self::init_state_properties(ctx)?,
            measurements: Self::init_measurements(ctx)?,
            laser_subscription: None,
        })
    }
}

// --- hardware ---
fn init_ek1100(ctx: &BuildContext) -> BuildResult<()> {
    ctx.find_ethercat_device_and_addr::<EK1100>(0)?;
    Ok(())
}

fn init_el2002(ctx: &BuildContext) -> BuildResult<Rc<RefCell<EL2002>>> {
    ctx.find_ethercat_device::<EL2002>(1)
}

fn init_el7031_0030(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL7031_0030>>> {
    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7031_0030>(4)?;

    let config = EL7031_0030Configuration {
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

    dev.borrow_mut()
        .write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

// used for old spool
fn init_el7041(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL7041_0052>>> {
    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7041_0052>(2)?;

    let config = EL7041_0052Configuration {
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

    dev.borrow_mut()
        .write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

fn init_el7031(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL7031>>> {
    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7031>(3)?;

    let config = EL7031Configuration {
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

    dev.borrow_mut()
        .write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

// used for new spool
fn init_el7031_0030_spool(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL7031_0030>>> {
    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7031_0030>(2)?;

    let config = EL7031_0030Configuration {
        stm_features: el7031_0030::coe::StmFeatures {
            operation_mode: EL70x1OperationMode::DirectVelocity,
            speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps2000,
            ..Default::default()
        },
        stm_motor: StmMotorConfiguration {
            max_current: 2700,
            ..Default::default()
        },
        pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
        ..Default::default()
    };

    dev.borrow_mut()
        .write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

// --- resources ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    fn init_config_properties(ctx: &mut BuildContext) -> BuildResult<ConfigProperties> {
        Ok(ConfigProperties {
            traverse_limit_inner: ctx
                .config::<millimeter>("traverse.limit_inner")
                .on_external_changed(Self::on_traverse_limit_inner_changed)
                .default(22.0)
                .build()?,
            traverse_limit_outer: ctx
                .config::<millimeter>("traverse.limit_outer")
                .on_external_changed(Self::on_traverse_limit_outer_changed)
                .default(92.0)
                .build()?,
            puller_regulation_mode: ctx
                .config::<PullerRegulationMode>("puller.regulation_mode")
                .on_external_changed(Self::on_puller_regulation_mode_changed)
                .default(PullerRegulationMode::Speed)
                .build()?,
            puller_gear_ratio: ctx
                .config::<GearRatio>("puller.gear_ratio")
                .on_external_changed(Self::on_puller_gear_ratio_changed)
                .default(GearRatio::OneToOne)
                .build()?,
            spool_regulation_mode: ctx
                .config::<SpoolSpeedControllerType>("spool.regulation_mode")
                .on_external_changed(Self::on_spool_regulation_mode_changed)
                .default(SpoolSpeedControllerType::Adaptive)
                .build()?,
            spool_forward: ctx
                .config::<bool>("spool.forward")
                .on_external_changed(Self::on_spool_forward_changed)
                .default(true)
                .build()?,
            spool_min_speed: ctx
                .config::<revolution_per_minute>("spool.min_max.speed_min")
                .on_external_changed(Self::on_spool_min_speed_changed)
                .default(0.0)
                .build()?,
            spool_max_speed: ctx
                .config::<revolution_per_minute>("spool.min_max.speed_max")
                .on_external_changed(Self::on_spool_max_speed_changed)
                .default(0.0)
                .build()?,
        })
    }

    fn init_state_properties(ctx: &mut BuildContext) -> BuildResult<StateProperties> {
        Ok(StateProperties {
            traverse_state: TraverseStateProperties {
                limit_inner: ctx.state::<millimeter>("traverse.limit_inner").build()?,
                limit_outer: ctx.state::<millimeter>("traverse.limit_outer").build()?,
                is_going_in: ctx.state::<bool>("traverse.is_going_in").build()?,
                is_going_out: ctx.state::<bool>("traverse.is_going_out").build()?,
                is_homed: ctx.state::<bool>("traverse.is_homed").build()?,
                is_going_home: ctx.state::<bool>("traverse.is_going_home").build()?,
                is_traversing: ctx.state::<bool>("traverse.is_traversing").build()?,
                laserpointer: ctx.state::<bool>("traverse.laserpointer").build()?,
                can_go_in: ctx.state::<bool>("traverse.can_go_in").build()?,
                can_go_out: ctx.state::<bool>("traverse.can_go_out").build()?,
                can_go_home: ctx.state::<bool>("traverse.can_go_home").build()?,
            },

            puller_state: PullerStateProperties {
                regulation: ctx
                    .state::<PullerRegulationMode>("puller.regulation")
                    .build()?,
                gear_ratio: ctx.state::<GearRatio>("puller.gear_ratio").build()?,
            },

            mode_state: ModeStateProperties {
                mode: ctx.state::<Mode>("mode.mode").build()?,
                can_wind: ctx.state::<bool>("mode.can_wind").build()?,
            },

            tension_arm_state: TensionArmStateProperties {
                zeroed: ctx.state::<bool>("tension_arm.zeroed").build()?,
            },

            spool_speed_controller_state: SpoolSpeedControllerStateProperties {
                regulation_mode: ctx
                    .state::<SpoolSpeedControllerType>("spool.regulation_mode")
                    .build()?,
                minmax_min_speed: ctx
                    .state::<revolution_per_minute>("spool.minmax.speed_min")
                    .build()?,
                minmax_max_speed: ctx
                    .state::<revolution_per_minute>("spool.minmax.speed_max")
                    .build()?,
                forward: ctx.state::<bool>("spool.forward").build()?,
            },
        })
    }

    fn init_measurements(ctx: &mut BuildContext) -> BuildResult<Measurements> {
        Ok(Measurements {
            traverse_position: ctx
                .measurement::<Option<millimeter>>("traverse.position")
                .build()?,

            puller_speed: ctx
                .measurement::<meter_per_minute>("puller.speed")
                .build()?,

            spool_rpm: ctx
                .measurement::<revolution_per_minute>("spool.rpm")
                .build()?,

            tension_arm_angle: ctx.measurement::<degree>("tension_arm.angle").build()?,
            spool_progress: ctx.measurement::<meter>("spool.progress").build()?,
        })
    }

    fn init_commands(ctx: &mut BuildContext) -> BuildResult<()> {
        // --- modes ---
        ctx.command("enter_standby_mode")
            .execute(Self::cmd_enter_standby_mode)
            .build()?;

        ctx.command("enter_hold_mode")
            .execute(Self::cmd_enter_hold_mode)
            .build()?;

        ctx.command("enter_pull_mode")
            .execute(Self::cmd_enter_pull_mode)
            .build()?;

        ctx.command("enter_wind_mode")
            .execute(Self::cmd_enter_wind_mode)
            .build()?;

        // --- traverse goto ---
        ctx.command("traverse.goto_home")
            .can_execute(Self::traverse_can_goto_home)
            .execute(Self::cmd_traverse_goto_home)
            .build()?;

        ctx.command("traverse.goto_limit_inner")
            .can_execute(Self::traverse_can_goto_limit_inner)
            .execute(Self::cmd_traverse_goto_limit_inner)
            .build()?;

        ctx.command("traverse.goto_limit_outer")
            .can_execute(Self::traverse_can_goto_limit_outer)
            .execute(Self::cmd_traverse_goto_limit_outer)
            .build()?;

        // --- traverse laser ---
        ctx.command("traverse.laserpointer.enable")
            .execute(Self::cmd_traverse_laser_enable)
            .build()?;

        ctx.command("traverse.laserpointer.disable")
            .execute(Self::cmd_traverse_laser_disable)
            .build()?;

        // --- spool ---
        ctx.command("spool.reset_progress")
            .execute(Self::cmd_spool_reset_progress)
            .build()?;

        // --- tension arm ---
        ctx.command("tension_arm.set_zero")
            .execute(Self::cmd_tension_arm_set_zero)
            .build()?;

        Ok(())
    }
}
