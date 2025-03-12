pub mod new;

use ethercat_hal::actors::analog_input_logger::AnalogInputLogger;
use ethercat_hal::actors::digital_output_setter::DigitalOutputSetter;
use ethercat_hal::actors::stepper_driver_pulse_train::StepperDriverPulseTrain;
use ethercat_hal::actors::Actor;

#[derive(Debug)]
pub struct WinderV1 {
    // drivers
    pub traverse_driver: StepperDriverPulseTrain,
    pub puller_driver: StepperDriverPulseTrain,
    pub winder_driver: StepperDriverPulseTrain,
    pub tension_arm_driver: AnalogInputLogger,
    pub laser_driver: DigitalOutputSetter,
}

impl Actor for WinderV1 {
    fn act(
        &mut self,
        now_ts: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.traverse_driver.act(now_ts).await;
            self.puller_driver.act(now_ts).await;
            self.winder_driver.act(now_ts).await;
            self.tension_arm_driver.act(now_ts).await;
            self.laser_driver.act(now_ts).await;
        })
    }
}
