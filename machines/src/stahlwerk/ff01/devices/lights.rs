use std::time::Instant;

use ethercat_hal::io::digital_output::DigitalOutput;

#[derive(Debug)]
pub struct SignalLights {
    green_light: SignalLight,
    yellow_light: SignalLight,
    red_light: SignalLight,

    #[allow(dead_code)]
    beeper: DigitalOutput,
}

// public interface
impl SignalLights {
    pub fn new(
        green_light_output: DigitalOutput,
        yellow_light_output: DigitalOutput,
        red_light_output: DigitalOutput,
        beeper_output: DigitalOutput,
    ) -> Self {
        Self {
            green_light: SignalLight::new(green_light_output),
            yellow_light: SignalLight::new(yellow_light_output),
            red_light: SignalLight::new(red_light_output),
            beeper: beeper_output,
        }
    }

    pub fn update(&mut self, now: Instant) {
        for light in self.lights_mut() {
            light.update(now);
        }
    }

    pub fn lights_disable_all(&mut self) {
        for light in self.lights_mut() {
            light.disable();
        }
    }

    #[allow(dead_code)]
    pub fn lights_enable_all(&mut self, expiry: Option<Instant>) {
        for light in self.lights_mut() {
            light.enable(expiry);
        }
    }

    #[allow(dead_code)]
    pub fn light_enabled(&self, light: Light) -> bool {
        self.get_light(light).enabled()
    }

    pub fn enable_light(&mut self, light: Light, expiry: Option<Instant>) {
        self.get_light_mut(light).enable(expiry);
    }

    #[allow(dead_code)]
    pub fn disable_light(&mut self, light: Light) {
        self.get_light_mut(light).disable();
    }
}

// utils
impl SignalLights {
    fn get_light(&self, light: Light) -> &SignalLight {
        match light {
            Light::Green => &self.green_light,
            Light::Yellow => &self.yellow_light,
            Light::Red => &self.red_light,
        }
    }

    fn get_light_mut(&mut self, light: Light) -> &mut SignalLight {
        match light {
            Light::Green => &mut self.green_light,
            Light::Yellow => &mut self.yellow_light,
            Light::Red => &mut self.red_light,
        }
    }

    fn lights_mut(&mut self) -> [&mut SignalLight; 3] {
        [
            &mut self.green_light,
            &mut self.yellow_light,
            &mut self.red_light,
        ]
    }
}

#[derive(Debug)]
pub struct SignalLight {
    output: DigitalOutput,
    expiry: Option<Instant>,
}

impl SignalLight {
    pub fn new(output: DigitalOutput) -> Self {
        Self {
            output,
            expiry: None,
        }
    }

    pub fn enabled(&self) -> bool {
        self.output.get()
    }

    pub fn enable(&mut self, expiry: Option<Instant>) {
        self.output.set(true);

        // Set the expiry if a duration is provided
        self.expiry = expiry;
    }

    pub fn disable(&mut self) {
        self.output.set(false);
        self.expiry = None;
    }

    #[allow(clippy::collapsible_if)]
    pub fn update(&mut self, now: Instant) {
        if let Some(expiry) = self.expiry {
            if now >= expiry {
                // exceeded expiry, disable
                self.disable();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Light {
    Green,
    Yellow,
    Red,
}
