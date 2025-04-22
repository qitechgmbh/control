use std::{fmt, future::Future, pin::Pin, sync::Arc};

use anyhow::Error;
use smol::lock::RwLock;

use crate::pdo::el70x1::{EncControlCompact, EncStatusCompact, StmControl, StmStatus, StmVelocity};

/// Pulse Train Output (PTO) device
///
/// Generates digital puleses with a given frequency (not PWM) and counts them.
pub struct StepperVelocityEL70x1 {
    /// Write to the pulse train output
    pub write: Box<
        dyn Fn(
                StepperVelocityEL70x1Output,
            ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>
            + Send
            + Sync,
    >,
    /// Read the state of the pulse train output
    pub state: Box<
        dyn Fn() -> Pin<Box<dyn Future<Output = Result<StepperVelocityEL70x1State, Error>> + Send>>
            + Send
            + Sync,
    >,
}

impl fmt::Debug for StepperVelocityEL70x1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StepperVelocity")
    }
}

impl<'device> StepperVelocityEL70x1 {
    pub fn new<PORT, DEVICE>(device: Arc<RwLock<DEVICE>>, port: PORT) -> StepperVelocityEL70x1
    where
        PORT: Clone + Copy + Send + Sync + 'static,
        DEVICE: StepperVelocityEL70x1Device<PORT> + Send + Sync + 'static,
    {
        // build async write closure
        let device1 = device.clone();
        let write = Box::new(
            move |value: StepperVelocityEL70x1Output| -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
                let device_clone = device1.clone();
                Box::pin(async move {
                    let mut device = device_clone.write().await;
                    device.stepper_velocity_write(port, value)
                })
            },
        );

        // build async get closure
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = Result<StepperVelocityEL70x1State, Error>> + Send>> {
                let device2 = device2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.stepper_velocity_state(port)
                })
            },
        );

        StepperVelocityEL70x1 { write, state }
    }
}

#[derive(Debug, Clone)]
pub struct StepperVelocityEL70x1State {
    pub input: StepperVelocityEL70x1Input,
    pub output: StepperVelocityEL70x1Output,
}

#[derive(Debug, Clone)]
pub struct StepperVelocityEL70x1Input {
    pub enc_status_compact: EncStatusCompact,
    pub stm_status: StmStatus,
    // pub stm_sychron_info_data: StmSynchronInfoData,
}

#[derive(Debug, Clone)]
pub struct StepperVelocityEL70x1Output {
    pub enc_control_compact: EncControlCompact,
    pub stm_control: StmControl,
    pub stm_velocity: StmVelocity,
}

pub trait StepperVelocityEL70x1Device<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn stepper_velocity_write(
        &mut self,
        port: PORT,
        value: StepperVelocityEL70x1Output,
    ) -> Result<(), Error>;
    fn stepper_velocity_state(&self, port: PORT) -> Result<StepperVelocityEL70x1State, Error>;
}
