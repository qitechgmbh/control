use super::{RxPdoObject, TxPdoObject};
use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

/// # `EncStatusCompact`
/// 48 bits / 6 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 48)]
pub struct EncStatusCompact {
    /// # 6000:01
    /// (6000:02 on EL7031-0030)
    /// The counter value was latched with the C track.
    pub latch_extern_valid: bool,

    /// # 6000:02
    /// (6000:03 on EL7031-0030)
    /// The counter value was stored via the external latch.
    pub set_counter_done: bool,

    /// # 6000:03
    /// (6000:04 on EL7031-0030)
    /// The counter was set.
    pub counter_underflow: bool,

    /// # 6000:04
    /// (6000:05 on EL7031-0030)
    /// Counter underflow.
    pub counter_overflow: bool,

    /// # 6000:0D
    /// Status of the external latch input.
    pub status_of_extern_latch: bool,

    /// # 6000:0E
    /// The Sync error bit is only required for DC mode. It indicates whether a synchronization error has occurred during the previous cycle.
    pub sync_error: bool,

    /// # 6000:10
    /// The TxPDO toggle is toggled by the slave when the data of the associated TxPDO is updated.
    pub txpdo_toggle: bool,

    /// # 6000:11
    /// The counter value.
    pub counter_value: u16,

    /// # 6000:12
    /// The latch value.
    pub latch_value: u16,
}

impl TxPdoObject for EncStatusCompact {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // txpdo toggle
        // Offset 1.7
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        // Offset 0.1
        self.latch_extern_valid = bits[1];
        // Offset 0.2
        self.set_counter_done = bits[2];
        // Offset 0.3
        self.counter_underflow = bits[3];
        // Offset 0.4
        self.counter_overflow = bits[4];
        // Offset 1.4
        self.status_of_extern_latch = bits[8 + 4];
        // Offset 1.5
        self.sync_error = bits[8 + 5];
        // Offset 2.0
        self.counter_value = bits[16..16 + 16].load_le();
        // Offset 4.0
        self.latch_value = bits[32..32 + 16].load_le();
    }
}

/// # `EncStatus`
/// 80 bits / 10 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 80)]
pub struct EncStatus {
    /// # 6000:01
    /// (6000:02 on EL7031-0030)
    /// The counter value was stored via the external latch.
    pub latch_extern_valid: bool,

    /// # 6000:02
    /// (6000:03 on EL7031-0030)
    /// The counter was set.
    pub set_counter_done: bool,

    /// # 6000:03
    /// (6000:04 on EL7031-0030)
    /// Counter underflow.
    pub counter_underflow: bool,

    /// # 6000:04
    /// (6000:05 on EL7031-0030)
    /// Counter overflow.
    pub counter_overflow: bool,

    /// # 6000:0D
    /// Status of the external latch input.
    pub status_of_extern_latch: bool,

    /// # 6000:0E
    /// The Sync error bit is only required for DC mode. It indicates whether a synchronization error has occurred during the previous cycle.
    pub sync_error: bool,

    /// # 6000:10
    /// The TxPDO toggle is toggled by the slave when the data of the associated TxPDO is updated.
    pub txpdo_toggle: bool,

    /// # 6000:11
    /// The counter value.
    pub counter_value: u32,

    /// # 6000:12
    /// The latch value.
    pub latch_value: u32,
}

impl TxPdoObject for EncStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 1.7
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        // Offset 0.1
        self.latch_extern_valid = bits[1];
        // Offset 0.2
        self.set_counter_done = bits[2];
        // Offset 0.3
        self.counter_underflow = bits[3];
        // Offset 0.4
        self.counter_overflow = bits[4];
        // Offset 1.4
        self.status_of_extern_latch = bits[8 + 4];
        // Offset 1.5
        self.sync_error = bits[8 + 5];
        // Offset 2.0
        self.counter_value = bits[16..16 + 32].load_le();
        // Offset 6.0
        self.latch_value = bits[48..48 + 32].load_le();
    }
}

/// # `EncTimestampCompact`
/// 32 bits / 4 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct EncTimestampCompact {
    /// # 6000:16
    /// Time stamp of the last counter change.
    pub timestamp: u32,
}

impl TxPdoObject for EncTimestampCompact {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 0.0
        self.timestamp = bits[0..32].load_le();
    }
}

/// # `EncTimestamp`
/// 16 bits / 2 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct StmStatus {
    /// # 6010:01
    /// Driver stage is ready for enabling.
    pub ready_to_enable: bool,

    /// # 6010:02
    /// Driver stage is ready for operation.
    pub ready: bool,

    /// # 6010:03
    /// A warning has occurred.
    pub warning: bool,

    /// # 6010:04
    /// An error has occurred (see index 0xA010).
    pub error: bool,

    /// # 6010:05
    /// Motor turns in positive direction.
    pub moving_positive: bool,

    /// # 6010:06
    /// Motor turns in negative direction.
    pub moving_negative: bool,

    /// # 6010:07
    /// Reduced torque is active.
    pub torque_reduced: bool,

    /// # 6010:0C
    /// Digital input 1.
    pub digital_input_1: bool,

    /// # 6010:0D
    /// Digital input 2.
    pub digital_input_2: bool,

    /// # 6010:0E
    /// The Sync error bit is only required for DC mode. It indicates whether a synchronization error has occurred during the previous cycle.
    pub sync_error: bool,

    /// # 6010:10
    /// The TxPDO toggle is toggled by the slave when the data of the associated TxPDO is updated.
    pub txpdo_toggle: bool,
}

impl TxPdoObject for StmStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 1.7
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        // Offset 0.0
        self.ready_to_enable = bits[0];
        // Offset 0.1
        self.ready = bits[1];
        // Offset 0.2
        self.warning = bits[2];
        // Offset 0.3
        self.error = bits[3];
        // Offset 0.4
        self.moving_positive = bits[4];
        // Offset 0.5
        self.moving_negative = bits[5];
        // Offset 0.6
        self.torque_reduced = bits[6];
        // Offset 1.3
        self.digital_input_1 = bits[8 + 3];
        // Offset 1.4
        self.digital_input_2 = bits[8 + 4];
        // Offset 1.5
        self.sync_error = bits[8 + 5];
    }
}

/// # `StmSynchronInfoData`
/// 32 bits / 4 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct StmSynchronInfoData {
    /// # 6010:11
    /// Synchronous information (selection via subindex 0x8012:11).
    pub info_data_1: u16,

    /// # 6010:12
    /// Synchronous information (selection via subindex 0x8012:19).
    pub info_data_2: u16,
}

impl TxPdoObject for StmSynchronInfoData {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 0.0
        self.info_data_1 = bits[0..16].load_le();
        // Offset 2.0
        self.info_data_2 = bits[16..32].load_le();
    }
}

/// # `PosStatusCompact`
/// 16 bits / 2 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct PosStatusCompact {
    /// # 6020:01
    /// A current travel command is active.
    pub busy: bool,

    /// # 6020:02
    /// Motor has arrived at target.
    pub in_target: bool,

    /// # 6020:03
    /// A warning has occurred.
    pub warning: bool,

    /// # 6020:04
    /// An error has occurred.
    pub error: bool,

    /// # 6020:05
    /// Motor is calibrated.
    pub calibrated: bool,

    /// # 6020:06
    /// Motor is in the acceleration phase.
    pub accelerate: bool,

    /// # 6020:07
    /// Motor is in the deceleration phase.
    pub decelerate: bool,

    /// # 6020:08
    /// Ready to execute.
    pub ready_to_execute: bool,
}

impl TxPdoObject for PosStatusCompact {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 0.0
        self.busy = bits[0];
        // Offset 0.1
        self.in_target = bits[1];
        // Offset 0.2
        self.warning = bits[2];
        // Offset 0.3
        self.error = bits[3];
        // Offset 0.4
        self.calibrated = bits[4];
        // Offset 0.5
        self.accelerate = bits[5];
        // Offset 0.6
        self.decelerate = bits[6];
        // Offset 0.7
        self.ready_to_execute = bits[7];
    }
}

/// # `PosStatus`
/// 96 bits / 12 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 96)]
pub struct PosStatus {
    /// # 6020:01
    /// A current travel command is active.
    pub busy: bool,

    /// # 6020:02
    /// Motor has arrived at target.
    pub in_target: bool,

    /// # 6020:03
    /// A warning has occurred.
    pub warning: bool,

    /// # 6020:04
    /// An error has occurred.
    pub error: bool,

    /// # 6020:05
    /// Motor is calibrated.
    pub calibrated: bool,

    /// # 6020:06
    /// Motor is in the acceleration phase.
    pub accelerate: bool,

    /// # 6020:07
    /// Motor is in the deceleration phase.
    pub decelerate: bool,

    /// # 6020:08
    /// Ready to execute.
    /// Not sure if this valie is only on EL7031-0030 or other too
    pub ready_to_execute: bool,

    /// # 6020:11
    /// Current target position of the travel command generator.
    pub actual_position: u32,

    /// # 6020:21
    /// Current set velocity of the travel command generator.
    pub actual_velocity: i16,

    /// # 6020:22
    /// Travel command time information.
    pub actual_drive_time: u32,
}

impl TxPdoObject for PosStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 0.0
        self.busy = bits[0];
        // Offset 0.1
        self.in_target = bits[1];
        // Offset 0.2
        self.warning = bits[2];
        // Offset 0.3
        self.error = bits[3];
        // Offset 0.4
        self.calibrated = bits[4];
        // Offset 0.5
        self.accelerate = bits[5];
        // Offset 0.6
        self.decelerate = bits[6];
        // Offset 2.0
        self.actual_position = bits[16..16 + 32].load_le();
        // Offset 6.0
        self.actual_velocity = bits[48..48 + 16].load_le();
        // Offset 8.0
        self.actual_drive_time = bits[64..64 + 32].load_le();
    }
}

/// # `StmInternalPosition`
/// 32 bits / 4 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct StmInternalPosition {
    /// # 6010:14
    /// Internal microstep position.
    pub internal_position: u32,
}

impl TxPdoObject for StmInternalPosition {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 0.0
        self.internal_position = bits[0..32].load_le();
    }
}

/// # `StmExternalPosition`
/// 32 bits / 4 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct StmExternalPosition {
    /// # 6010:15
    /// Encoder position.
    pub external_position: u32,
}

impl TxPdoObject for StmExternalPosition {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 0.0
        self.external_position = bits[0..32].load_le();
    }
}

/// # `PosActualPositionLag`
/// 32 bits / 4 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct PosActualPositionLag {
    /// # 6020:23
    /// Actual position lag.
    pub actual_position_lag: u32,
}

impl TxPdoObject for PosActualPositionLag {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Offset 0.0
        self.actual_position_lag = bits[0..32].load_le();
    }
}

/// # `EncControlCompact`
/// 32 bits / 4 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct EncControlCompact {
    /// # 7000:01
    /// (7000:02 on EL7031-0030)
    /// Activate latching via the C-track.
    pub enable_latch_extern_on_positive_edge: bool,

    /// # 7000:02
    /// (7000:03 on EL7031-0030)
    /// Activate external latch with positive edge.
    pub set_counter: bool,

    /// # 7000:03
    /// (7000:04 on EL7031-0030)
    /// Set the counter value.
    pub enable_latch_extern_on_negative_edge: bool,

    /// # 7000:11
    /// This is the counter value to be set via "Set counter".
    pub set_counter_value: u16,
}

impl RxPdoObject for EncControlCompact {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.1
        buffer.set(1, self.enable_latch_extern_on_positive_edge);
        // Offset 0.2
        buffer.set(2, self.set_counter);
        // Offset 0.3
        buffer.set(3, self.enable_latch_extern_on_negative_edge);
        // Offset 2.0
        buffer[16..16 + 16].store_le(self.set_counter_value);
    }
}

/// # `EncControl`
/// 48 bits / 6 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 48)]
pub struct EncControl {
    /// # 7000:01
    /// (7000:02 on EL7031-0030)
    /// Activate external latch with positive edge.
    pub enable_latch_extern_on_positive_edge: bool,

    /// # 7000:02
    /// (7000:03 on EL7031-0030)
    /// Activate external latch with positive edge.
    pub set_counter: bool,

    /// # 7000:03
    /// (7000:04 on EL7031-0030)
    /// Activate external latch with negative edge.
    pub enable_latch_extern_on_negative_edge: bool,

    /// # 7000:11
    /// (7000:12 on EL7031-0030)
    /// This is the counter value to be set via "Set counter".
    pub set_counter_value: u32,
}

impl RxPdoObject for EncControl {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.1
        buffer.set(1, self.enable_latch_extern_on_positive_edge);
        // Offset 0.2
        buffer.set(2, self.set_counter);
        // Offset 0.3
        buffer.set(3, self.enable_latch_extern_on_negative_edge);
        // Offset 2.0
        buffer[16..16 + 32].store_le(self.set_counter_value);
    }
}

/// # `StmControl`
/// 16 bits / 2 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct StmControl {
    /// # 7010:01
    /// Activates the output stage.
    pub enable: bool,

    /// # 7010:02
    /// All errors that may have occurred are reset by setting this bit (rising edge).
    pub reset: bool,

    /// # 7010:03
    /// Activation of reduced torque (coil current).
    pub reduce_torque: bool,
}

impl RxPdoObject for StmControl {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.0
        buffer.set(0, self.enable);
        // Offset 0.1
        buffer.set(1, self.reset);
        // Offset 0.2
        buffer.set(2, self.reduce_torque);
    }
}

/// # `StmPosition`
/// 32 bits / 4 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct StmPosition {
    /// # 7010:11
    /// Set position.
    pub position: u32,
}

impl RxPdoObject for StmPosition {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.0
        buffer[0..32].store_le(self.position);
    }
}

/// # `StmVelocity`
/// 16 bits / 2 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct StmVelocity {
    /// # 7010:21
    /// Set velocity.
    pub velocity: i16,
}

impl RxPdoObject for StmVelocity {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.0
        buffer[0..16].store_le(self.velocity);
    }
}

/// # `PosControlCompact`
/// 48 bits / 6 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 48)]
pub struct PosControlCompact {
    /// # 7020:01
    /// Start travel command (rising edge), or prematurely abort travel command (falling edge).
    pub execute: bool,

    /// # 7020:02
    /// Prematurely abort travel command with an emergency ramp (rising edge).
    pub emergency_stop: bool,

    /// # 7020:11
    /// Specification of the target position.
    pub target_position: u32,
}

impl RxPdoObject for PosControlCompact {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.0
        buffer.set(0, self.execute);
        // Offset 0.1
        buffer.set(1, self.emergency_stop);
        // Offset 2.0
        buffer[16..16 + 32].store_le(self.target_position);
    }
}

/// # `PosControl`
/// 112 bits / 14 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 112)]
pub struct PosControl {
    /// # 7020:01
    /// Start travel command (rising edge), or prematurely abort travel command (falling edge).
    pub execute: bool,

    /// # 7020:02
    /// Prematurely abort travel command with an emergency ramp (rising edge).
    pub emergency_stop: bool,

    /// # 7020:11
    /// Specification of the target position.
    pub target_position: u32,

    /// # 7020:21
    /// Specification of the maximum set velocity.
    pub target_velocity: i16,

    /// # 7020:22
    /// Specification of the start type (e.g. absolute, relative, endless plus/minus, etc.).
    pub start_type: u16,

    /// # 7020:23
    /// Specification of the acceleration.
    pub acceleration: u16,

    /// # 7020:24
    /// Specification of the deceleration.
    pub deceleration: u16,
}

impl RxPdoObject for PosControl {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.0
        buffer.set(0, self.execute);
        // Offset 0.1
        buffer.set(1, self.emergency_stop);
        // Offset 2.0
        buffer[16..16 + 32].store_le(self.target_position);
        // Offset 6.0
        buffer[48..48 + 16].store_le(self.target_velocity);
        // Offset 8.0
        buffer[64..64 + 16].store_le(self.start_type);
        // Offset 10.0
        buffer[80..80 + 16].store_le(self.acceleration);
        // Offset 12.0
        buffer[96..96 + 16].store_le(self.deceleration);
    }
}

/// # `PosControl2`
/// 112 bits / 14 bytes
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 112)]
pub struct PosControl2 {
    /// # 7021:01
    /// Start travel command (rising edge), or prematurely abort travel command (falling edge).
    pub execute: bool,

    /// # 7021:02
    /// Prematurely abort travel command with an emergency ramp (rising edge).
    pub emergency_stop: bool,

    /// # 7021:03
    /// Enable auto start.
    pub target_position: u32,

    /// # 7021:11
    /// Specification of the target position.
    pub target_velocity: i16,

    /// # 7021:21
    /// Specification of the maximum set velocity.
    pub start_type: u16,

    /// # 7021:22
    /// Specification of the start type (e.g. absolute, relative, endless plus/minus, etc.).
    pub acceleration: u16,

    /// # 7021:23
    /// Specification of the acceleration.
    pub deceleration: u16,
}

impl RxPdoObject for PosControl2 {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        // Offset 0.0
        buffer.set(0, self.execute);
        // Offset 0.1
        buffer.set(1, self.emergency_stop);
        // Offset 2.0
        buffer[16..16 + 32].store_le(self.target_position);
        // Offset 6.0
        buffer[48..48 + 16].store_le(self.target_velocity);
        // Offset 8.0
        buffer[64..64 + 16].store_le(self.start_type);
        // Offset 10.0
        buffer[80..80 + 16].store_le(self.acceleration);
        // Offset 12.0
        buffer[96..96 + 16].store_le(self.deceleration);
    }
}
