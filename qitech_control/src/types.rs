use qitech_framework::EnumProperty;

#[derive(Debug, Default, Clone, Copy, PartialEq, EnumProperty)]
pub enum RotationDirection {
    #[default]
    Forward,
    Reverse,
}

impl RotationDirection {
    pub fn modifier(self) -> f64 {
        match self {
            RotationDirection::Forward => 1.0,
            RotationDirection::Reverse => -1.0,
        }
    }
}
