use crate::xtrem::protocol::{Function, Request};

pub fn encode<'a>(
    id_origin: u8,
    id_dest:   u8,
    request: Request<'_>,
    buf: &'a mut [u8]
) -> &'a [u8] {


    true
}

impl Frame {

    pub fn encode_frame(
        id_origin: u8,
        id_dest:   u8,
        request: Request<'_>,
        buf: &mut [u8]
    ) -> bool {


        true
    }

    /// Compute XOR LRC
    pub fn compute_lrc(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc ^ b)
    }

    pub fn encode(&self, buf: &mut [u8]) {

        buf[0] = self.stx;

        // encode header
        buf[1] = format!("{:02}", self.id_origin);
    }

    /// Builds full raw bytes of a frame.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // First add STX
        buf.push(self.stx);

        // Build ASCII payload
        let mut payload = String::new();
        use std::fmt::Write;
        if let Err(e) = write!(
            &mut payload,
            "{:02}{:02}{}{:04X}{:02X}",
            self.id_origin,
            self.id_dest,
            self.function.to_char(),
            self.data_address,
            self.data_length,
        ) {
            eprintln!("Failed to write header: {}", e);
            return buf.clone();
        }

        // Append data
        for b in &self.data {
            if let Err(e) = write!(&mut payload, "{:02X}", b) {
                eprintln!("Failed to write data bytes: {}", e);
                return buf.clone();
            }
        }

        // Append this frame's LRC
        if let Err(e) = write!(&mut payload, "{:02X}", self.lrc) {
            eprintln!("Failed to write LRC: {}", e);
            return buf.clone();
        }

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

        if let Some(unit_index) = ascii.find("kg").or_else(|| ascii.find('g')) {
            let mut start = unit_index;
            while start > 0 {
                let c = ascii.as_bytes()[start - 1] as char;
                if !(c.is_ascii_digit() || c == '.' || c == ' ') {
                    break;
                }
                start -= 1;
            }
            let number_str = ascii[start..unit_index].trim();
            if let Ok(v) = number_str.parse::<f64>() {
                return v;
            }
        }

        0.0
    }
}
