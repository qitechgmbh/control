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

#[derive(Debug, Clone)]
pub struct XtremFrame {
    pub stx: u8,
    pub id_origin: u8,
    pub id_dest: u8,
    pub function: Function,
    pub data_address: u16,
    pub data: Vec<u8>,
    pub lrc: u8,
    pub etx: u8,
}

impl XtremFrame {
    /// Constructor
    pub fn new(
        id_origin: u8,
        id_dest: u8,
        function: Function,
        data_address: u16,
        data: Vec<u8>,
    ) -> Self {
        let mut payload = Vec::new();
        payload.extend(format!("{:02X}{:02X}", id_origin, id_dest).as_bytes());
        payload.push(function.as_char() as u8);
        payload.extend(format!("{:04X}", data_address).as_bytes());
        payload.extend(format!("{:02X}", data.len()).as_bytes());
        payload.extend(&data);

        let lrc = Self::compute_lrc(&payload);

        Self {
            stx: 0x02,
            id_origin,
            id_dest,
            function,
            data_address,
            data,
            lrc,
            etx: 0x03,
        }
    }

    /// Compute XOR LRC
    fn compute_lrc(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc ^ b)
    }

    /// Build full XTREM frame
    pub fn encode(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend(format!("{:02X}{:02X}", self.id_origin, self.id_dest).as_bytes());
        payload.push(self.function.as_char() as u8);
        payload.extend(format!("{:04X}", self.data_address).as_bytes());
        payload.extend(format!("{:02X}", self.data.len()).as_bytes());
        payload.extend(&self.data);

        let mut frame = vec![self.stx];
        frame.extend(&payload);
        frame.extend(format!("{:02X}", self.lrc).as_bytes());
        frame.push(self.etx);

        frame
    }

    /// Decode an XTREM frame
    pub fn decode(frame: &[u8]) -> Option<Self> {
        if frame.len() < 12 || frame[0] != 0x02 || *frame.last()? != 0x03 {
            return None;
        }

        // Extract inner ASCII-encoded payload
        let inner = &frame[1..frame.len() - 1]; // drop STX/ETX
        if inner.len() < 2 {
            return None;
        }

        // Last 2 ASCII chars are LRC
        let (payload, lrc_ascii) = inner.split_at(inner.len() - 2);

        let lrc_str = std::str::from_utf8(lrc_ascii).ok()?;
        let received_lrc = u8::from_str_radix(lrc_str, 16).ok()?;

        let calculated = Self::compute_lrc(payload);

        if received_lrc != calculated {
            return None;
        }

        // Now parse ASCII components
        let text = std::str::from_utf8(payload).ok()?;
        let mut idx = 0;
        let mut take = |n: usize| {
            let s = &text[idx..idx + n];
            idx += n;
            s
        };

        let id_origin = u8::from_str_radix(take(2), 16).ok()?;
        let id_dest = u8::from_str_radix(take(2), 16).ok()?;
        let function = Function::from_char(take(1).chars().next()?)?;
        let data_address = u16::from_str_radix(take(4), 16).ok()?;
        let data_len = usize::from_str_radix(take(2), 16).ok()?;

        let data_str = &text[idx..idx + data_len];
        let data = data_str.as_bytes().to_vec();

        Some(Self {
            stx: 0x02,
            id_origin,
            id_dest,
            function,
            data_address,
            data,
            lrc: received_lrc,
            etx: 0x03,
        })
    }
}
