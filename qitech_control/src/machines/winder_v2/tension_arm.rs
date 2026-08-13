use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact;
use qitech_framework::machine::ActErrorKind;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::analog_input::physical::AnalogInputValue;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::angle::degree;
use qitech_lib::units::angle::revolution;
use qitech_lib::units::electric_potential::volt;
use qitech_lib::units::f64::*;

use crate::machines::winder_v2::WinderV1;

pub struct TensionArm {
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    zero: StateProperty<Option<Angle>>,
    angle: Measurement<Angle>,
}

impl TensionArm {
    pub fn init<const VARIANT: usize>(
        ctx: &mut BuildContext,
        device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    ) -> BuildResult<Self> {
        ctx.command("tension_arm.set_zero")
            .execute(|m: &mut WinderV1<VARIANT>| {
                m.tension_arm.set_zero().map_err(|e| ActError {
                    kind: e,
                    impact: ActErrorImpact::Degraded,
                })
            })
            .build()?;

        Ok(Self {
            device,
            zero: ctx
                .state::<Option<revolution>>("tension_arm.zero")
                .build()?,
            angle: ctx.measurement::<revolution>("tension_arm.angle").build()?,
        })
    }
}

impl TensionArm {
    pub fn zeroed(&self) -> bool {
        self.zero.get().is_some()
    }

    pub fn angle(&self) -> Angle {
        self.angle.get()
    }

    pub fn update(&mut self) -> Result<(), ActErrorKind> {
        self.update_angle()
    }

    fn set_zero(&mut self) -> Result<(), ActErrorKind> {
        let angle = self.raw_angle()?;
        self.zero.set(Some(angle));
        Ok(())
    }
}

// --- utils ---
impl TensionArm {
    fn update_angle(&mut self) -> Result<(), ActErrorKind> {
        let raw = self.raw_angle()?;
        let zero = self.zero.get().unwrap_or(Angle::new::<degree>(0.0));

        let angle = if raw < zero {
            // We've wrapped around, so add a full revolution
            raw + Angle::new::<revolution>(1.0) - zero
        } else {
            // Normal case
            raw - zero
        };

        let angle = if angle >= Angle::new::<degree>(270.0) {
            angle - Angle::new::<degree>(360.0)
        } else {
            angle
        };

        self.angle.set(angle);
        Ok(())
    }

    fn raw_angle(&self) -> Result<Angle, ActErrorKind> {
        let volts = self.read_volts()?;
        Ok(self.volts_to_angle(volts))
    }

    /// Read the normalized voltage from the analog input.
    fn read_volts(&self) -> Result<f64, ActErrorKind> {
        let analog_input = self.device.borrow();

        let range = analog_input
            .analog_input_range()
            .ok_or_else(|| ActErrorKind::Custom("No input range supplied".to_string()))?;

        let value = analog_input
            .get_analog_input(0)
            .map_err(|e| ActErrorKind::Custom(e.to_string()))?
            .get_physical(&range);

        match value {
            AnalogInputValue::Potential(v) => Ok(v.get::<volt>()),
            _ => panic!("Expected a potential value"),
        }
    }

    fn volts_to_angle(&self, volts: f64) -> Angle {
        const MIN_VOLTAGE: f64 = 0.0;
        const MAX_VOLTAGE: f64 = 5.0;
        const FULL_REVOLUTION: f64 = 1.0;

        let revolutions = (volts - MIN_VOLTAGE) / (MAX_VOLTAGE - MIN_VOLTAGE);
        Angle::new::<revolution>(revolutions % FULL_REVOLUTION)
    }
}
