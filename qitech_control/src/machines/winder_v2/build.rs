use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildError;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::MachineBuild;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
pub use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::ek1100::EK1100;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el2002::EL2002;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::EL7031;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::coe::EL7031Configuration;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031::pdo::EL7031PredefinedPdoAssignment;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::EL7031_0030;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::coe::EL7031_0030Configuration;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::EL7041_0052;
pub use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::coe::EL7041_0052Configuration;
pub use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
pub use qitech_lib::ethercat_hal::shared_config;
pub use qitech_lib::ethercat_hal::shared_config::el70x1::EL70x1OperationMode;
pub use qitech_lib::ethercat_hal::shared_config::el70x1::StmMotorConfiguration;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::length::meter;

use crate::machines::winder_v2::LaserPointer;
use crate::machines::winder_v2::Puller;
use crate::machines::winder_v2::Spool;
use crate::machines::winder_v2::SpoolAutomaticAction;
use crate::machines::winder_v2::TensionArm;
use crate::machines::winder_v2::Traverse;
use crate::machines::winder_v2::VARIANT_7031_SPOOL;
use crate::machines::winder_v2::VARIANT_REGULAR;
use crate::machines::winder_v2::WinderV1;
use crate::machines::winder_v2::types::AutomaticActionSpoolAction;
use crate::machines::winder_v2::types::Mode;

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
            el7031_0030,
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
            el7031_0030,
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
        tension_arm: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        laser_pointer: Rc<RefCell<dyn DigitalOutputDevice>>,
    ) -> BuildResult<Self> {
        let laser_pointer = LaserPointer::init::<VARIANT>(ctx, laser_pointer)?;
        let tension_arm = TensionArm::init::<VARIANT>(ctx, tension_arm)?;
        let traverse = Traverse::init::<VARIANT>(ctx, traverse)?;
        let puller = Puller::init::<VARIANT>(ctx, puller)?;
        let spool = Spool::init::<VARIANT>(ctx, spool)?;

        // --- mode transition ---
        ctx.command("mode.standby")
            .execute(|m: &mut Self| m.set_mode(Mode::Standby))
            .build()?;

        ctx.command("mode.hold")
            .execute(|m: &mut Self| m.set_mode(Mode::Hold))
            .build()?;

        ctx.command("mode.pull")
            .execute(|m: &mut Self| m.set_mode(Mode::Pull))
            .build()?;

        ctx.command("mode.wind")
            .can_execute(Self::can_wind)
            .execute(|m: &mut Self| m.set_mode(Mode::Wind))
            .build()?;

        // --- construct machine ---
        Ok(Self {
            spool,
            puller,
            traverse,
            tension_arm,
            laser_pointer,
            mode: ctx.state::<Mode>("mode").build()?,
            spool_automatic_action: SpoolAutomaticAction {
                progress: Length::ZERO,
                progress_last_check: Instant::now(),
                target_length: ctx
                    .config::<meter>("spool_automatic.required_meters")
                    .default(250.0)
                    .build()?,
                mode: ctx
                    .config::<AutomaticActionSpoolAction>("spool_automatic.action")
                    .default(AutomaticActionSpoolAction::NoAction)
                    .build()?,
            },
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
        .write_config(interface.clone(), addr, &config)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

    interface
        .enable_dc_sync0(addr)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

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
        .write_config(interface.clone(), addr, &config)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;
    interface
        .enable_dc_sync0(addr)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

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
        .write_config(interface.clone(), addr, &config)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

    interface
        .enable_dc_sync0(addr)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

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
        .write_config(interface.clone(), addr, &config)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

    interface
        .enable_dc_sync0(addr)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

    Ok(dev)
}
