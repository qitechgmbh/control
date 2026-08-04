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

use qitech_framework::machine::error::BuildError;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::angle::degree;
use qitech_lib::units::angular_velocity::revolution_per_minute;
pub use winder2_imports::*;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::error::BuildResult;

use crate::machines::winder_v2::SpoolAutomaticAction;
use crate::machines::winder_v2::WinderV1;
use crate::machines::winder_v2::api::ConfigProperties;
use crate::machines::winder_v2::api::GearRatio;
use crate::machines::winder_v2::api::Measurements;
use crate::machines::winder_v2::api::ModeStateProperties;
use crate::machines::winder_v2::api::PullerRegulationMode;
use crate::machines::winder_v2::api::PullerStateProperties;
use crate::machines::winder_v2::api::SpoolAutomaticActionStateProperties;
use crate::machines::winder_v2::api::SpoolSpeedControllerStateProperties;
use crate::machines::winder_v2::api::StateProperties;
use crate::machines::winder_v2::api::TensionArmStateProperties;
use crate::machines::winder_v2::api::TraverseStateProperties;
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

impl MachineBuild for WinderV1 {
    fn build(ctx: BuildContext) -> BuildResult<Self> {
        let ident = ctx.ident_unique().identification;

        if ident == WinderV1::MACHINE_IDENTIFICATION {
            Self::new_normal(ctx)
        } else if ident == WinderV1::MACHINE_IDENTIFICATION_7031_SPOOL {
            Self::new_spool_7031(ctx)
        } else {
            Err(BuildError::UnexpectedMachineIdentification)
        }
    }
}

impl WinderV1 {
    fn new_normal(ctx: BuildContext) -> BuildResult<Self> {
        let interface = ctx.get_ethercat_interface()?;

        init_ek1100(&ctx)?;
        let el2002 = init_el2002(&ctx)?;
        let el7041 = init_el7041(&ctx, interface.clone())?;
        let el7031 = init_el7031(&ctx, interface.clone())?;
        let el7031_0030 = init_el7031_0030(&ctx, interface.clone())?;

        Self::new_any(
            ctx,
            el7031,
            el7031_0030.clone(),
            el7041,
            TensionArm::new(el7031_0030.clone()),
            el2002,
        )
    }

    fn new_spool_7031(ctx: BuildContext) -> BuildResult<Self> {
        let interface = ctx.get_ethercat_interface()?;

        init_ek1100(&ctx)?;
        let el2002 = init_el2002(&ctx)?;
        let el7031_0030 = init_el7031_0030(&ctx, interface.clone())?;
        let el7031 = init_el7031(&ctx, interface.clone())?;
        let el7031_0030_spool = init_el7031_0030_spool(&ctx, interface.clone())?;

        Self::new_any(
            ctx,
            el7031,
            el7031_0030.clone(),
            el7031_0030_spool,
            TensionArm::new(el7031_0030.clone()),
            el2002,
        )
    }

    fn new_any(
        mut ctx: BuildContext,
        traverse: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        puller: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        spool: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        tension_arm: TensionArm,
        laser: Rc<RefCell<dyn DigitalOutputDevice>>,
    ) -> BuildResult<Self> {
        init_commands(&mut ctx)?;

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
            ),
            mode: Winder2Mode::Standby,
            spool_mode: SpoolMode::Standby,
            traverse_mode: TraverseMode::Standby,
            puller_mode: PullerMode::Standby,
            spool_speed_controller: SpoolSpeedController::new(),
            spool_step_converter: AngularStepConverter::new(200),
            spool_automatic_action: SpoolAutomaticAction {
                progress: Length::ZERO,
                progress_last_check: Instant::now(),
                target_length: Length::new::<meter>(250.0),
                mode: SpoolAutomaticActionMode::NoAction,
            },
            puller_speed_controller: PullerSpeedController::new(
                Velocity::new::<meter_per_minute>(1.0),
                LinearStepConverter::from_diameter(
                    200,                            // Assuming 200 steps per revolution for the puller stepper,
                    Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                ),
            ),
            config_props: init_config_properties(&mut ctx)?,
            state_props: init_state_properties(&mut ctx)?,
            measurements: init_measurements(&mut ctx)?,
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
fn init_config_properties(ctx: &mut BuildContext) -> BuildResult<ConfigProperties> {
    Ok(ConfigProperties {
        traverse_limit_inner: ctx
            .config::<millimeter>("traverse.limit_inner")
            .on_changed(WinderV1::on_traverse_limit_inner_changed)
            .default(22.0)
            .register()?,

        traverse_limit_outer: ctx
            .config::<millimeter>("traverse.limit_outer")
            .on_changed(WinderV1::on_traverse_limit_outer_changed)
            .default(92.0)
            .register()?,

        traverse_step_size: ctx
            .config::<millimeter>("traverse.step_size")
            .on_changed(WinderV1::on_traverse_step_size_changed)
            .default(1.75)
            .register()?,

        traverse_padding: ctx
            .config::<millimeter>("traverse.padding")
            .on_changed(WinderV1::on_traverse_padding_changed)
            .default(0.88)
            .register()?,

        puller_regulation_mode: ctx
            .config::<PullerRegulationMode>("puller.regulation_mode")
            .on_changed(WinderV1::on_puller_regulation_mode_changed)
            .default(PullerRegulationMode::Speed)
            .register()?,

        puller_target_speed: ctx
            .config::<meter_per_minute>("puller.target_speed")
            .on_changed(WinderV1::on_puller_target_speed_changed)
            .default(1.0)
            .register()?,

        puller_forward: ctx
            .config::<bool>("puller.forward")
            .on_changed(WinderV1::on_puller_forward_changed)
            .default(true)
            .register()?,

        puller_gear_ratio: ctx
            .config::<GearRatio>("puller.gear_ratio")
            .on_changed(WinderV1::on_puller_gear_ratio_changed)
            .default(GearRatio::OneToOne)
            .register()?,

        puller_adaptive_max_speed_change_percent: ctx
            .config::<f64>("puller.adaptive.max_speed_change_percent")
            .on_changed(WinderV1::on_puller_adaptive_max_speed_change_percent_changed)
            .default(33.0)
            .register()?,

        puller_adaptive_adjustment_interval: ctx
            .config::<meter>("puller.adaptive.adjustment_interval")
            .on_changed(WinderV1::on_puller_adaptive_adjustment_interval_changed)
            .default(0.5)
            .register()?,

        puller_adaptive_step_percent: ctx
            .config::<f64>("puller.adaptive.step_percent")
            .on_changed(WinderV1::on_puller_adaptive_step_percent_changed)
            .default(3.3)
            .register()?,

        puller_adaptive_accepted_difference: ctx
            .config::<millimeter>("puller.adaptive.accepted_difference")
            .on_changed(WinderV1::on_puller_adaptive_accepted_difference_changed)
            .default(0.01)
            .register()?,

        spool_regulation_mode: ctx
            .config::<SpoolSpeedControllerType>("spool.regulation_mode")
            .on_changed(WinderV1::on_spool_regulation_mode_changed)
            .default(SpoolSpeedControllerType::default())
            .register()?,

        spool_min_speed: ctx
            .config::<meter_per_minute>("spool.min_speed")
            .on_changed(WinderV1::on_spool_min_speed_changed)
            .default(0.0)
            .register()?,

        spool_max_speed: ctx
            .config::<meter_per_minute>("spool.max_speed")
            .on_changed(WinderV1::on_spool_max_speed_changed)
            .default(0.0)
            .register()?,

        spool_forward: ctx
            .config::<bool>("spool.forward")
            .on_changed(WinderV1::on_spool_forward_changed)
            .default(true)
            .register()?,

        spool_adaptive_tension_target: ctx
            .config::<f64>("spool.adaptive.tension_target")
            .on_changed(WinderV1::on_spool_adaptive_tension_target_changed)
            .default(0.7)
            .register()?,

        spool_adaptive_radius_learning_rate: ctx
            .config::<f64>("spool.adaptive.radius_learning_rate")
            .on_changed(WinderV1::on_spool_adaptive_radius_learning_rate_changed)
            .default(0.5)
            .register()?,

        spool_adaptive_max_speed_multiplier: ctx
            .config::<f64>("spool.adaptive.max_speed_multiplier")
            .on_changed(WinderV1::on_spool_adaptive_max_speed_multiplier_changed)
            .default(4.0)
            .register()?,

        spool_adaptive_acceleration_factor: ctx
            .config::<f64>("spool.adaptive.acceleration_factor")
            .on_changed(WinderV1::on_spool_adaptive_acceleration_factor_changed)
            .default(0.2)
            .register()?,

        spool_adaptive_deacceleration_urgency_multiplier: ctx
            .config::<f64>("spool.adaptive.deacceleration_urgency_multiplier")
            .on_changed(WinderV1::on_spool_adaptive_deacceleration_urgency_multiplier_changed)
            .default(15.0)
            .register()?,

        spool_automatic_required_length: ctx
            .config::<meter>("spool_automatic.required_meters")
            .on_changed(WinderV1::on_spool_automatic_required_length_changed)
            .default(250.0)
            .register()?,

        spool_automatic_action: ctx
            .config::<SpoolAutomaticActionMode>("spool_automatic.action")
            .on_changed(WinderV1::on_spool_automatic_action_changed)
            .default(SpoolAutomaticActionMode::NoAction)
            .register()?,
    })
}

fn init_state_properties(ctx: &mut BuildContext) -> BuildResult<StateProperties> {
    Ok(StateProperties {
        traverse_state: TraverseStateProperties {
            limit_inner: ctx.state::<millimeter>("traverse.limit_inner").register()?,
            limit_outer: ctx.state::<millimeter>("traverse.limit_outer").register()?,
            is_going_in: ctx.state::<bool>("traverse.is_going_in").register()?,
            is_going_out: ctx.state::<bool>("traverse.is_going_out").register()?,
            is_homed: ctx.state::<bool>("traverse.is_homed").register()?,
            is_going_home: ctx.state::<bool>("traverse.is_going_home").register()?,
            is_traversing: ctx.state::<bool>("traverse.is_traversing").register()?,
            laserpointer: ctx.state::<bool>("traverse.laserpointer").register()?,
            step_size: ctx.state::<millimeter>("traverse.step_size").register()?,
            padding: ctx.state::<millimeter>("traverse.padding").register()?,
            can_go_in: ctx.state::<bool>("traverse.can_go_in").register()?,
            can_go_out: ctx.state::<bool>("traverse.can_go_out").register()?,
            can_go_home: ctx.state::<bool>("traverse.can_go_home").register()?,
        },

        puller_state: PullerStateProperties {
            regulation: ctx
                .state::<PullerRegulationMode>("puller.regulation")
                .register()?,
            target_speed: ctx
                .state::<meter_per_minute>("puller.target_speed")
                .register()?,
            forward: ctx.state::<bool>("puller.forward").register()?,
            gear_ratio: ctx.state::<GearRatio>("puller.gear_ratio").register()?,
            adaptive_speed_delta_max: ctx
                .state::<f64>("puller.adaptive_speed_delta_max")
                .register()?,
            adaptive_adjustment_distance: ctx
                .state::<millimeter>("puller.adaptive_adjustment_distance")
                .register()?,
            adaptive_change_per_step: ctx
                .state::<f64>("puller.adaptive_change_per_step")
                .register()?,
            allowed_diameter_deviation: ctx
                .state::<millimeter>("puller.allowed_diameter_deviation")
                .register()?,
        },

        spool_automatic_action_state: SpoolAutomaticActionStateProperties {
            spool_required_meters: ctx
                .state::<meter>("spool_automatic_action.spool_required_meters")
                .register()?,
            spool_automatic_action_mode: ctx
                .state::<SpoolAutomaticActionMode>(
                    "spool_automatic_action.spool_automatic_action_mode",
                )
                .register()?,
        },

        mode_state: ModeStateProperties {
            mode: ctx.state::<Mode>("mode.mode").register()?,
            can_wind: ctx.state::<bool>("mode.can_wind").register()?,
        },

        tension_arm_state: TensionArmStateProperties {
            zeroed: ctx.state::<bool>("tension_arm.zeroed").register()?,
        },

        spool_speed_controller_state: SpoolSpeedControllerStateProperties {
            regulation_mode: ctx
                .state::<SpoolSpeedControllerType>("spool_speed_controller.regulation_mode")
                .register()?,
            minmax_min_speed: ctx
                .state::<revolution_per_minute>("spool_speed_controller.minmax_min_speed")
                .register()?,
            minmax_max_speed: ctx
                .state::<revolution_per_minute>("spool_speed_controller.minmax_max_speed")
                .register()?,
            adaptive_tension_target: ctx
                .state::<f64>("spool_speed_controller.adaptive_tension_target")
                .register()?,
            adaptive_radius_learning_rate: ctx
                .state::<f64>("spool_speed_controller.adaptive_radius_learning_rate")
                .register()?,
            adaptive_max_speed_multiplier: ctx
                .state::<f64>("spool_speed_controller.adaptive_max_speed_multiplier")
                .register()?,
            adaptive_acceleration_factor: ctx
                .state::<f64>("spool_speed_controller.adaptive_acceleration_factor")
                .register()?,
            adaptive_deacceleration_urgency_multiplier: ctx
                .state::<f64>("spool_speed_controller.adaptive_deacceleration_urgency_multiplier")
                .register()?,
            forward: ctx
                .state::<bool>("spool_speed_controller.forward")
                .register()?,
        },
    })
}

fn init_measurements(ctx: &mut BuildContext) -> BuildResult<Measurements> {
    Ok(Measurements {
        traverse_position: ctx
            .measurement::<Option<millimeter>>("traverse.position")
            .register()?,

        puller_speed: ctx
            .measurement::<meter_per_minute>("puller.speed")
            .register()?,

        spool_rpm: ctx
            .measurement::<revolution_per_minute>("spool.rpm")
            .register()?,

        tension_arm_angle: ctx.measurement::<degree>("tension_arm.angle").register()?,
        spool_progress: ctx.measurement::<meter>("spool.progress").register()?,
    })
}

fn init_commands(ctx: &mut BuildContext) -> BuildResult<()> {
    // --- modes ---
    ctx.command("enter_standby_mode")
        .execute(WinderV1::cmd_enter_standby_mode)
        .register()?;

    ctx.command("enter_hold_mode")
        .execute(WinderV1::cmd_enter_hold_mode)
        .register()?;

    ctx.command("enter_pull_mode")
        .execute(WinderV1::cmd_enter_pull_mode)
        .register()?;

    ctx.command("enter_wind_mode")
        .execute(WinderV1::cmd_enter_wind_mode)
        .register()?;

    // --- traverse goto ---
    ctx.command("traverse.goto_home")
        .execute(WinderV1::cmd_traverse_goto_home)
        .register()?;

    ctx.command("traverse.goto_limit_inner")
        .execute(WinderV1::cmd_traverse_goto_limit_inner)
        .register()?;

    ctx.command("traverse.goto_limit_outer")
        .execute(WinderV1::cmd_traverse_goto_limit_outer)
        .register()?;

    // --- traverse laser ---
    ctx.command("traverse.laserpointer.enable")
        .execute(WinderV1::cmd_traverse_laser_enable)
        .register()?;

    ctx.command("traverse.laserpointer.disable")
        .execute(WinderV1::cmd_traverse_laser_disable)
        .register()?;

    // --- spool ---
    ctx.command("spool.reset_progress")
        .execute(WinderV1::cmd_spool_reset_progress)
        .register()?;

    // --- tension arm ---
    ctx.command("tension_arm.set_zero")
        .execute(WinderV1::cmd_tension_arm_set_zero)
        .register()?;

    Ok(())
}
