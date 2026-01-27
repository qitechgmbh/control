use strum_macros::{EnumCount};

#[derive(Debug, Clone, Copy, EnumCount)]
pub enum HoldingRegister
{
    /// Register 0x0002
    SetFrequency = 0x2,

    /// Register 0x0003
    #[allow(dead_code)]
    RunCommand = 0x3,

    /// Register 0x0004
    #[allow(dead_code)]
    AccelerationTime = 0x4,

    /// Register 0x0005
    #[allow(dead_code)]
    DecelerationTime = 0x5,
}

#[derive(Debug, Clone, Copy, EnumCount)]
pub enum InputRegister
{
    /// Register 0x0008
    #[allow(dead_code)]
    BusVoltage,

    /// Register 0x0009
    #[allow(dead_code)]
    LineCurrent,

    /// Register 0x000A
    #[allow(dead_code)]
    DriveTemperature,

    /// Register 0x000B
    #[allow(dead_code)]
    SystemStatus,

    /// Register 0x000C
    #[allow(dead_code)]
    ErrorCode,

    /// Register 0x000D
    #[allow(dead_code)]
    CurrentFrequency,
}

impl HoldingRegister 
{
    pub const OFFSET: u16 = 0x2;
    
    pub const fn address(self) -> u16
    {
        match self 
        {
            Self::SetFrequency     => Self::OFFSET,     // 0x0002
            Self::RunCommand       => Self::OFFSET + 1, // 0x0003
            Self::AccelerationTime => Self::OFFSET + 2, // 0x0004
            Self::DecelerationTime => Self::OFFSET + 3, // 0x0005
        }
    }
}

impl InputRegister 
{
    pub const OFFSET: u16 = 0x8;
    
    #[allow(dead_code)]
    pub const fn address(self) -> u16 
    {
        match self 
        {
            Self::BusVoltage       => Self::OFFSET,     // 0x0008
            Self::LineCurrent      => Self::OFFSET + 1, // 0x0009
            Self::DriveTemperature => Self::OFFSET + 2, // 0x000A
            Self::SystemStatus     => Self::OFFSET + 3, // 0x000B
            Self::ErrorCode        => Self::OFFSET + 4, // 0x000C
            Self::CurrentFrequency => Self::OFFSET + 5, // 0x000D
        }
    }
}