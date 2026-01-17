use crate::analog_input_test_machine::AnalogInputTestMachine;
use crate::ip20_test_machine::IP20TestMachine;
#[cfg(feature = "mock-machine")]
use crate::{
    extruder1::mock::ExtruderV2 as ExtruderV2Mock1, extruder2::mock::ExtruderV2 as ExtruderV2Mock2,
    mock::MockMachine, winder2::mock::Winder2,
};

use crate::{
    Machine, MachineNewParams, MachineNewTrait, machine_identification::MachineIdentification,
};

#[cfg(not(feature = "mock-machine"))]
use crate::extruder1::ExtruderV2;
#[cfg(not(feature = "mock-machine"))]
use crate::{
    aquapath1::AquaPathV1, buffer1::BufferV1, extruder2::ExtruderV3, laser::LaserMachine,
    winder2::Winder2,
};

use crate::test_machine::TestMachine;

use lazy_static::lazy_static;

use anyhow::Error;
use std::{any::TypeId, collections::HashMap};

pub type MachineNewClosure =
    Box<dyn Fn(&MachineNewParams) -> Result<Box<dyn Machine>, Error> + Send + Sync>;

pub struct MachineRegistry {
    type_map: HashMap<TypeId, (MachineIdentification, MachineNewClosure)>,
}

impl Default for MachineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MachineRegistry {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn register<T: MachineNewTrait + 'static>(
        &mut self,
        machine_identficiation: MachineIdentification,
    ) {
        self.type_map.insert(
            TypeId::of::<T>(),
            (
                machine_identficiation.clone(),
                // create a machine construction closure
                Box::new(|machine_new_params| Ok(Box::new(T::new(machine_new_params)?))),
            ),
        );
    }

    pub fn new_machine(
        &self,
        machine_new_params: &MachineNewParams,
    ) -> Result<Box<dyn Machine>, anyhow::Error> {
        // get machiine identification
        let device_identification =
            &machine_new_params
                .device_group
                .first()
                .ok_or(anyhow::anyhow!(
                    "[{}::MachineConstructor::new_machine] No device in group",
                    module_path!()
                ))?;

        // find machine new function by comparing MachineIdentification
        let (_, machine_new_closure) = self
            .type_map
            .values()
            .find(|(mi, _)| {
                mi == &device_identification
                    .device_machine_identification
                    .machine_identification_unique
                    .machine_identification
            })
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] Machine not found",
                module_path!()
            ))?;

        // call machine new function by reference
        (machine_new_closure)(machine_new_params)
    }
}

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(Winder2::MACHINE_IDENTIFICATION);

        #[cfg(feature = "mock-machine")]
        mc.register::<ExtruderV2Mock1>(ExtruderV2Mock1::MACHINE_IDENTIFICATION);

        #[cfg(feature = "mock-machine")]
        mc.register::<ExtruderV2Mock2>(ExtruderV2Mock2::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<ExtruderV2>(ExtruderV2::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<ExtruderV3>(ExtruderV3::MACHINE_IDENTIFICATION);

        #[cfg(feature = "mock-machine")]
        mc.register::<MockMachine>(MockMachine::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<LaserMachine>(LaserMachine::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<BufferV1>(BufferV1::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<AquaPathV1>(AquaPathV1::MACHINE_IDENTIFICATION);

        mc.register::<TestMachine>(TestMachine::MACHINE_IDENTIFICATION);

        mc.register::<IP20TestMachine>(IP20TestMachine::MACHINE_IDENTIFICATION);

        mc.register::<AnalogInputTestMachine>(AnalogInputTestMachine::MACHINE_IDENTIFICATION);

        mc
    };
}
