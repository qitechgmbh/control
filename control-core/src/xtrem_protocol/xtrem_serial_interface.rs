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

/// Field name | Description | Bytes Length
///
/// STX       | Start of message: ASCII character 02h | 1
/// ID-O      | Network address of the device sending the message | 2
/// ID-D      | Network address of the destination device | 2
/// F         | Function, action to perform | 1
/// D_ADDRESS | Data address: Identification code of the data register on which the action will be
/// carried out | 4
/// D_L       | data_length (number of byte) | 2
/// DATA      | Date sent | Variable
/// LRC       | Longitudinal redundancy check | 2
/// ETX       | End of message: ASCII character 03h | 1
#[derive(Debug)]
pub struct XtremFrame {
    pub id_origin: u8,
    pub id_dest: u8,
    pub function: Function,
    pub data_address: u16,
    pub data: Vec<u8>,
}

impl XtremFrame {
    /// Computes XOR LRC over provided bytes
    fn extract_lrc(&self, data: &[u8]) -> u8 {
        let mut lrc: u8 = 0;
        for &b in data {
            lrc ^= b;
        }
        lrc
    }

    /// Checks if frame's LRC is valid
    fn check_lrc(&self, frame: &[u8]) -> bool {
        if frame.len() < 3 {
            return false;
        }
        let data_len = frame.len() - 2;
        let (payload, lrc_ascii) = frame.split_at(data_len);

        let lrc_str = match std::str::from_utf8(lrc_ascii) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let received = match u8::from_str_radix(lrc_str, 16) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let calculated = self.extract_lrc(payload);
        calculated == received
    }

    /// Builds an XTREM protocol frame with ASCII encoding and LRC.
    pub fn encode(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        payload.extend(format!("{:02X}{:02X}", self.id_origin, self.id_dest).as_bytes());
        payload.push(self.function.as_char() as u8);
        payload.extend(format!("{:04X}", self.data_address).as_bytes());
        payload.extend(format!("{:02X}", self.data.len()).as_bytes());
        payload.extend(&self.data);

        let lrc = self.extract_lrc(&payload);
        // Start with stx = 0x02
        let mut frame = vec![0x02];
        frame.extend(&payload);
        frame.extend(format!("{:02X}", lrc).as_bytes());
        // End with etx = 0x03
        frame.push(0x03);
        frame
    }

    /// Parses an XTREM frame and validates its LRC
     pub fn decode(&self, frame: &[u8]) -> Option<Self> {
        if frame.len() < 10 || frame[0] != 0x02 || *frame.last()? != 0x03 {
            return None;
        }

        // Remove STX/ETX and split off LRC (last two ASCII hex chars)
        let inner = &frame[1..frame.len() - 1];
        if inner.len() < 2 || !self.check_lrc(inner) {
            return None;
        }

        // Convert to a UTF-8 string once and parse by slices
        let text = String::from_utf8_lossy(inner);
        let mut pos = 0;

        // Helper: take N chars as a &str slice, then advance
        let mut take = |n: usize| {
            if pos + n > text.len() { return None; }
            let s = &text[pos..pos + n];
            pos += n;
            Some(s)
        };

        let id_origin = u8::from_str_radix(take(2)?, 16).ok()?;
        let id_dest   = u8::from_str_radix(take(2)?, 16).ok()?;
        let function  = Function::from_char(take(1)?.chars().next()?)?;
        let data_address = u16::from_str_radix(take(4)?, 16).ok()?;
        let data_len     = usize::from_str_radix(take(2)?, 16).ok()?;

        let data_start = pos;
        let data_end   = data_start + data_len;
        if data_end + 2 > text.len() {
            return None;
        }

        let data = text[data_start..data_end].as_bytes().to_vec();

        Some(Self {
            id_origin,
            id_dest,
            function,
            data_address,
            data,
        })
    }
}

