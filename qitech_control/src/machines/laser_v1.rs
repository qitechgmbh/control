use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework::Machine;
use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact;
use qitech_framework::machine::ActErrorKind;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildError;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::EventEmitter;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine_build;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserError;
use qitech_lib::units::Length;
use qitech_lib::units::length::millimeter;

#[derive(Machine)]
pub struct LaserV1 {
    // --- hardware ---
    device: Rc<RefCell<LaserDevice>>,

    // -- config ---
    diameter_target: ConfigProperty<Length>,
    diameter_tolerance_upper: ConfigProperty<Length>,
    diameter_tolerance_lower: ConfigProperty<Length>,

    // --- state ---
    in_tolerance: StateProperty<bool>,

    // --- measurements ---
    diameter: Measurement<Length>,
    diameter_x: Measurement<Option<Length>>,
    diameter_y: Measurement<Option<Length>>,
    roundness: Measurement<Option<f64>>,

    // --- events ---
    out_of_tolerance: EventEmitter<()>,

    // -- misc ---
    request_timer: Duration,
}

impl MachineBuild for LaserV1 {
    #[machine_build(LaserV1)]
    fn build(ctx: &mut BuildContext<'_>) -> Result<Self, BuildError> {
        let device = ctx.get_modbus_rtu_device::<LaserDevice>(0)?;

        let diameter_target = ctx
            .config::<millimeter>("diameter.target")
            .default(1.75)
            .minimum(0.0)
            .maximum(5.0)
            .build()?;

        let diameter_tolerance_lower = ctx
            .config::<millimeter>("diameter.tolerance.lower")
            .default(0.05)
            .minimum(0.0)
            .build()?;

        let diameter_tolerance_upper = ctx
            .config::<millimeter>("diameter.tolerance.upper")
            .default(0.05)
            .minimum(0.0)
            .maximum(1.0)
            .build()?;

        Ok(Self {
            device,
            diameter_target,
            diameter_tolerance_upper,
            diameter_tolerance_lower,
            in_tolerance: ctx.state::<bool>("in_tolerance").build()?,
            diameter: ctx.measurement::<millimeter>("diameter").build()?,
            diameter_x: ctx
                .measurement::<Option<millimeter>>("diameter_x")
                .build()?,
            diameter_y: ctx
                .measurement::<Option<millimeter>>("diameter_y")
                .build()?,
            roundness: ctx.measurement::<Option<f64>>("roundness").build()?,
            out_of_tolerance: ctx.event("out_of_tolerance").build()?,
            request_timer: Duration::ZERO,
        })
    }
}

impl Machine for LaserV1 {
    fn act(&mut self, dt: Duration) -> ActResult {
        self.update_device(dt)?;

        // --- update measurements ---
        if let Some(m) = self.device.borrow().measurement.clone() {
            fn convert(value: u16) -> Length {
                Length::new::<millimeter>(value as f64 / 1000.0)
            }

            self.diameter.set(convert(m.diameter));
            self.diameter_x.set(m.x_axis.map(convert));
            self.diameter_y.set(m.y_axis.map(convert));
        }

        // --- update roundness ---
        let roundness = self.compute_roundness();
        self.roundness.set(roundness);

        // --- update in_tolerance ---
        let in_tolerance = self.compute_in_tolerance();
        if self.in_tolerance.set(in_tolerance) && !in_tolerance {
            // value changed from in tolerance -> out of tolerance
            self.out_of_tolerance.emit(&());
        };

        Ok(())
    }
}

impl LaserV1 {
    fn update_device(&mut self, dt: Duration) -> ActResult {
        let mut laser = self.device.borrow_mut();

        if let Err(e) = laser.handle_response()
            && let Some(laser_error) = e.downcast_ref::<LaserError>()
            && let LaserError::IoErr() = laser_error
        {
            return Err(ActError {
                kind: ActErrorKind::HardwareFault("Physical hardware I/O broke.".into()),
                impact: ActErrorImpact::Irrecoverable,
            });
        }

        self.request_timer = self.request_timer.saturating_sub(dt);

        if self.request_timer.is_zero() {
            self.request_timer = Duration::from_millis(6);

            if let Err(err) = laser.send_next_request() {
                println!("send_next_request {:?}", err);
            }
        }

        Ok(())
    }

    /// Roundness = min(x, y) / max(x, y)
    fn compute_roundness(&mut self) -> Option<f64> {
        let (Some(x), Some(y)) = (
            self.diameter_x.get_as::<millimeter>(),
            self.diameter_y.get_as::<millimeter>(),
        ) else {
            return None;
        };

        if x > 0.0 && y > 0.0 {
            let roundness = f64::min(x, y) / f64::max(x, y);
            Some(roundness)
        } else if x == 0.0 && y == 0.0 {
            Some(0.0)
        } else {
            None
        }
    }

    /// Calculates if the current diameter is inside of the tolerance
    fn compute_in_tolerance(&mut self) -> bool {
        let target = self.diameter_target.get();
        let top = target + self.diameter_tolerance_upper.get();
        let bottom = target - self.diameter_tolerance_lower.get();

        self.diameter.get() < top && self.diameter.get() > bottom
    }
}
