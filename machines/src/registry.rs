use crate::minimal_machines::analog_input_test_machine::AnalogInputTestMachine;
use crate::minimal_machines::digital_input_test_machine::DigitalInputTestMachine;
use crate::minimal_machines::ip20_test_machine::IP20TestMachine;
use crate::minimal_machines::wago_8ch_dio_test_machine::Wago8chDigitalIOTestMachine;
use crate::minimal_machines::wago_750_430_di_machine::Wago750_430DiMachine;
use crate::minimal_machines::wago_750_460_machine::Wago750_460Machine;
use crate::minimal_machines::wago_750_501_test_machine::Wago750_501TestMachine;
use crate::minimal_machines::wago_750_531_machine::Wago750_531Machine;
use crate::minimal_machines::wago_750_553_machine::Wago750_553Machine;
use crate::minimal_machines::wago_ai_test_machine::WagoAiTestMachine;
use crate::minimal_machines::wago_do_test_machine::WagoDOTestMachine;
use crate::wago_serial_machine::WagoSerialMachine;
#[cfg(feature = "mock-machine")]
use crate::{
    extruder1::mock::ExtruderV2 as ExtruderV2Mock1, extruder2::mock::ExtruderV2 as ExtruderV2Mock2,
    minimal_machines::mock::MockMachine, winder2::mock::Winder2,
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

use crate::minimal_machines::{
    test_machine::TestMachine, test_machine_stepper::TestMachineStepper,
};

use lazy_static::lazy_static;

use crate::minimal_machines::motor_test_machine::MotorTestMachine;
use anyhow::Error;
use std::{any::TypeId, collections::HashMap};

pub type MachineNewClosure =
    Box<dyn Fn(&MachineNewParams) -> Result<Box<dyn Machine>, Error> + Send + Sync>;

pub struct MachineRegistry {
    type_map: HashMap<TypeId, (Vec<MachineIdentification>, MachineNewClosure)>,
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

    pub fn register<T: MachineNewTrait + 'static + Machine>(
        &mut self,
        machine_identification: Vec<MachineIdentification>,
    ) {
        self.type_map.insert(
            TypeId::of::<T>(),
            (
                machine_identification.clone(),
                // create a machine construction closure
                Box::new(|machine_new_params| Ok(Box::new(T::new(machine_new_params)?))),
            ),
        );
    }

    pub fn new_machine(
        &self,        
        machine_new_params: &MachineNewParams
    ) -> Result<Box<dyn Machine>, anyhow::Error> {
        let device_identification =
            &machine_new_params
                .device_group
                .first()
                .ok_or(anyhow::anyhow!(
                    "[{}::MachineConstructor::new_machine] No device in group",
                    module_path!()
                ))?;
        let ident = device_identification.device_machine_identification.machine_identification_unique.machine_identification;
        let (_, machine_new_closure) = self.type_map
            .values()
            .find(|(ids, _)| ids.contains(&ident)) // 'ids' is the Vec<MachineIdentification>
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] Machine not found",
                module_path!()
            ))?;        
        (machine_new_closure)(machine_new_params)
    }
}

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(vec![Winder2::MACHINE_IDENTIFICATION, Winder2::MACHINE_IDENTIFICATION_7031_SPOOL ]);

        #[cfg(feature = "mock-machine")]
        mc.register::<ExtruderV2Mock1>(vec![ExtruderV2Mock1::MACHINE_IDENTIFICATION]);

        #[cfg(feature = "mock-machine")]
        mc.register::<ExtruderV2Mock2>(vec![ExtruderV2Mock2::MACHINE_IDENTIFICATION]);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<ExtruderV2>(vec![ExtruderV2::MACHINE_IDENTIFICATION]);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<ExtruderV3>(vec![ExtruderV3::MACHINE_IDENTIFICATION]);

        #[cfg(feature = "mock-machine")]
        mc.register::<MockMachine>(vec![MockMachine::MACHINE_IDENTIFICATION]);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<LaserMachine>(vec![LaserMachine::MACHINE_IDENTIFICATION]);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<BufferV1>(vec![BufferV1::MACHINE_IDENTIFICATION]);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<AquaPathV1>(vec![AquaPathV1::MACHINE_IDENTIFICATION]);

        mc.register::<TestMachine>(vec![TestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<IP20TestMachine>(vec![IP20TestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<AnalogInputTestMachine>(vec![AnalogInputTestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<WagoAiTestMachine>(vec![WagoAiTestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<MotorTestMachine>(vec![MotorTestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<DigitalInputTestMachine>(vec![DigitalInputTestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<WagoDOTestMachine>(vec![WagoDOTestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<Wago750_531Machine>(vec![Wago750_531Machine::MACHINE_IDENTIFICATION]);

        mc.register::<Wago750_501TestMachine>(vec![Wago750_501TestMachine::MACHINE_IDENTIFICATION]);

        mc.register::<Wago8chDigitalIOTestMachine>(
            vec![Wago8chDigitalIOTestMachine::MACHINE_IDENTIFICATION],
        );

        mc.register::<WagoSerialMachine>(vec![WagoSerialMachine::MACHINE_IDENTIFICATION]);

        mc.register::<TestMachineStepper>(vec![TestMachineStepper::MACHINE_IDENTIFICATION]);

        mc.register::<Wago750_430DiMachine>(vec![Wago750_430DiMachine::MACHINE_IDENTIFICATION]);

        mc.register::<Wago750_460Machine>(vec![Wago750_460Machine::MACHINE_IDENTIFICATION]);

        mc.register::<Wago750_553Machine>(vec![Wago750_553Machine::MACHINE_IDENTIFICATION]);
        mc
    };
}