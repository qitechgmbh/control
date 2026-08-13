use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;

use crate::machines::winder_v2::WinderV1;

pub struct LaserPointer {
    device: Rc<RefCell<dyn DigitalOutputDevice>>,
    enabled: StateProperty<bool>,
}

impl LaserPointer {
    const LASER_PORT: usize = 0;

    pub fn init<const VARIANT: usize>(
        ctx: &mut BuildContext,
        device: Rc<RefCell<dyn DigitalOutputDevice>>,
    ) -> BuildResult<Self> {
        ctx.command("laser_pointer.enable")
            .execute(|m: &mut WinderV1<VARIANT>| m.laser_pointer.set_enabled(true))
            .build()?;

        ctx.command("laser_pointer.disable")
            .execute(|m: &mut WinderV1<VARIANT>| m.laser_pointer.set_enabled(false))
            .build()?;

        let enabled = ctx.state::<bool>("laser_pointer.enabled").build()?;
        Ok(Self { device, enabled })
    }

    fn set_enabled(&mut self, value: bool) -> ActResult {
        self.enabled.set(value);
        self.device.borrow_mut().set_output(Self::LASER_PORT, value);
        Ok(())
    }
}
