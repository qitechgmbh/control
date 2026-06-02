/// Why a safety stop was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    WinderTensionArm,
    TapeFeederTensionArm,
    InletTensionArm,
    Optris1Voltage,
    Optris2Voltage,
    HeaterOverTemperature { zones: u8 },
    SleepTimer,
}

/// Safety stop profile — replaces separate emergency_stop variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyStop {
    /// Motors off, mode → Hold, operation_mode → Setup; heaters unchanged.
    MotorsOnly { reason: StopReason },
    /// Same as MotorsOnly plus disable heating.
    Full { reason: StopReason },
}

impl SafetyStop {
    pub const fn reason(self) -> StopReason {
        match self {
            SafetyStop::MotorsOnly { reason } | SafetyStop::Full { reason } => reason,
        }
    }

    pub const fn disables_heaters(self) -> bool {
        matches!(self, SafetyStop::Full { .. })
    }
}
