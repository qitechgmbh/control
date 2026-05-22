use crate::extruder1::ExtruderV2;
use crate::minimal_machines::analog_input_test_machine::AnalogInputTestMachine;
use crate::minimal_machines::digital_input_test_machine::DigitalInputTestMachine;
use crate::minimal_machines::ip20_test_machine::IP20TestMachine;
use crate::minimal_machines::mock::MockMachine;
use crate::minimal_machines::motor_test_machine::MotorTestMachine;
use crate::minimal_machines::test_machine::TestMachine;
use crate::minimal_machines::test_machine_stepper::TestMachineStepper;
use crate::minimal_machines::wago_750_430_di_machine::Wago750_430DiMachine;
use crate::minimal_machines::wago_750_460_machine::Wago750_460Machine;
use crate::minimal_machines::wago_750_501_test_machine::Wago750_501TestMachine;
use crate::minimal_machines::wago_750_531_machine::Wago750_531Machine;
use crate::minimal_machines::wago_750_553_machine::Wago750_553Machine;
use crate::minimal_machines::wago_8ch_dio_test_machine::Wago8chDigitalIOTestMachine;
use crate::minimal_machines::wago_ai_test_machine::WagoAiTestMachine;
use crate::minimal_machines::wago_do_test_machine::WagoDOTestMachine;
use crate::{
    MachineHardware, MachineNew, QiTechMachine, aquapath1::AquaPathV1, laser::LaserMachine,
    winder2::Winder2,
};
use anyhow::Error;
use lazy_static::lazy_static;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use std::{any::TypeId, collections::HashMap};
pub type MachineNewClosure =
    Box<dyn Fn(MachineHardware) -> Result<Box<dyn QiTechMachine>, Error> + Send + Sync>;

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

    pub fn register<T: MachineNew + 'static + QiTechMachine>(
        &mut self,
        machine_identification: Vec<MachineIdentification>,
    ) {
        self.type_map.insert(
            TypeId::of::<T>(),
            (
                machine_identification.clone(),
                // create a machine construction closure
                Box::new(|hardware: MachineHardware| Ok(Box::new(T::new(hardware)?))),
            ),
        );
    }

    pub fn new_machine(
        &self,
        ident: MachineIdentificationUnique,
        hardware: MachineHardware,
    ) -> Result<Box<dyn QiTechMachine>, anyhow::Error> {
        let ident = ident.machine_ident;

        let (_, machine_new_closure) = self
            .type_map
            .values()
            .find(|(ids, _)| ids.contains(&ident)) // 'ids' is the Vec<MachineIdentification>
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] Machine not found",
                module_path!()
            ))?;
        // call machine new function by reference
        (machine_new_closure)(hardware)
    }
}

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();

        // Production machines
        mc.register::<ExtruderV2>(vec![ExtruderV2::MACHINE_IDENTIFICATION, ExtruderV2::MACHINE_IDENTIFICATION_V3]);
        mc.register::<Winder2>(vec![Winder2::MACHINE_IDENTIFICATION]);
        mc.register::<LaserMachine>(vec![LaserMachine::MACHINE_IDENTIFICATION]);
        mc.register::<AquaPathV1>(vec![AquaPathV1::MACHINE_IDENTIFICATION]);

        // Minimal machines
        mc.register::<AnalogInputTestMachine>(vec![AnalogInputTestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<DigitalInputTestMachine>(vec![DigitalInputTestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<IP20TestMachine>(vec![IP20TestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<MockMachine>(vec![MockMachine::MACHINE_IDENTIFICATION]);
        mc.register::<MotorTestMachine>(vec![MotorTestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<TestMachine>(vec![TestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<TestMachineStepper>(vec![TestMachineStepper::MACHINE_IDENTIFICATION]);
        mc.register::<Wago750_430DiMachine>(vec![Wago750_430DiMachine::MACHINE_IDENTIFICATION]);
        mc.register::<Wago750_460Machine>(vec![Wago750_460Machine::MACHINE_IDENTIFICATION]);
        mc.register::<Wago750_501TestMachine>(vec![Wago750_501TestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<Wago750_531Machine>(vec![Wago750_531Machine::MACHINE_IDENTIFICATION]);
        mc.register::<Wago750_553Machine>(vec![Wago750_553Machine::MACHINE_IDENTIFICATION]);
        mc.register::<Wago8chDigitalIOTestMachine>(vec![Wago8chDigitalIOTestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<WagoAiTestMachine>(vec![WagoAiTestMachine::MACHINE_IDENTIFICATION]);
        mc.register::<WagoDOTestMachine>(vec![WagoDOTestMachine::MACHINE_IDENTIFICATION]);

        mc
    };
}
