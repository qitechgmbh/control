use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework::MachineIdentification;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::MachineInterface;
use qitech_framework::machine::error::ActError;
use qitech_framework::machine::error::ActErrorKind;
use qitech_framework::machine::error::ActResult;
use qitech_framework::machine::error::BuildError;
use qitech_framework::machine::resource::ConfigProperty;
use qitech_framework::machine::resource::Measurement;
use qitech_framework::machine::resource::StateProperty;
use qitech_framework::vendors;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserError;
use qitech_lib::units::Length;
use qitech_lib::units::length::millimeter;

// #[machine("laser_v1")]
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

    // -- misc ---
    last_request: Instant,
}

impl MachineInterface for LaserV1 {
    const SCHEMAS: &[&'static str] = &[include_str!("../../../schemas/laser_v1.yaml")];
}

impl MachineBuild for LaserV1 {
    fn build(mut ctx: BuildContext) -> Result<Self, BuildError> {
        let device = ctx.get_modbus_rtu_device::<LaserDevice>(0)?;

        let diameter_target = ctx
            .config::<millimeter>("diameter.target")
            .default(1.75)
            .minimum(0.0)
            .register()?;

        let diameter_tolerance_lower = ctx
            .config::<millimeter>("diameter.tolerance.lower")
            .default(0.05)
            .minimum(0.0)
            .register()?;

        let diameter_tolerance_upper = ctx
            .config::<millimeter>("diameter.tolerance.upper")
            .default(0.05)
            .minimum(0.0)
            .register()?;

        Ok(Self {
            device,
            diameter_target,
            diameter_tolerance_upper,
            diameter_tolerance_lower,
            in_tolerance: ctx.state::<bool>("in_tolerance").register()?,
            diameter: ctx.measurement::<millimeter>("diameter").register()?,
            diameter_x: ctx
                .measurement::<Option<millimeter>>("diameter_x")
                .register()?,
            diameter_y: ctx
                .measurement::<Option<millimeter>>("diameter_y")
                .register()?,
            roundness: ctx.measurement::<Option<f64>>("roundness").register()?,
            last_request: Instant::now(),
        })
    }
}

impl Machine for LaserV1 {
    fn act(&mut self) -> ActResult {
        self.update_device()?;

        if let Some(m) = self.device.borrow().measurement.clone() {
            fn convert(value: u16) -> Length {
                Length::new::<millimeter>(value as f64 / 1000.0)
            }

            self.diameter.set(convert(m.diameter));
            self.diameter_x.set(Some(convert(m.x_axis)));
            self.diameter_y.set(Some(convert(m.y_axis)));
        }

        let roundness = self.compute_roundness();
        self.roundness.set(roundness);

        let in_tolerance = self.compute_in_tolerance();
        self.in_tolerance.set(in_tolerance);

        Ok(())
    }
}

impl LaserV1 {
    pub const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 6,
    };

    fn update_device(&mut self) -> ActResult {
        let mut laser = self.device.borrow_mut();

        if let Err(e) = laser.handle_response()
            && let Some(laser_error) = e.downcast_ref::<LaserError>()
            && let LaserError::IoErr() = laser_error
        {
            return Err(ActError {
                recoverable: false,
                kind: ActErrorKind::HardwareFault("Physical hardware I/O broke.".into()),
            });
        }

        let now = Instant::now();
        if now.duration_since(self.last_request) > Duration::from_millis(6) {
            self.last_request = now;
            let res = laser.send_next_request();

            if res.is_err() {
                println!("send_next_request {:?}", res);
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
        // const DIAMETER_EPSILON: f64 = 0.0001; // in mm
        //
        // // early return true if the diameter is 0 to prevent warning happening before start
        // if self.diameter.get_as::<millimeter>() < DIAMETER_EPSILON {
        //     return true;
        // }

        let target = self.diameter_target.get();
        let top = target + self.diameter_tolerance_upper.get();
        let bottom = target - self.diameter_tolerance_lower.get();

        self.diameter.get() < top && self.diameter.get() > bottom
    }
}
