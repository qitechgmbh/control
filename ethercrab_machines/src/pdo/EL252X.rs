use super::{PdoObject, RxPdoObject, TxPdoObject};

#[derive(Debug, Clone, Default)]
pub struct PtoStatus {
    pub status_select_end_counter: bool,
    pub status_ramp_active: bool,
    pub status_input_t: bool,
    pub status_input_z: bool,
    pub status_error: bool,
    pub status_sync_error: bool,
    pub status_txpdo_toggle: bool,
}

impl PdoObject for PtoStatus {
    fn get_sdo_index(&self) -> u16 {
        0x1A01
    }

    fn size(&self) -> usize {
        16
    }
}

impl TxPdoObject for PtoStatus {
    fn read(&mut self, buffer: &[u8]) {
        self.status_select_end_counter = buffer[0] & 0b0000_0001 != 0;
        self.status_ramp_active = buffer[0] & 0b0000_0010 != 0;
        self.status_input_t = buffer[0] & 0b0001_0000 != 0;
        self.status_input_z = buffer[0] & 0b0010_0000 != 0;
        self.status_error = buffer[0] & 0b0100_0000 != 0;

        self.status_sync_error = buffer[1] & 0b0010_0000 != 0;
        self.status_txpdo_toggle = buffer[1] & 0b1000_0000 != 0;
    }
}

#[derive(Debug, Clone, Default)]
pub struct EncStatus {
    pub status_set_counter_done: bool,
    pub status_counter_underflow: bool,
    pub status_counter_overflow: bool,
    pub status_sync_error: bool,
    pub status_txpdo_toggle: bool,
    pub counter_value: u32,
}

impl PdoObject for EncStatus {
    fn get_sdo_index(&self) -> u16 {
        0x1A05
    }

    fn size(&self) -> usize {
        64
    }
}

impl TxPdoObject for EncStatus {
    fn read(&mut self, buffer: &[u8]) {
        self.status_set_counter_done = buffer[0] & 0b0000_0100 != 0;
        self.status_counter_underflow = buffer[0] & 0b0000_100 != 0;
        self.status_counter_overflow = buffer[0] & 0b0001_0000 != 0;

        self.status_sync_error = buffer[1] & 0b0010_0000 != 0;
        self.status_txpdo_toggle = buffer[1] & 0b1000_0000 != 0;

        self.counter_value = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    }
}

#[derive(Debug, Clone, Default)]
pub struct PtoControl {
    pub control_frequency_select: bool,
    pub control_disble_ramp: bool,
    pub control_go_counter: bool,
    pub frequency_value: u32,
}

impl PdoObject for PtoControl {
    fn get_sdo_index(&self) -> u16 {
        0x1601
    }

    fn size(&self) -> usize {
        32
    }
}

impl RxPdoObject for PtoControl {
    fn write(&self, buffer: &mut [u8]) {
        buffer[0] = 0u8
            | (self.control_frequency_select as u8)
            | (self.control_disble_ramp as u8) << 1
            | (self.control_go_counter as u8) << 2;

        buffer[2..=3].copy_from_slice(&self.frequency_value.to_be_bytes());
    }
}

#[derive(Debug, Clone, Default)]
pub struct PtoTarget {
    pub target_counter_value: u32,
}

impl PdoObject for PtoTarget {
    fn get_sdo_index(&self) -> u16 {
        0x1607
    }

    fn size(&self) -> usize {
        32
    }
}

impl RxPdoObject for PtoTarget {
    fn write(&self, buffer: &mut [u8]) {
        buffer[4..=7].copy_from_slice(&self.target_counter_value.to_be_bytes());
    }
}

#[derive(Debug, Clone, Default)]
pub struct EncControl {
    pub control_set_counter: bool,
    pub set_counter_value: u32,
}

impl PdoObject for EncControl {
    fn get_sdo_index(&self) -> u16 {
        0x1605
    }

    fn size(&self) -> usize {
        32
    }
}

impl RxPdoObject for EncControl {
    fn write(&self, buffer: &mut [u8]) {
        buffer[8] = 0u8 | (self.control_set_counter as u8) << 2;

        buffer[10..=13].copy_from_slice(&self.set_counter_value.to_be_bytes());
    }
}
