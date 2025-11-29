use ethercrab::{SubDevice, SubDeviceRef};
use std::fmt;

const DIAGNOSIS_HISTORY_INDEX: u16 = 0x10f3;
const NEWEST_MESSAGE: u8 = 02;
const NEW_MESSAGE_AVAILABLE: u8 = 04;
const DIAG_MESSAGE_LENGTH: usize = 26;

pub struct SubdeviceDiagnosisEntry {
    pub diag_code: u32,
    pub flags: u16,
    pub text_id: u16,
    pub timestamp: u64,
    pub data_type_p1: u16,
    pub p1: u8,
    pub flags_p2: u16,
    pub p2: [u8; 5],
}

impl fmt::Display for SubdeviceDiagnosisEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SubdeviceDiagnosisEntry {{ diag_code: {}, flags: {}, text_id: {}, timestamp: {}, data_type_p1: {}, p1: {}, flags_p2: {}, p2: {:?} }}",
            self.diag_code,
            self.flags,
            self.text_id,
            self.timestamp,
            self.data_type_p1,
            self.p1,
            self.flags_p2,
            self.p2
        )
    }
}

impl SubdeviceDiagnosisEntry {
    pub fn to_pretty_string(&self) -> String {
        // Decode message type
        let msg_type = match self.flags {
            0x0000 => "Info",
            0x0001 => "Warning",
            0x0002 => "Error",
            _ => "Unknown",
        };

        // Decode data type of P1
        let p1_type = match self.data_type_p1 {
            0x0005 => "UINT8",
            _ => "Unknown",
        };

        // Decode data type of P2
        let p2_type = match self.flags_p2 {
            0x0005 => "BYTE[5]",
            _ => "Unknown",
        };

        // Format P2 bytes as hex list
        let p2_hex: Vec<String> = self.p2.iter().map(|b| format!("0x{:02X}", b)).collect();

        format!(
            "Diagnosis Entry:\n\
             ├─ diag_code:     0x{:08X}\n\
             ├─ message type:  {} (0x{:04X})\n\
             ├─ text_id:       {}{}\n\
             ├─ timestamp:     {} (64-bit)\n\
             ├─ P1 type:       {} (0x{:04X})\n\
             ├─ P1 value:      {}\n\
             ├─ P2 type:       {} (0x{:04X})\n\
             └─ P2 data:       [{}]",
            self.diag_code,
            msg_type,
            self.flags,
            self.text_id,
            if self.text_id == 0 { " (no text)" } else { "" },
            self.timestamp,
            // P1
            p1_type,
            self.data_type_p1,
            self.p1,
            // P2
            p2_type,
            self.flags_p2,
            p2_hex.join(", ")
        )
    }
}

/*
    Could be tested with this:
        00 E8 00 A0 02 00 00 00 5F FB D5 A5 A6 0B 00 00 05 00 11 05 10 08 2A 00 2A 00
*/

pub fn convert_raw_diagnosis_bytes(
    message: [u8; DIAG_MESSAGE_LENGTH],
) -> Result<SubdeviceDiagnosisEntry, anyhow::Error> {
    let diag_code_bytes = [message[0], message[1], message[2], message[3]];
    let diag_code = u32::from_le_bytes(diag_code_bytes);

    let flags_code_bytes = [message[4], message[5]];
    let flags = u16::from_le_bytes(flags_code_bytes);

    let text_id_bytes = [message[6], message[7]];
    let text_id = u16::from_le_bytes(text_id_bytes);

    let timestamp_bytes = [
        message[8],
        message[9],
        message[10],
        message[11],
        message[12],
        message[13],
        message[14],
        message[15],
    ];
    let timestamp = u64::from_le_bytes(timestamp_bytes);

    let data_type_p1_bytes = [message[16], message[17]];
    let data_type_p1 = u16::from_le_bytes(data_type_p1_bytes);

    // TODO: Find the Constants
    // 0x0005 Apparently Means p1 variable is u8
    // For now we only handle u8
    let p1 = match data_type_p1 {
        0x0005 => message[18],
        _ => 0,
    };

    if p1 == 0 {
        return Err(anyhow::anyhow!(
            "convert_raw_diagnosis_bytes: Datatype_p1 is unknown {:?}",
            data_type_p1
        ));
    }

    // flags_p2 is packed, whatever thats supposed to mean
    let flags_p2_bytes = [message[19], message[20]];
    let flags_p2 = u16::from_le_bytes(flags_p2_bytes);

    let p2 = [
        message[21],
        message[22],
        message[23],
        message[24],
        message[25],
    ];
    return Ok(SubdeviceDiagnosisEntry {
        diag_code,
        flags,
        text_id,
        timestamp,
        data_type_p1,
        p1,
        flags_p2,
        p2,
    });
}

/// Expects the SubdeviceRef to be the Coupler or any other device with a diag history index
pub async fn get_most_recent_diagnosis_message(
    device: &SubDeviceRef<'_, &SubDevice>,
) -> Option<String> {
    let new_message_available = device
        .sdo_read::<bool>(DIAGNOSIS_HISTORY_INDEX, NEW_MESSAGE_AVAILABLE)
        .await;
    let new_message_available = match new_message_available {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(
                "get_most_recent_diagnosis_message: Failed to read new_message_available {:?}",
                e
            );
            return None;
        }
    };

    if !new_message_available {
        tracing::info!("get_most_recent_diagnosis_message: No Diagnosis Messages found");
        // No messages available
        return None;
    }

    let newest_message_index = device
        .sdo_read::<u8>(DIAGNOSIS_HISTORY_INDEX, NEWEST_MESSAGE)
        .await;
    let newest_message_index = match newest_message_index {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(
                "get_most_recent_diagnosis_message: Failed to read newest_message_index {:?}",
                e
            );
            return None;
        }
    };

    tracing::info!(
        "get_most_recent_diagnosis_message: Reading newest message at index: {}",
        newest_message_index
    );
    let res = device
        .sdo_read::<[u8; DIAG_MESSAGE_LENGTH]>(DIAGNOSIS_HISTORY_INDEX, newest_message_index)
        .await;
    let message: [u8; DIAG_MESSAGE_LENGTH] = match res {
        Ok(res) => res,
        Err(_) => {
            tracing::error!("get_most_recent_diagnosis_message: Failed to read Diagnosis Message");
            return None;
        }
    };
    let message = convert_raw_diagnosis_bytes(message);
    let message = match message {
        Ok(msg) => msg,
        Err(e) => {
            tracing::error!(
                "get_most_recent_diagnosis_message: Failed to convert Diagnosis Message bytes to struct {:?}",
                e
            );
            return None;
        }
    };

    tracing::info!(
        "get_most_recent_diagnosis_message: {}",
        message.to_pretty_string()
    );
    return Some(message.to_pretty_string());
}
