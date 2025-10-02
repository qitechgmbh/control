use std::{fs::File, io::Read};

/// Typically 0x28, 0x29, 0x2A, 0x2B
#[derive(Debug)]
pub struct DeviceMachineIdentification {
    pub vendor: u16,
    pub machine: u16,
    pub serial: u16,
    pub role: u16,
}

/// Identity
/// - `0x8`: `vendor_id`
/// - `0xA`: `product_code`
/// - `0xC`: `revision_number`
/// - `0xE`: `serial_number`
#[derive(Debug)]
pub struct Identity {
    pub vendor_id: u32,
    pub product_code: u32,
    pub revision_number: u32,
    pub serial_number: u32,
}

/// 0x0 to 0x7 - ESC Configuration
#[derive(Debug)]
pub struct EscConfiguratoin {
    pub pdi_control: u16,         // 0x0: PDI Control
    pub pdi_configuration: u16,   // 0x1: PDI Configuration
    pub sync_impulse_length: u16, // 0x2: Sync Impulse Length
    pub pdi_configuration2: u16,  // 0x3: PDI Configuration 2
    pub station_alias: u16,       // 0x4: Configured Station Alias
    pub checksum: u16,            // 0x7: Checksum (0x5-0x6 are reserved)
}

/// Hardware delays 0x10 to 0x14
#[derive(Debug)]
pub struct HardwareDelays {
    pub physical_read_offset: u16,  // 0x10: Physical Read Offset
    pub physical_write_offset: u16, // 0x11: Physical Write Offset
    pub device_emulation: u16,      // 0x12: Device Emulation
    pub reserved1: u16,             // 0x13: Reserved
    pub reserved2: u16,             // 0x14: Reserved
}

/// Bootstrap mailbox configuration 0x15 to 0x17
#[derive(Debug)]
pub struct BootstrapMailboxConfig {
    pub standard_receive_mailbox_offset: u16, // 0x15: Standard Receive Mailbox Offset
    pub standard_receive_mailbox_size: u16,   // 0x16: Standard Receive Mailbox Size
    pub standard_send_mailbox_offset: u16,    // 0x17: Standard Send Mailbox Offset
}

/// Mailbox sync manager configuration 0x18 to 0x1A
#[derive(Debug)]
pub struct MailboxSyncManConfig {
    pub standard_send_mailbox_size: u16, // 0x18: Standard Send Mailbox Size
    pub mailbox_protocol: u16,           // 0x19: Supported Mailbox Protocols
    pub size_and_flags: u16,             // 0x1A: Size and Flags
}
/// Standard FMMU configuration (0x1B to 0x1F)
#[derive(Debug)]
pub struct FmmuConfiguration {
    pub fmmu0: u16,    // 0x1B: FMMU0 usage
    pub fmmu1: u16,    // 0x1C: FMMU1 usage
    pub fmmu2: u16,    // 0x1D: FMMU2 usage
    pub fmmu3: u16,    // 0x1E: FMMU3 usage
    pub reserved: u16, // 0x1F: Reserved
}

/// Standard Sync Manager configuration (0x20 to 0x27)
#[derive(Debug)]
pub struct SyncManagerConfiguration {
    pub sm0_config: SyncManagerChannelConfig, // 0x20-0x21: SM0 (Mailbox Out)
    pub sm1_config: SyncManagerChannelConfig, // 0x22-0x23: SM1 (Mailbox In)
    pub sm2_config: SyncManagerChannelConfig, // 0x24-0x25: SM2 (Process Data Out)
    pub sm3_config: SyncManagerChannelConfig, // 0x26-0x27: SM3 (Process Data In)
}

#[derive(Debug)]
pub struct SyncManagerChannelConfig {
    pub physical_start_address: u16, // Physical start address
    pub length_and_control: u16,     // Length and control settings
}

/// Distributed Clocks configuration (0x30 to 0x3F)
#[derive(Debug)]
pub struct DistributedClocksConfig {
    pub dc_activation: u16,    // 0x30: DC activation register
    pub sync0_cycle_time: u32, // 0x31-0x32: SYNC0 cycle time
    pub sync0_shift_time: u32, // 0x33-0x34: SYNC0 shift time
    pub sync1_cycle_time: u32, // 0x35-0x36: SYNC1 cycle time
    pub sync1_shift_time: u32, // 0x37-0x38: SYNC1 shift time
    pub reserved: [u16; 7],    // 0x39-0x3F: Reserved
}

/// String repository information (starts after standard registers)
#[derive(Debug)]
pub struct StringRepository {
    pub string_count: usize,
    pub strings: Vec<String>, // Extracted strings from EEPROM
}

/// Categories (0x40 to 0x7F - device profile area)
#[derive(Debug)]
pub struct DeviceProfile {
    pub category_type: u16, // 0x40: Category type
    pub data: Vec<u16>,     // Profile-specific data
}

/// Extended device information
#[derive(Debug)]
pub struct ExtendedInformation {
    pub device_profiles: Vec<DeviceProfile>,
    pub string_repository: Option<StringRepository>,
}

#[derive(Debug)]
pub struct EepromData {
    pub device_machine_identification: DeviceMachineIdentification,
    pub identity: Identity,
    pub esc_configuration: EscConfiguratoin,
    pub hardware_delays: HardwareDelays,
    pub bootstrap_mailbox_config: BootstrapMailboxConfig,
    pub mailbox_sync_man_config: MailboxSyncManConfig,
    pub fmmu_configuration: FmmuConfiguration,
    pub sync_manager_configuration: SyncManagerConfiguration,
    pub distributed_clocks_config: DistributedClocksConfig,
    pub extended_information: ExtendedInformation,
}

fn extract_string_sections(bytes: &[u8]) -> Vec<String> {
    let mut result = Vec::new();

    // Minimum sequence of ASCII characters to consider as a string
    const MIN_STRING_LENGTH: usize = 4;

    let mut current_string = Vec::new();
    let mut in_string = false;

    for &byte in bytes {
        if (32..127).contains(&byte) {
            // Printable ASCII character
            current_string.push(byte);
            in_string = true;
        } else if in_string {
            // End of string - only keep strings that are long enough
            if current_string.len() >= MIN_STRING_LENGTH {
                if let Ok(s) = String::from_utf8(current_string.clone()) {
                    // Only add strings that are not already in the list
                    let clean = s.trim();
                    if !clean.is_empty()
                        && !result
                            .iter()
                            .any(|existing: &String| existing.contains(clean))
                    {
                        result.push(clean.to_string());
                    }
                }
            }
            current_string.clear();
            in_string = false;
        }
    }

    // Handle case where string extends to end of data
    if in_string && current_string.len() >= MIN_STRING_LENGTH {
        if let Ok(s) = String::from_utf8(current_string) {
            let clean = s.trim();
            if !clean.is_empty()
                && !result
                    .iter()
                    .any(|existing: &String| existing.contains(clean))
            {
                result.push(clean.to_string());
            }
        }
    }

    // Post-process: break apart large strings that might contain multiple components
    let mut processed_results = Vec::new();
    for s in result {
        // If a string is very long, it might contain multiple logical parts
        if s.len() > 30 {
            // Try to split at common separators
            let parts: Vec<&str> = s
                .split(['!', '(', ')', '.', ','])
                .map(|p| p.trim())
                .filter(|p| !p.is_empty() && p.len() >= MIN_STRING_LENGTH)
                .collect();

            if parts.len() > 1 {
                for part in parts {
                    if !processed_results
                        .iter()
                        .any(|existing: &String| existing.contains(part))
                    {
                        processed_results.push(part.to_string());
                    }
                }
            } else {
                processed_results.push(s);
            }
        } else {
            processed_results.push(s);
        }
    }

    processed_results
}

impl std::fmt::Display for EepromData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Extract device name and info from strings if available
        let device_name = if let Some(repo) = &self.extended_information.string_repository {
            repo.strings
                .iter()
                .find(|s| s.contains("EL") && s.chars().any(|c| c.is_ascii_digit()))
                .cloned()
                .unwrap_or_else(|| "Unknown Device".to_string())
        } else {
            "Unknown Device".to_string()
        };

        // Format the header with device name
        writeln!(
            f,
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓"
        )?;
        writeln!(
            f,
            "┃                      ETHERCAT DEVICE EEPROM                 ┃"
        )?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(f, "┃ Device: {:<51} ┃", device_name)?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;

        // Device Identity
        writeln!(
            f,
            "┃ DEVICE IDENTITY                                             ┃"
        )?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃  Vendor ID:       0x{:08X}                                ┃",
            self.identity.vendor_id
        )?;
        writeln!(
            f,
            "┃  Product Code:    0x{:08X}                                ┃",
            self.identity.product_code
        )?;
        writeln!(
            f,
            "┃  Revision Number: 0x{:08X}                                ┃",
            self.identity.revision_number
        )?;
        writeln!(
            f,
            "┃  Serial Number:   0x{:08X}                                ┃",
            self.identity.serial_number
        )?;

        // Machine Identification
        if self.device_machine_identification.vendor != 0
            || self.device_machine_identification.machine != 0
            || self.device_machine_identification.serial != 0
            || self.device_machine_identification.role != 0
        {
            writeln!(
                f,
                "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
            )?;
            writeln!(
                f,
                "┃ MACHINE IDENTIFICATION                                      ┃"
            )?;
            writeln!(
                f,
                "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
            )?;
            writeln!(
                f,
                "┃  Vendor:  0x{:04X}                                            ┃",
                self.device_machine_identification.vendor
            )?;
            writeln!(
                f,
                "┃  Machine: 0x{:04X}                                            ┃",
                self.device_machine_identification.machine
            )?;
            writeln!(
                f,
                "┃  Serial:  0x{:04X}                                            ┃",
                self.device_machine_identification.serial
            )?;
            writeln!(
                f,
                "┃  Role:    0x{:04X}                                            ┃",
                self.device_machine_identification.role
            )?;
        }

        // ESC Configuration
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃ ESC CONFIGURATION                                           ┃"
        )?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃  PDI Control:        0x{:04X}                                 ┃",
            self.esc_configuration.pdi_control
        )?;
        writeln!(
            f,
            "┃  PDI Configuration:  0x{:04X}                                 ┃",
            self.esc_configuration.pdi_configuration
        )?;
        writeln!(
            f,
            "┃  Sync Impulse Len:   0x{:04X}                                 ┃",
            self.esc_configuration.sync_impulse_length
        )?;
        writeln!(
            f,
            "┃  PDI Configuration2: 0x{:04X}                                 ┃",
            self.esc_configuration.pdi_configuration2
        )?;
        writeln!(
            f,
            "┃  Station Alias:      0x{:04X}                                 ┃",
            self.esc_configuration.station_alias
        )?;
        writeln!(
            f,
            "┃  Checksum:           0x{:04X}                                 ┃",
            self.esc_configuration.checksum
        )?;

        // Mailbox Configuration
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃ MAILBOX CONFIGURATION                                       ┃"
        )?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃  Bootstrap Mailbox:                                         ┃"
        )?;
        writeln!(
            f,
            "┃    Receive Offset: 0x{:04X}                                   ┃",
            self.bootstrap_mailbox_config
                .standard_receive_mailbox_offset
        )?;
        writeln!(
            f,
            "┃    Receive Size:   0x{:04X}                                   ┃",
            self.bootstrap_mailbox_config.standard_receive_mailbox_size
        )?;
        writeln!(
            f,
            "┃    Send Offset:    0x{:04X}                                   ┃",
            self.bootstrap_mailbox_config.standard_send_mailbox_offset
        )?;
        writeln!(
            f,
            "┃  Standard Mailbox:                                          ┃"
        )?;
        writeln!(
            f,
            "┃    Send Size:      0x{:04X}                                   ┃",
            self.mailbox_sync_man_config.standard_send_mailbox_size
        )?;
        writeln!(
            f,
            "┃    Protocols:      0x{:04X}                                   ┃",
            self.mailbox_sync_man_config.mailbox_protocol
        )?;
        writeln!(
            f,
            "┃    Size & Flags:   0x{:04X}                                   ┃",
            self.mailbox_sync_man_config.size_and_flags
        )?;

        // FMMU Configuration
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃ FMMU CONFIGURATION                                          ┃"
        )?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃  FMMU0: 0x{:04X}                                              ┃",
            self.fmmu_configuration.fmmu0
        )?;
        writeln!(
            f,
            "┃  FMMU1: 0x{:04X}                                              ┃",
            self.fmmu_configuration.fmmu1
        )?;
        writeln!(
            f,
            "┃  FMMU2: 0x{:04X}                                              ┃",
            self.fmmu_configuration.fmmu2
        )?;
        writeln!(
            f,
            "┃  FMMU3: 0x{:04X}                                              ┃",
            self.fmmu_configuration.fmmu3
        )?;

        // Sync Manager Configuration
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃ SYNC MANAGER CONFIGURATION                                  ┃"
        )?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃  SM0: Address=0x{:04X}, Control=0x{:04X}                        ┃",
            self.sync_manager_configuration
                .sm0_config
                .physical_start_address,
            self.sync_manager_configuration
                .sm0_config
                .length_and_control
        )?;
        writeln!(
            f,
            "┃  SM1: Address=0x{:04X}, Control=0x{:04X}                        ┃",
            self.sync_manager_configuration
                .sm1_config
                .physical_start_address,
            self.sync_manager_configuration
                .sm1_config
                .length_and_control
        )?;
        writeln!(
            f,
            "┃  SM2: Address=0x{:04X}, Control=0x{:04X}                        ┃",
            self.sync_manager_configuration
                .sm2_config
                .physical_start_address,
            self.sync_manager_configuration
                .sm2_config
                .length_and_control
        )?;
        writeln!(
            f,
            "┃  SM3: Address=0x{:04X}, Control=0x{:04X}                        ┃",
            self.sync_manager_configuration
                .sm3_config
                .physical_start_address,
            self.sync_manager_configuration
                .sm3_config
                .length_and_control
        )?;

        // Distributed Clocks
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃ DISTRIBUTED CLOCKS                                          ┃"
        )?;
        writeln!(
            f,
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
        )?;
        writeln!(
            f,
            "┃  Activation:       0x{:04X}                                   ┃",
            self.distributed_clocks_config.dc_activation
        )?;
        writeln!(
            f,
            "┃  SYNC0 Cycle Time: 0x{:08X}                               ┃",
            self.distributed_clocks_config.sync0_cycle_time
        )?;
        writeln!(
            f,
            "┃  SYNC0 Shift Time: 0x{:08X}                               ┃",
            self.distributed_clocks_config.sync0_shift_time
        )?;
        writeln!(
            f,
            "┃  SYNC1 Cycle Time: 0x{:08X}                               ┃",
            self.distributed_clocks_config.sync1_cycle_time
        )?;
        writeln!(
            f,
            "┃  SYNC1 Shift Time: 0x{:08X}                               ┃",
            self.distributed_clocks_config.sync1_shift_time
        )?;

        // Device Profile
        if !self.extended_information.device_profiles.is_empty() {
            writeln!(
                f,
                "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
            )?;
            writeln!(
                f,
                "┃ DEVICE PROFILE                                              ┃"
            )?;
            writeln!(
                f,
                "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
            )?;

            for (i, profile) in self.extended_information.device_profiles.iter().enumerate() {
                writeln!(
                    f,
                    "┃  Profile {}: Type=0x{:04X}, Size={:<3}                           ┃",
                    i,
                    profile.category_type,
                    profile.data.len()
                )?;
            }
        }

        // String Repository
        if let Some(repo) = &self.extended_information.string_repository {
            if !repo.strings.is_empty() {
                writeln!(
                    f,
                    "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
                )?;
                writeln!(
                    f,
                    "┃ STRING REPOSITORY                                           ┃"
                )?;
                writeln!(
                    f,
                    "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
                )?;

                for (i, string) in repo.strings.iter().enumerate() {
                    // Truncate very long strings
                    let display_string = if string.len() > 50 {
                        format!("{}...", &string[0..47])
                    } else {
                        string.clone()
                    };

                    // Ensure consistent spacing for string index
                    writeln!(f, "┃  {:<2}: {:<54} ┃", i, display_string)?;
                }
            }
        }

        writeln!(
            f,
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛"
        )?;

        Ok(())
    }
}

/// Create EEPROM data structure according to standard EtherCAT specification
impl From<&[u8]> for EepromData {
    fn from(bytes: &[u8]) -> Self {
        // Convert byte array to word array with proper endianness and bounds checking
        let words: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        // Standard section parsing with bounds checking
        let identity = Identity {
            vendor_id: if words.len() > 0x9 {
                u32_from_u16(words[0x8], words[0x9])
            } else {
                0
            },
            product_code: if words.len() > 0xB {
                u32_from_u16(words[0xA], words[0xB])
            } else {
                0
            },
            revision_number: if words.len() > 0xD {
                u32_from_u16(words[0xC], words[0xD])
            } else {
                0
            },
            serial_number: if words.len() > 0xF {
                u32_from_u16(words[0xE], words[0xF])
            } else {
                0
            },
        };

        // Basic device information with bounds checking
        let device_info = DeviceMachineIdentification {
            vendor: if words.len() > 0x28 { words[0x0028] } else { 0 },
            machine: if words.len() > 0x29 { words[0x0029] } else { 0 },
            serial: if words.len() > 0x2A { words[0x002a] } else { 0 },
            role: if words.len() > 0x2B { words[0x002b] } else { 0 },
        };

        // Extract string sections using sliding window approach
        let strings = extract_string_sections(bytes);

        // Create string repository
        let string_repository = StringRepository {
            string_count: strings.len(),
            strings,
        };

        // Parse DC configuration with bounds checking
        let dc_config = DistributedClocksConfig {
            dc_activation: if words.len() > 0x30 { words[0x30] } else { 0 },
            sync0_cycle_time: if words.len() > 0x32 {
                u32_from_u16(words[0x31], words[0x32])
            } else {
                0
            },
            sync0_shift_time: if words.len() > 0x34 {
                u32_from_u16(words[0x33], words[0x34])
            } else {
                0
            },
            sync1_cycle_time: if words.len() > 0x36 {
                u32_from_u16(words[0x35], words[0x36])
            } else {
                0
            },
            sync1_shift_time: if words.len() > 0x38 {
                u32_from_u16(words[0x37], words[0x38])
            } else {
                0
            },
            reserved: [
                if words.len() > 0x39 { words[0x39] } else { 0 },
                if words.len() > 0x3A { words[0x3A] } else { 0 },
                if words.len() > 0x3B { words[0x3B] } else { 0 },
                if words.len() > 0x3C { words[0x3C] } else { 0 },
                if words.len() > 0x3D { words[0x3D] } else { 0 },
                if words.len() > 0x3E { words[0x3E] } else { 0 },
                if words.len() > 0x3F { words[0x3F] } else { 0 },
            ],
        };

        // Parse and categorize device profile sections
        let mut device_profiles = Vec::new();

        // Category at 0x40 - General category
        if words.len() > 0x40 {
            let category_type = words[0x40];
            let end_index = std::cmp::min(0x68, words.len());

            // Only add if we have actual data
            if end_index > 0x41 {
                let device_profile = DeviceProfile {
                    category_type,
                    data: words[0x41..end_index].to_vec(),
                };
                device_profiles.push(device_profile);
            }
        }

        // Extract FMMU configuration with bounds checking
        let fmmu_config = FmmuConfiguration {
            fmmu0: if words.len() > 0x1B { words[0x1B] } else { 0 },
            fmmu1: if words.len() > 0x1C { words[0x1C] } else { 0 },
            fmmu2: if words.len() > 0x1D { words[0x1D] } else { 0 },
            fmmu3: if words.len() > 0x1E { words[0x1E] } else { 0 },
            reserved: if words.len() > 0x1F { words[0x1F] } else { 0 },
        };

        // SyncManager configuration
        let sm_config = SyncManagerConfiguration {
            sm0_config: SyncManagerChannelConfig {
                physical_start_address: if words.len() > 0x20 { words[0x20] } else { 0 },
                length_and_control: if words.len() > 0x21 { words[0x21] } else { 0 },
            },
            sm1_config: SyncManagerChannelConfig {
                physical_start_address: if words.len() > 0x22 { words[0x22] } else { 0 },
                length_and_control: if words.len() > 0x23 { words[0x23] } else { 0 },
            },
            sm2_config: SyncManagerChannelConfig {
                physical_start_address: if words.len() > 0x24 { words[0x24] } else { 0 },
                length_and_control: if words.len() > 0x25 { words[0x25] } else { 0 },
            },
            sm3_config: SyncManagerChannelConfig {
                physical_start_address: if words.len() > 0x26 { words[0x26] } else { 0 },
                length_and_control: if words.len() > 0x27 { words[0x27] } else { 0 },
            },
        };

        // Construct the final EEPROM data object with bounds checking
        Self {
            device_machine_identification: device_info,
            identity,
            esc_configuration: EscConfiguratoin {
                pdi_control: if !words.is_empty() { words[0x0] } else { 0 },
                pdi_configuration: if words.len() > 0x1 { words[0x1] } else { 0 },
                sync_impulse_length: if words.len() > 0x2 { words[0x2] } else { 0 },
                pdi_configuration2: if words.len() > 0x3 { words[0x3] } else { 0 },
                station_alias: if words.len() > 0x4 { words[0x4] } else { 0 },
                checksum: if words.len() > 0x7 { words[0x7] } else { 0 },
            },
            hardware_delays: HardwareDelays {
                physical_read_offset: if words.len() > 0x10 { words[0x10] } else { 0 },
                physical_write_offset: if words.len() > 0x11 { words[0x11] } else { 0 },
                device_emulation: if words.len() > 0x12 { words[0x12] } else { 0 },
                reserved1: if words.len() > 0x13 { words[0x13] } else { 0 },
                reserved2: if words.len() > 0x14 { words[0x14] } else { 0 },
            },
            bootstrap_mailbox_config: BootstrapMailboxConfig {
                standard_receive_mailbox_offset: if words.len() > 0x15 { words[0x15] } else { 0 },
                standard_receive_mailbox_size: if words.len() > 0x16 { words[0x16] } else { 0 },
                standard_send_mailbox_offset: if words.len() > 0x17 { words[0x17] } else { 0 },
            },
            mailbox_sync_man_config: MailboxSyncManConfig {
                standard_send_mailbox_size: if words.len() > 0x18 { words[0x18] } else { 0 },
                mailbox_protocol: if words.len() > 0x19 { words[0x19] } else { 0 },
                size_and_flags: if words.len() > 0x1A { words[0x1A] } else { 0 },
            },
            fmmu_configuration: fmmu_config,
            sync_manager_configuration: sm_config,
            distributed_clocks_config: dc_config,
            extended_information: ExtendedInformation {
                device_profiles,
                string_repository: Some(string_repository),
            },
        }
    }
}

/// Helper function for parsing EEPROM data from a file
pub async fn read_eeprom(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the file
    let mut file = File::open(file)?;
    let mut byte_buffer = Vec::new();
    file.read_to_end(&mut byte_buffer)?;

    // Parse the EEPROM data directly from bytes
    let eeprom_data = EepromData::from(byte_buffer.as_slice());

    // Print the parsed data
    println!("{}", eeprom_data);

    Ok(())
}

/// turn &[u16, u16] little endian into u32
pub const fn u32_from_u16(start_word: u16, end_word: u16) -> u32 {
    let start_bytes = start_word.to_le_bytes();
    let end_bytes = end_word.to_le_bytes();
    u32::from_le_bytes([start_bytes[0], start_bytes[1], end_bytes[0], end_bytes[1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_from_u16() {
        let start_word = 0x5678;
        let end_word = 0x1234;
        let result = u32_from_u16(start_word, end_word);
        assert_eq!(result, 0x12345678);
    }
}
