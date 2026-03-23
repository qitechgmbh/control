#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    ReadRequest,
    ReadResponse,
    WriteRequest,
    WriteResponse,
    ExecuteRequest,
    ExecuteResponse,
}

impl Function {
    pub const fn as_char(&self) -> char {
        match self {
            Self::ReadRequest => 'R',
            Self::ReadResponse => 'r',
            Self::WriteRequest => 'W',
            Self::WriteResponse => 'w',
            Self::ExecuteRequest => 'E',
            Self::ExecuteResponse => 'e',
        }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'R' => Some(Self::ReadRequest),
            'r' => Some(Self::ReadResponse),
            'W' => Some(Self::WriteRequest),
            'w' => Some(Self::WriteResponse),
            'E' => Some(Self::ExecuteRequest),
            'e' => Some(Self::ExecuteResponse),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataAddress {
    Serial,
    DeviceID,
    Weight,
}

impl DataAddress {
    pub const fn as_hex(&self) -> u16 {
        match self {
            Self::Serial => 0x0000,
            Self::DeviceID => 0x0001,
            Self::Weight => 0x0101,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub stx: u8,
    pub id_origin: u8,
    pub id_dest: u8,
    pub function: Function,
    pub data_address: u16,
    pub data_length: u8,
    pub data: Vec<u8>,
    pub lrc: u8,
    pub etx: u8,
}
