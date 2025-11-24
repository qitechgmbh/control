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
        let mut buf = Vec::new();
        // First add STX
        buf.push(self.stx);

        // Build ASCII payload
        let mut payload = String::new();
        use std::fmt::Write;
        write!(
            &mut payload,
            "{:02}{:02}{}{:04X}{:02X}",
            self.id_origin,
            self.id_dest,
            self.function.as_char(),
            self.data_address,
            self.data_length,
        )
        .unwrap();

        // Append data
        for b in &self.data {
            write!(&mut payload, "{:02X}", b).unwrap();
        }

        // Append this frame's LRC
        write!(&mut payload, "{:02X}", self.lrc).unwrap();

        buf.extend_from_slice(payload.as_bytes());

        // Last add ETX and CR/LF
        buf.push(self.etx);
        buf.extend_from_slice(b"\r\n");

        buf
    }

    /// Parses an XTREM ASCII response and extracts the numeric weight in kg.
    pub fn parse_weight_from_response(data: &[u8]) -> f64 {
        // Convert to ASCII string
        let ascii = String::from_utf8_lossy(data);

        if let Some(kg_index) = ascii.find("kg") {
            let mut start = kg_index;
            while start > 0 {
                let c = ascii.as_bytes()[start - 1] as char;
                if !(c.is_ascii_digit() || c == '.' || c == ' ') {
                    break;
                }
                start -= 1;
            }
            let number_str = ascii[start..kg_index].trim();
            if let Ok(v) = number_str.parse::<f64>() {
                return v;
            }
        }

        0.0
    }
}
