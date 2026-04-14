use crate::{
    MachineHardware, MachineNew, QiTechMachine, laser::LaserMachine, minimal_machines::digital_input_test_machine::DigitalInputTestMachine, winder2::Winder2
};
use anyhow::Error;
use lazy_static::lazy_static;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use std::{any::TypeId, collections::HashMap};
use crate::extruder1::ExtruderV2;
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


        let (_, machine_new_closure) = self.type_map
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
        mc.register::<DigitalInputTestMachine>(vec![DigitalInputTestMachine::MACHINE_IDENTIFICATION]);
        
        mc.register::<ExtruderV2>(vec![ExtruderV2::MACHINE_IDENTIFICATION,ExtruderV2::MACHINE_IDENTIFICATION_V3 ]);
        mc.register::<Winder2>(vec![Winder2::MACHINE_IDENTIFICATION]);
        mc.register::<LaserMachine>(vec![LaserMachine::MACHINE_IDENTIFICATION]);
/*
        #[cfg(not(feature = "mock-machine"))]
        mc.register::<LaserMachine>(LaserMachine::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<BufferV1>(BufferV1::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<AquaPathV1>(AquaPathV1::MACHINE_IDENTIFICATION);

        mc.register::<TestMachine>(TestMachine::MACHINE_IDENTIFICATION);

        mc.register::<IP20TestMachine>(IP20TestMachine::MACHINE_IDENTIFICATION);

        mc.register::<AnalogInputTestMachine>(AnalogInputTestMachine::MACHINE_IDENTIFICATION);

        mc.register::<WagoAiTestMachine>(WagoAiTestMachine::MACHINE_IDENTIFICATION);

        mc.register::<MotorTestMachine>(MotorTestMachine::MACHINE_IDENTIFICATION);


        mc.register::<WagoDOTestMachine>(WagoDOTestMachine::MACHINE_IDENTIFICATION);

        mc.register::<Wago750_501TestMachine>(Wago750_501TestMachine::MACHINE_IDENTIFICATION);

        mc.register::<Wago8chDigitalIOTestMachine>(
            Wago8chDigitalIOTestMachine::MACHINE_IDENTIFICATION,
        );

        mc.register::<WagoSerialMachine>(WagoSerialMachine::MACHINE_IDENTIFICATION);

        mc.register::<TestMachineStepper>(TestMachineStepper::MACHINE_IDENTIFICATION);
        mc.register::<Wago750_430DiMachine>(Wago750_430DiMachine::MACHINE_IDENTIFICATION);
        mc.register::<Wago750_553Machine>(Wago750_553Machine::MACHINE_IDENTIFICATION);*/
        mc
    };
}
