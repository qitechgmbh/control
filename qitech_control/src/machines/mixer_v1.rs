use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework::Machine;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildError;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::RemoteProperty;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine::SubscribeContext;
use qitech_framework::machine::SubscribeResult;
use qitech_framework::machine_build;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::ek1100::EK1100;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el2002::EL2002;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::EL7041_0052;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el7041_0052::coe::EL7041_0052Configuration;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::ethercat_hal::shared_config;
use qitech_lib::ethercat_hal::shared_config::el70x1::EL70x1OperationMode;
use qitech_lib::ethercat_hal::shared_config::el70x1::StmMotorConfiguration;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::angular_velocity::revolution_per_minute;

const MIXING_MOTOR_PORT: usize = 0;
const HOPPER_PORT: usize = 0;
const MOTOR_FULL_STEPS_PER_REV: f64 = 200.0;

pub struct ExtruderSubscription {
    ident: MachineInstanceIdentification,
    rpm: RemoteProperty<AngularVelocity>,
}

#[derive(Machine)]
pub struct MixerV1 {
    // --- hardware ---
    mixing_motor: Rc<RefCell<EL2002>>,
    hopper_a: Rc<RefCell<EL7041_0052>>,
    hopper_b: Rc<RefCell<EL7041_0052>>,

    // --- config ---
    extruder_kg_per_rpm: ConfigProperty<f64>,
    hopper_a_target_rpm: ConfigProperty<f64>,
    hopper_a_forward: ConfigProperty<bool>,
    hopper_a_dosing_percent: ConfigProperty<f64>,
    hopper_a_calibration_steps_per_kgh: ConfigProperty<f64>,
    hopper_b_target_rpm: ConfigProperty<f64>,
    hopper_b_forward: ConfigProperty<bool>,
    hopper_b_dosing_percent: ConfigProperty<f64>,
    hopper_b_calibration_steps_per_kgh: ConfigProperty<f64>,

    // --- state ---
    mixing_motor_on: StateProperty<bool>,
    hopper_a_enabled: StateProperty<bool>,
    hopper_a_ready: StateProperty<bool>,
    hopper_a_error: StateProperty<bool>,
    hopper_b_enabled: StateProperty<bool>,
    hopper_b_ready: StateProperty<bool>,
    hopper_b_error: StateProperty<bool>,

    // --- subscriptions ---
    extruder_subscription: Option<ExtruderSubscription>,
}

impl MachineBuild for MixerV1 {
    #[machine_build(MixerV1)]
    fn build(ctx: &mut BuildContext<'_>) -> BuildResult<Self> {
        let interface = ctx.get_ethercat_interface()?;

        ctx.find_ethercat_device_and_addr::<EK1100>(0)?;

        let mixing_motor = ctx.find_ethercat_device::<EL2002>(1)?;
        let hopper_a = init_stepper(ctx, interface.clone(), 2)?;
        let hopper_b = init_stepper(ctx, interface.clone(), 3)?;

        let hopper_a_target_rpm = ctx
            .config::<f64>("hopper_a.target_rpm")
            .default(0.0)
            .minimum(0.0)
            .on_external_changed(Self::push_hopper_a_speed)
            .build()?;
        let hopper_a_forward = ctx
            .config::<bool>("hopper_a.forward")
            .default(true)
            .on_external_changed(Self::push_hopper_a_speed)
            .build()?;
        let hopper_a_dosing_percent = ctx
            .config::<f64>("hopper_a.dosing_percent")
            .default(0.0)
            .minimum(0.0)
            .on_external_changed(Self::push_hopper_a_ratio_speed)
            .build()?;
        let hopper_a_calibration_steps_per_kgh = ctx
            .config::<f64>("hopper_a.calibration_steps_per_kgh")
            .default(34.47)
            .minimum(0.0)
            .on_external_changed(Self::push_hopper_a_ratio_speed)
            .build()?;

        let hopper_b_target_rpm = ctx
            .config::<f64>("hopper_b.target_rpm")
            .default(0.0)
            .minimum(0.0)
            .on_external_changed(Self::push_hopper_b_speed)
            .build()?;
        let hopper_b_forward = ctx
            .config::<bool>("hopper_b.forward")
            .default(true)
            .on_external_changed(Self::push_hopper_b_speed)
            .build()?;
        let hopper_b_dosing_percent = ctx
            .config::<f64>("hopper_b.dosing_percent")
            .default(0.0)
            .minimum(0.0)
            .on_external_changed(Self::push_hopper_b_ratio_speed)
            .build()?;
        let hopper_b_calibration_steps_per_kgh = ctx
            .config::<f64>("hopper_b.calibration_steps_per_kgh")
            .default(6.37)
            .minimum(0.0)
            .on_external_changed(Self::push_hopper_b_ratio_speed)
            .build()?;

        let extruder_kg_per_rpm = ctx
            .config::<f64>("extruder_kg_per_rpm")
            .default(0.1)
            .minimum(0.0)
            .on_external_changed(Self::push_ratio_speeds)
            .build()?;

        ctx.command("mixing_motor.start")
            .execute(|m: &mut Self| m.set_mixing_motor(true))
            .build()?;
        ctx.command("mixing_motor.stop")
            .execute(|m: &mut Self| m.set_mixing_motor(false))
            .build()?;

        ctx.command("hopper_a.enable")
            .execute(|m: &mut Self| m.set_hopper_a_enabled(true))
            .build()?;
        ctx.command("hopper_a.disable")
            .execute(|m: &mut Self| m.set_hopper_a_enabled(false))
            .build()?;

        ctx.command("hopper_b.enable")
            .execute(|m: &mut Self| m.set_hopper_b_enabled(true))
            .build()?;
        ctx.command("hopper_b.disable")
            .execute(|m: &mut Self| m.set_hopper_b_enabled(false))
            .build()?;

        Ok(Self {
            mixing_motor,
            hopper_a,
            hopper_b,
            extruder_kg_per_rpm,
            hopper_a_target_rpm,
            hopper_a_forward,
            hopper_a_dosing_percent,
            hopper_a_calibration_steps_per_kgh,
            hopper_b_target_rpm,
            hopper_b_forward,
            hopper_b_dosing_percent,
            hopper_b_calibration_steps_per_kgh,
            mixing_motor_on: ctx.state::<bool>("mixing_motor_on").build()?,
            hopper_a_enabled: ctx.state::<bool>("hopper_a_enabled").build()?,
            hopper_a_ready: ctx.state::<bool>("hopper_a_ready").build()?,
            hopper_a_error: ctx.state::<bool>("hopper_a_error").build()?,
            hopper_b_enabled: ctx.state::<bool>("hopper_b_enabled").build()?,
            hopper_b_ready: ctx.state::<bool>("hopper_b_ready").build()?,
            hopper_b_error: ctx.state::<bool>("hopper_b_error").build()?,
            extruder_subscription: None,
        })
    }
}

impl Machine for MixerV1 {
    fn act(&mut self, _dt: Duration) -> ActResult {
        if let Ok(input) = self.hopper_a.borrow().get_input(HOPPER_PORT) {
            self.hopper_a_ready.set(input.ready);
            self.hopper_a_error.set(input.error);
        }

        if let Ok(input) = self.hopper_b.borrow().get_input(HOPPER_PORT) {
            self.hopper_b_ready.set(input.ready);
            self.hopper_b_error.set(input.error);
        }

        if self.extruder_subscription.is_some() {
            Self::push_ratio_speeds(self)?;
        }

        Ok(())
    }

    fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        self.extruder_subscription = Some(ExtruderSubscription {
            ident: ctx.provider(),
            rpm: ctx.measurement("motor.rpm")?,
        });

        Ok(())
    }

    fn unsubscribe(&mut self, ident: MachineInstanceIdentification) {
        if let Some(sub) = &mut self.extruder_subscription
            && sub.ident == ident
        {
            self.extruder_subscription = None;
        }
    }
}

impl MixerV1 {
    fn set_mixing_motor(&mut self, on: bool) -> ActResult {
        self.mixing_motor_on.set(on);
        self.mixing_motor
            .borrow_mut()
            .set_output(MIXING_MOTOR_PORT, on);
        Ok(())
    }

    fn set_hopper_a_enabled(&mut self, enabled: bool) -> ActResult {
        self.hopper_a_enabled.set(enabled);
        self.hopper_a.borrow_mut().set_enabled(HOPPER_PORT, enabled);
        Ok(())
    }

    fn set_hopper_b_enabled(&mut self, enabled: bool) -> ActResult {
        self.hopper_b_enabled.set(enabled);
        self.hopper_b.borrow_mut().set_enabled(HOPPER_PORT, enabled);
        Ok(())
    }

    fn push_hopper_a_speed(m: &mut Self) -> ActResult {
        let magnitude = m.hopper_a_target_rpm.get() * MOTOR_FULL_STEPS_PER_REV / 60.0;
        let signed = if m.hopper_a_forward.get() {
            magnitude
        } else {
            -magnitude
        };
        let _ = m.hopper_a.borrow_mut().set_speed(HOPPER_PORT, signed);
        Ok(())
    }

    fn push_hopper_b_speed(m: &mut Self) -> ActResult {
        let magnitude = m.hopper_b_target_rpm.get() * MOTOR_FULL_STEPS_PER_REV / 60.0;
        let signed = if m.hopper_b_forward.get() {
            magnitude
        } else {
            -magnitude
        };
        let _ = m.hopper_b.borrow_mut().set_speed(HOPPER_PORT, signed);
        Ok(())
    }

    fn push_ratio_speeds(m: &mut Self) -> ActResult {
        Self::push_hopper_a_ratio_speed(m)?;
        Self::push_hopper_b_ratio_speed(m)
    }

    fn current_extruder_output_rate(&self) -> f64 {
        match &self.extruder_subscription {
            Some(sub) => sub.rpm.get_as::<revolution_per_minute>() * self.extruder_kg_per_rpm.get(),
            None => 0.0,
        }
    }

    fn push_hopper_a_ratio_speed(m: &mut Self) -> ActResult {
        let masterbatch_kg_h =
            m.current_extruder_output_rate() * m.hopper_a_dosing_percent.get() / 100.0;
        let magnitude = masterbatch_kg_h * m.hopper_a_calibration_steps_per_kgh.get();
        let signed = if m.hopper_a_forward.get() {
            magnitude
        } else {
            -magnitude
        };
        let _ = m.hopper_a.borrow_mut().set_speed(HOPPER_PORT, signed);
        Ok(())
    }

    fn push_hopper_b_ratio_speed(m: &mut Self) -> ActResult {
        let masterbatch_kg_h =
            m.current_extruder_output_rate() * m.hopper_b_dosing_percent.get() / 100.0;
        let magnitude = masterbatch_kg_h * m.hopper_b_calibration_steps_per_kgh.get();
        let signed = if m.hopper_b_forward.get() {
            magnitude
        } else {
            -magnitude
        };
        let _ = m.hopper_b.borrow_mut().set_speed(HOPPER_PORT, signed);
        Ok(())
    }
}

fn init_stepper(
    ctx: &BuildContext,
    interface: EtherCATThreadChannel,
    role: u16,
) -> BuildResult<Rc<RefCell<EL7041_0052>>> {
    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7041_0052>(role)?;

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
