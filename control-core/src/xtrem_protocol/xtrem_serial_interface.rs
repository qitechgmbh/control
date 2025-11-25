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

        // Expected layout:
        // 01 | 00 | r | 0001 | 02 | 01
        // ^    ^    ^    ^      ^    ^
        // |    |    |    |      |    └─ data (here "01")
        // |    |    |    |      └──── data length
        // |    |    |    └─────────── register
        // |    |    └──────────────── function
        // |    └───────────────────── destination ID
        // └────────────────────────── origin ID

        // Safety check: must be long enough
        if ascii.len() < 13 {
            return 0.0;
        }

        // Data length is at position 10–12
        let data_len_str = &ascii[10..12];
        let data_len = data_len_str.parse::<usize>().unwrap_or(0);

        // Data starts at position 12
        let data_start = 12;
        let data_end = data_start + data_len * 2; // 2 chars per byte (ASCII hex)

        if ascii.len() < data_end {
            return 0.0;
        }

        let data_str = &ascii[data_start..data_end];

        // Convert ASCII hex to integer
        if let Ok(value) = u16::from_str_radix(data_str, 16) {
            return value as f64;
        }

        0.0
    }
}
