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
    ReadSerial,
}

impl DataAddress {
    pub const fn as_hex(&self) -> u16 {
        match self {
            Self::ReadSerial => 0x0000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct XtremFrame {
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

impl XtremFrame {
    /// Compute XOR LRC
    pub fn compute_lrc(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc ^ b)
    }
    /// Builds full raw bytes of a frame.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = vec![self.stx];
        buf.push(self.id_origin);
        buf.push(self.id_dest);
        buf.push(self.function.as_char() as u8);
        buf.extend_from_slice(&self.data_address.to_be_bytes());
        buf.push(self.data_length);
        buf.extend_from_slice(&self.data);
        buf.push(self.lrc);
        buf.push(self.etx);
        buf
    }
}
