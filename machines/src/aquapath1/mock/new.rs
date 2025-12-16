use std::{sync::Arc, time::{Duration, Instant}};

use crate::{MachineNewTrait, MachineNewParams};
use super::{AquaPathV1Namespace, AquaPathV1, AquaPathV1Mode, controller::Controller};
use anyhow::Error;
use control_core::controllers::pid::PidController;
use ethercat_hal::io::analog_output::{AnalogOutput, AnalogOutputDevice};
use smol::lock::RwLock;
use super::{
    MockMachine,
    api::{MockMachineNamespace, Mode},
};
use crate::{MachineNewHardware, MachineNewParams, MachineNewTrait};
use anyhow::Error;
use units::{f64::Frequency, ThermodynamicTemperature};
use units::frequency::hertz;

impl MachineNewTrait for AquaPathV1 {
    fn new<'maindevice>(
        params: &MachineNewParams
    ) -> Result<Self, Error>
    {
        let front_controller = Controller::new(
            0.0, 0.0, 0.0, Duration::new(0u64, 0u32), 
            crate::aquapath1::Temperature { temperature: (), cooling: true, heating: false, target_temperature: () },
            ThermodynamicTemperature::new(0),
            AnalogOutput::new(Arc::new(
                AnalogOutputDevice
            )),
        );

        let (sender, receiver) = smol::channel::unbounded();
        let mut water_cooling = Self {
            main_sender: None,
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            namespace: AquapathV1Namespace {
                namespace: params.namespace.clone(),
            },
            mode: AquapathV1Mode::Standby,
            last_measurement_emit: Instant::now(),
            front_controller,
            back_controller,
        };
        water_cooling.emit_state();

        Ok(water_cooling)
    }
}
