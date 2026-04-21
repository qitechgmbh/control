use crate::io::stepper_velocity_wago_750_671::{
    C1Mode, C3Flag, S1Flag, StatusByteS1, StepperVelocityWago750671, Wago750671Mode,
};

#[derive(Debug, Clone, Copy)]
pub enum Wago750671ReferenceDirection {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub struct Wago750671ReferenceStatus {
    pub ready: bool,
    pub start_ack: bool,
    pub reference_mode_ack: bool,
    pub busy: bool,
    pub reference_ok: bool,
    pub input1: bool,
    pub input2: bool,
}

pub struct StepperReferenceWago750671<'a> {
    stepper: &'a mut StepperVelocityWago750671,
}

impl<'a> StepperReferenceWago750671<'a> {
    pub(crate) fn new(stepper: &'a mut StepperVelocityWago750671) -> Self {
        Self { stepper }
    }

    pub fn start_reference_run(&mut self, direction: Wago750671ReferenceDirection) {
        let control_byte3 = match direction {
            Wago750671ReferenceDirection::Positive => C3Flag::DirectionPos as u8,
            Wago750671ReferenceDirection::Negative => C3Flag::DirectionNeg as u8,
        };
        self.stepper
            .request_mode_internal(C1Mode::Reference, control_byte3, true);
    }

    pub fn get_mode(&self) -> Option<Wago750671Mode> {
        self.stepper.get_mode()
    }

    pub fn status(&self) -> Wago750671ReferenceStatus {
        let s1 = StatusByteS1::from_bits(self.stepper.get_status_byte1());

        Wago750671ReferenceStatus {
            ready: s1.has_flag(S1Flag::Ready),
            start_ack: s1.has_flag(S1Flag::StartAck),
            reference_mode_ack: self.reference_mode_ack(),
            busy: self.busy(),
            reference_ok: self.reference_ok(),
            input1: self.stepper.get_s3_bit0(),
            input2: self.stepper.get_s3_bit1(),
        }
    }

    pub fn reference_mode_ack(&self) -> bool {
        self.stepper.get_s1_bit5_reference_mode_ack()
    }

    pub fn reference_ok(&self) -> bool {
        self.stepper.get_s2_reference_ok()
    }

    pub fn busy(&self) -> bool {
        self.stepper.get_s2_busy()
    }
}
