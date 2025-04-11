use std::{fmt, future::Future, pin::Pin, sync::Arc};

use anyhow::Error;
use smol::lock::RwLock;

use crate::pdo::el7031::{
    EncControlCompact, EncStatusCompact, PosControl2, StmControl, StmStatus, StmVelocity,
};

/// Pulse Train Output (PTO) device
///
/// Generates digital puleses with a given frequency (not PWM) and counts them.
pub struct StepperVelocity {
    /// Write to the pulse train output
    pub write: Box<
        dyn Fn(StepperVelocityOutput) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>
            + Send
            + Sync,
    >,
    /// Read the state of the pulse train output
    pub state: Box<
        dyn Fn() -> Pin<Box<dyn Future<Output = Result<StepperVelocityState, Error>> + Send>>
            + Send
            + Sync,
    >,
}

impl fmt::Debug for StepperVelocity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StepperVelocity")
    }
}

impl<'device> StepperVelocity {
    pub fn new<PORT, DEVICE>(device: Arc<RwLock<DEVICE>>, port: PORT) -> StepperVelocity
    where
        PORT: Clone + Copy + Send + Sync + 'static,
        DEVICE: StepperVelocityDevice<PORT> + Send + Sync + 'static,
    {
        // build async write closure
        let device1 = device.clone();
        let write = Box::new(
            move |value: StepperVelocityOutput| -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
                let device_clone = device1.clone();
                Box::pin(async move {
                    let mut device = device_clone.write().await;
                    device.stepper_velocity_write(port, value);
                })
            },
        );

        // build async get closure
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = Result<StepperVelocityState, Error>> + Send>> {
                let device2 = device2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.stepper_velocity_state(port)
                })
            },
        );

        StepperVelocity { write, state }
    }
}

#[derive(Debug, Clone)]
pub struct StepperVelocityState {
    pub input: StepperVelocityInput,
    pub output: StepperVelocityOutput,
}

#[derive(Debug, Clone)]
pub struct StepperVelocityInput {
    pub enc_status_compact: EncStatusCompact,
    pub stm_status: StmStatus,
    // pub stm_sychron_info_data: StmSynchronInfoData,
}

#[derive(Debug, Clone)]
pub struct StepperVelocityOutput {
    pub enc_control_compact: EncControlCompact,
    pub stm_control: StmControl,
    pub stm_velocity: StmVelocity,
    pub pos_control_2: PosControl2,
}

pub trait StepperVelocityDevice<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn stepper_velocity_write(
        &mut self,
        port: PORT,
        value: StepperVelocityOutput,
    ) -> Result<(), Error>;
    fn stepper_velocity_state(&self, port: PORT) -> Result<StepperVelocityState, Error>;
}
