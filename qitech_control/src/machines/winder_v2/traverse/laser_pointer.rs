use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;

pub struct LaserPointer {
    pub device: Rc<RefCell<dyn DigitalOutputDevice>>,
    pub enabled: StateProperty<bool>,
}

impl LaserPointer {
    const LASER_PORT: usize = 0;

    pub fn set_enabled(&mut self, value: bool) {
        self.enabled.set(value);
        self.device.borrow_mut().set_output(Self::LASER_PORT, true);
    }
}
