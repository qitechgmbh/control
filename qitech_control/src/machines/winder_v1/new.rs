mod winder2_imports {
    pub use std::time::Instant;

    pub use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::ek1100::EK1100;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el2002::EL2002;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::EL7031;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::coe::EL7031Configuration;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::pdo::EL7031PredefinedPdoAssignment;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::EL7031_0030;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::coe::EL7031_0030Configuration;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::{self};
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::EL7041_0052;
    pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::coe::EL7041_0052Configuration;
    pub use qitech_lib::ethercat_hal::shared_config;
    pub use qitech_lib::ethercat_hal::shared_config::el70x1::EL70x1OperationMode;
    pub use qitech_lib::ethercat_hal::shared_config::el70x1::StmMotorConfiguration;
    pub use qitech_lib::units::ConstZero;
    pub use qitech_lib::units::f64::*;
    pub use qitech_lib::units::length::centimeter;
    pub use qitech_lib::units::length::meter;
    pub use qitech_lib::units::length::millimeter;
    pub use qitech_lib::units::velocity::meter_per_minute;

    pub use super::super::WinderV1;
    pub use super::super::Winder2Mode;
    pub use super::super::puller_speed_controller::PullerSpeedController;
    pub use super::super::spool_speed_controller::SpoolSpeedController;
    pub use super::super::tension_arm::TensionArm;
    pub use super::super::traverse_controller::TraverseController;
    pub use crate::converters::angular_step_converter::AngularStepConverter;
    pub use crate::converters::linear_step_converter::LinearStepConverter;
}

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::error::BuildError;
use qitech_framework::machine::error::BuildResult;
use qitech_lib::units::angle::degree;
use qitech_lib::units::angular_velocity::revolution_per_minute;
pub use winder2_imports::*;

use crate::machines::winder_v1::SpoolAutomaticAction;
use crate::machines::winder_v1::api::Configurations;
use crate::machines::winder_v1::api::GearRatio;
use crate::machines::winder_v1::api::Measurements;
use crate::machines::winder_v1::api::Mode;
use crate::machines::winder_v1::api::ModeState;
use crate::machines::winder_v1::api::PullerRegulationMode;
use crate::machines::winder_v1::api::PullerState;
use crate::machines::winder_v1::api::SpoolAutomaticActionMode;
use crate::machines::winder_v1::api::SpoolAutomaticActionState;
use crate::machines::winder_v1::api::SpoolSpeedControllerState;
use crate::machines::winder_v1::api::States;
use crate::machines::winder_v1::api::TensionArmState;
use crate::machines::winder_v1::api::TraverseState;
use crate::machines::winder_v1::spool_speed_controller::SpoolSpeedControllerType;

impl MachineBuild for WinderV1 {
    fn build(ctx: BuildContext) -> BuildResult<Self> {
        let ident = ctx.ident_unique().identification;

        if ident == WinderV1::MACHINE_IDENTIFICATION {
            Self::new_normal(ctx)
        } else if ident == WinderV1::MACHINE_IDENTIFICATION_7031_SPOOL {
            Self::new_winder_spool_7031(ctx)
        } else {
            Err(BuildError::UnexpectedMachineIdentification)
        }
    }
}

impl WinderV1 {
    fn new_normal(mut ctx: BuildContext) -> BuildResult<Self> {
        let _ek1100 = ctx.find_ethercat_device_and_addr::<EK1100>(0)?;
        let el2002 = ctx.find_ethercat_device_and_addr::<EL2002>(1)?;
        let el7041 = ctx.find_ethercat_device_and_addr::<EL7041_0052>(2)?;
        let el7031 = ctx.find_ethercat_device_and_addr::<EL7031>(3)?;
        let el7031_0030 = ctx.find_ethercat_device_and_addr::<EL7031_0030>(4)?;

        let mode = Winder2Mode::Standby;
        let interface = ctx.get_ethercat_interface()?;

        // Role 4: Stepper Puller EL7031-0030
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

        let mut b = el7031_0030.0.borrow_mut();
        (*b).write_config(interface.clone(), el7031_0030.1, &el7031_0030_config)?;
        drop(b);
        interface.enable_dc_sync0(el7031_0030.1)?;

        // Role 3: Stepper Traverse EL7031
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
        let mut b = el7031.0.borrow_mut();
        (*b).write_config(interface.clone(), el7031.1, &el7031_config)?;
        drop(b);
        interface.enable_dc_sync0(el7031.1)?;

        // Role 2: Stepper Spool EL7041-0052
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
        let mut b = el7041.0.borrow_mut();
        (*b).write_config(interface.clone(), el7041.1, &el7041_config)?;
        drop(b);
        interface.enable_dc_sync0(el7041.1)?;

        Ok(Self {
            traverse: el7031.0,
            puller: el7031_0030.0.clone(),
            spool: el7041.0,
            laser: el2002.0,
            tension_arm: TensionArm::new(el7031_0030.0.clone()),
            mode: mode.clone(),
            spool_step_converter: AngularStepConverter::new(200),
            spool_speed_controller: SpoolSpeedController::new(),
            spool_mode: mode.clone().into(),
            traverse_mode: mode.clone().into(),
            puller_mode: mode.into(),
            puller_speed_controller: PullerSpeedController::new(
                Velocity::new::<meter_per_minute>(1.0),
                LinearStepConverter::from_diameter(
                    200,                            // Assuming 200 steps per revolution for the puller stepper,
                    Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                ),
            ),
            traverse_controller: TraverseController::new(
                ctx.config::<millimeter>("traverse.limit_inner")
                    .default(22.0)
                    .register()?,
                ctx.config::<millimeter>("traverse.limit_outer")
                    .default(92.0)
                    .register()?,
                ctx.config::<millimeter>("traverse.step_size")
                    .default(1.75)
                    .register()?,
                ctx.config::<millimeter>("traverse.padding")
                    .default(0.88)
                    .register()?,
                64, // Microsteps
            ),
            spool_automatic_action: super::SpoolAutomaticAction {
                progress: Length::ZERO,
                progress_last_check: Instant::now(),
                target_length: ctx.config::<meter>("spool_automatic.required_meters")
                    .default(250.0)
                    .register()?,
                mode: ctx.config::<SpoolAutomaticActionMode>("spool_automatic.action")
                    .default(SpoolAutomaticActionMode::NoAction)
                    .register()?,
            },
            laser_enabled: false,
            laser_subscription: None,

            measurements: init_measurements(&mut ctx)?,
            states: init_states(&mut ctx)?,
            configurations: init_config(&mut ctx)?,
        })
    }

    fn new_winder_spool_7031(mut ctx: BuildContext) -> BuildResult<Self> {
        let _ek1100 = ctx.find_ethercat_device_and_addr::<EK1100>(0)?;
        let el2002 = ctx.find_ethercat_device_and_addr::<EL2002>(1)?;
        let el7031_0030_spool = ctx.find_ethercat_device_and_addr::<EL7031_0030>(2)?;
        let el7031 = ctx.find_ethercat_device_and_addr::<EL7031>(3)?;
        let el7031_0030 = ctx.find_ethercat_device_and_addr::<EL7031_0030>(4)?;

        let mode = Winder2Mode::Standby;
        let interface = ctx.get_ethercat_interface()?;

        // Role 4: Stepper Puller EL7031-0030
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
        let mut b = el7031_0030.0.borrow_mut();
        (*b).write_config(interface.clone(), el7031_0030.1, &el7031_0030_config)?;
        drop(b);
        interface.enable_dc_sync0(el7031_0030.1)?;

        // Role 3: Stepper Traverse EL7031
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
        let mut b = el7031.0.borrow_mut();
        (*b).write_config(interface.clone(), el7031.1, &el7031_config)?;
        drop(b);
        interface.enable_dc_sync0(el7031.1)?;

        // Role 2: Stepper Spool EL7031-0030
        let el7031_0030_spool_config = EL7031_0030Configuration {
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
        let mut b = el7031_0030_spool.0.borrow_mut();
        (*b).write_config(
            interface.clone(),
            el7031_0030_spool.1,
            &el7031_0030_spool_config,
        )?;
        drop(b);
        interface.enable_dc_sync0(el7031_0030_spool.1)?;

        // --- register commands ---
        init_commands(&mut ctx)?;

        Ok(Self {
            traverse: el7031.0,
            puller: el7031_0030.0.clone(),
            spool: el7031_0030_spool.0,
            laser: el2002.0,
            tension_arm: TensionArm::new(el7031_0030.0.clone()),
            mode: mode.clone(),
            spool_step_converter: AngularStepConverter::new(200),
            spool_speed_controller: SpoolSpeedController::new(),
            spool_mode: mode.clone().into(),
            traverse_mode: mode.clone().into(),
            puller_mode: mode.into(),
            puller_speed_controller: PullerSpeedController::new(
                Velocity::new::<meter_per_minute>(1.0),
                LinearStepConverter::from_diameter(
                    200,                            // Assuming 200 steps per revolution for the puller stepper,
                    Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                ),
            ),
            traverse_controller: TraverseController::new(
                ctx.config::<millimeter>("traverse.limit_inner")
                    .default(22.0)
                    .register()?,
                ctx.config::<millimeter>("traverse.limit_outer")
                    .default(92.0)
                    .register()?,
                ctx.config::<millimeter>("traverse.step_size")
                    .default(1.75)
                    .register()?,
                ctx.config::<millimeter>("traverse.padding")
                    .default(0.88)
                    .register()?,
                64, // Microsteps
            ),
            spool_automatic_action: SpoolAutomaticAction {
                progress: Length::ZERO,
                progress_last_check: Instant::now(),
                target_length: ctx.config::<meter>("spool_automatic.required_meters")
                    .default(250.0)
                    .register()?,
                mode: ctx.config::<SpoolAutomaticActionMode>("spool_automatic.action")
                    .default(SpoolAutomaticActionMode::NoAction)
                    .register()?,
            },
            laser_enabled: false,
            laser_subscription: None,

            measurements: init_measurements(&mut ctx)?,
            states: init_states(&mut ctx)?,
            configurations: init_config(&mut ctx)?,
        })
    }
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

fn init_config(ctx: &mut BuildContext) -> BuildResult<Configurations> {
    Ok(Configurations {
        // --- puller ---
        puller_regulation_mode: ctx
            .config::<PullerRegulationMode>("puller.regulation_mode")
            .default(PullerRegulationMode::Speed)
            .register()?,

        puller_target_speed: ctx
            .config::<meter_per_minute>("puller.target_speed")
            .default(1.0)
            .register()?,

        puller_forward: ctx
            .config::<bool>("puller.forward")
            .default(true)
            .register()?,

        puller_gear_ratio: ctx
            .config::<GearRatio>("puller.gear_ratio")
            .default(GearRatio::OneToOne)
            .register()?,

        puller_adaptive_max_speed_change_percent: ctx
            .config::<f64>("puller.adapative.max_speed_change_percent")
            .default(33.0)
            .register()?,

        puller_adaptive_adjustment_interval_meters: ctx
            .config::<meter>("puller.adapative.adjustment_interval")
            .default(0.5)
            .register()?,

        puller_adaptive_step_percent: ctx
            .config::<f64>("puller.adapative.step_percent")
            .default(3.3)
            .register()?,

        puller_adaptive_accepted_difference: ctx
            .config::<millimeter>("puller.adapative.accepted_difference")
            .default(0.01)
            .register()?,

        // --- spool ---
        spool_adaptive_tension_target: ctx
            .config::<f64>("spool.adaptive.tension_target")
            .default(0.7)
            .register()?,

        spool_adaptive_radius_learning_rate: ctx
            .config::<f64>("spool.adaptive.radius_learning_rate")
            .default(0.5)
            .register()?,

        spool_adaptive_max_speed_multiplier: ctx
            .config::<f64>("spool.adaptive.max_speed_multiplier")
            .default(4.0)
            .register()?,

        spool_adaptive_acceleration_factor: ctx
            .config::<f64>("spool.adaptive.acceleration_factor")
            .default(0.2)
            .register()?,

        spool_adaptive_deacceleration_urgency_multiplier: ctx
            .config::<f64>("spool.adaptive.deacceleration_urgency_multiplier")
            .default(15.0)
            .register()?,
    })
}

fn init_states(ctx: &mut BuildContext) -> BuildResult<States> {
    Ok(States {
        traverse_state: TraverseState {
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

        puller_state: PullerState {
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

        spool_automatic_action_state: SpoolAutomaticActionState {
            spool_required_meters: ctx
                .state::<meter>("spool_automatic_action.spool_required_meters")
                .register()?,
            spool_automatic_action_mode: ctx
                .state::<SpoolAutomaticActionMode>(
                    "spool_automatic_action.spool_automatic_action_mode",
                )
                .register()?,
        },

        mode_state: ModeState {
            mode: ctx.state::<Mode>("mode.mode").register()?,
            can_wind: ctx.state::<bool>("mode.can_wind").register()?,
        },

        tension_arm_state: TensionArmState {
            zeroed: ctx.state::<bool>("tension_arm.zeroed").register()?,
        },

        spool_speed_controller_state: SpoolSpeedControllerState {
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

fn init_commands(ctx: &mut BuildContext) -> BuildResult<()> {
    // --- modes ---
    ctx.command("set_mode.standby")
        .execute(WinderV1::cmd_mode_standby)
        .register()?;

    ctx.command("set_mode.hold")
        .execute(WinderV1::cmd_mode_hold)
        .register()?;

    ctx.command("set_mode.pull")
        .execute(WinderV1::cmd_mode_pull)
        .register()?;

    ctx.command("set_mode.wind")
        .execute(WinderV1::cmd_mode_wind)
        .register()?;

    // --- traverse goto ---
    ctx.command("traverse.goto_home")
        .can_execute(WinderV1::can_go_home)
        .execute(WinderV1::cmd_traverse_home)
        .register()?;

    ctx.command("traverse.goto_limit_inner")
        .can_execute(WinderV1::can_go_in)
        .execute(WinderV1::cmd_goto_limit_inner)
        .register()?;

    ctx.command("traverse.goto_limit_outer")
        .can_execute(WinderV1::can_go_out)
        .execute(WinderV1::cmd_goto_limit_outer)
        .register()?;

    // --- traverse laser ---
    ctx.command("traverse.laserpointer.enable")
        .execute(WinderV1::cmd_enable_traverse_laserpointer)
        .register()?;

    ctx.command("traverse.laserpointer.disable")
        .execute(WinderV1::cmd_disable_traverse_laserpointer)
        .register()?;

    // --- tension arm ---
    ctx.command("tension_arm.set_zero")
        .execute(WinderV1::cmd_tension_arm_set_zero)
        .register()?;

    Ok(())
}
