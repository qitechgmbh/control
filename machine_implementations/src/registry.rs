use crate::{
    MachineHardware, MachineNew, QiTechMachine,
    minimal_machines::digital_input_test_machine::DigitalInputTestMachine,
};
use anyhow::Error;
use lazy_static::lazy_static;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use std::{any::TypeId, collections::HashMap};

pub type MachineNewClosure =
    Box<dyn Fn(MachineHardware) -> Result<Box<dyn QiTechMachine>, Error> + Send + Sync>;

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

    pub fn register<T: MachineNew + 'static + QiTechMachine>(
        &mut self,
        machine_identification: MachineIdentification,
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
        // find machine new function by comparing MachineIdentification
        let (_, machine_new_closure) =
            self.type_map
                .values()
                .find(|(mi, _)| mi == &ident)
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
        mc.register::<DigitalInputTestMachine>(DigitalInputTestMachine::MACHINE_IDENTIFICATION);
/*


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
