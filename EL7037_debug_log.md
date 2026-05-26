# EL7037 Driver Debug Log

Branch: `1365-implement-el7037-driver`

## Issues Found & Fixed

### 1. `stm_controller_2` wrote to non-existent `0x8013`
**Symptom:** `0x06040043` on SubDevice 0x1001 during stm_controller_2 write.
**Root cause:** EL7037 is single-channel; `0x8013` only exists on dual-channel EL7031.
**Fix:** Removed `stm_controller_2` field and its `write_config(device, 0x8013)` call from `EL7037Configuration`.
**File:** `ethercat-hal/src/devices/el7037/coe.rs`

### 2. `StmControllerConfiguration` wrote to `0x8011:03` (inner_window)
**Symptom:** `0x06040043` — EL7037 doc shows `0x8011:0 = 0x02` (only subindices 01, 02).
**Fix:** Commented out `sdo_write(base_index, 0x03, self.inner_window)` in shared `StmControllerConfiguration::write_config`.
**File:** `ethercat-hal/src/shared_config/el70x1.rs`

### 3. `StmFeatures::write_config` did not write `operation_mode` (0x8012:01)
**Symptom:** Terminal stayed in `Automatic` mode instead of `DirectVelocity`.
**Fix:** Added `sdo_write(0x8012, 0x01, u8::from(self.operation_mode))` as the first write.
**File:** `ethercat-hal/src/shared_config/el70x1.rs`

### 4. `EL70x1InfoData` enum missing EL7037 values
**Symptom:** `MotorLoad` (11) and `MotorDcCurrent` (13) not available as info data selections.
**EL7037 permitted info data:** 0, 7, 11, 13, 101, 103, 104, 150–153.
**EL7031 info data:** 0–7, 101, 103, 104, 150–153 (includes 1–6 which are invalid for EL7037).
**Fix:** Added `MotorLoad = 11` and `MotorDcCurrent = 13` variants with Debug, TryFrom, and From impls.
**File:** `ethercat-hal/src/shared_config/el70x1.rs`

### 5. `EL7037Configuration::default()` used EL7031 info data values
**Symptom:** Default `select_info_data_1 = MotorCurrentCoilA (3)` — invalid for EL7037.
**Fix:** Override defaults in `EL7037Configuration::default()` to `MotorLoad` (11) and `MotorDcCurrent` (13).
**File:** `ethercat-hal/src/devices/el7037/coe.rs`

### 6. `TestMotor` config used `StmFeatures::default()` with EL7031 values
**Symptom:** `select_info_data_1=MotorCurrentCoilA (3)` written to `0x8012:11` → `0x06040043`.
**Root cause:** `StmFeatures { ..Default::default() }` uses shared `StmFeatures::default()` (EL7031 defaults), not `EL7037Configuration::default().stm_features`.
**Fix:** Explicitly set `select_info_data_1: EL70x1InfoData::MotorLoad` and `select_info_data_2: EL70x1InfoData::MotorDcCurrent` in TestMotor config.
**File:** `machines/src/minimal_machines/motor_test_machine/new.rs`

### 7. `8010:03` nominal_voltage — unit conversion & supply-voltage validation ✅
**Symptom:** `0x06040043` when writing 5000 (50V in 10mV units) — terminal stores 2400 (24V).
**Root cause:** EL7037 uses **10 mV** units (shared config uses 1 mV). The write was rejected because 50V exceeds the actual 24V supply voltage. When configured with matching 24V (24000 in 1mV → 2400 in 10mV), the write succeeds.
**Fix:** Write `nominal_voltage / 10` with graceful fallback. If the terminal rejects (supply mismatch), `tracing::debug!` logs it and the terminal default is preserved.
**File:** `ethercat-hal/src/devices/el7037/coe.rs`

### 8. `8010:05` motor_emf re-enabled ✅
**Status:** Write succeeds. Shared default `0x00C8` (200) vs EL7037 doc default `0x0000` (0).
**File:** `ethercat-hal/src/devices/el7037/coe.rs`

### 9. Shared `StmControllerConfiguration::write_config` restored for EL7031/EL7041 ✅
**Root cause:** Subindices 03–08 were commented out in the shared function, breaking EL7031/EL7031_0030/EL7041_0052 which have those subindices on `0x8011` and `0x8013`.
**Fix:** Restored full `write_config` (all subindices 01–08). EL7037 now writes only `0x8011:01,02` directly (max subindex = 0x02 per doc), bypassing the shared function.
**File:** `ethercat-hal/src/shared_config/el70x1.rs`, `ethercat-hal/src/devices/el7037/coe.rs`

## Configuration Write Order (EL7037)

All steps verified working ✅

1. `encoder` → `8000:0E` ✅
2. `stm_motor` → `8010:01,02,03(graceful fallback),04,05,06,09,10,11` ✅
3. `stm_controller_1` → `8011:01,02` (direct, no shared write_config) ✅
4. `stm_features` → `8012:01,05,09,11,19,30,31,32,36` ✅
5. `pos_configuration` → `8020:01–10` ✅
6. `pos_features` → `8021:01,11,13,14,15,16` ✅
7. `txpdo_assignment` → `1C13` ✅
8. `rxpdo_assignment` → `1C12` ✅

### Remaining caveats

- **`0x8010:03` (nominal_voltage):** Written in 10mV units (`nominal_voltage / 10`).
  Rejected if the value exceeds the actual supply voltage; falls back to terminal default
  with a `tracing::debug!` log. Works correctly when config value matches supply (e.g. 24V).
- **`0x8010:05` (motor_emf):** Re-enabled. Shared default differs from EL7037 doc default.
- **`0x8011:03–08`:** EL7037 doc says max subindex = 0x02 — these genuinely don't exist.
  EL7037 writes only 01,02 directly; shared `StmControllerConfiguration::write_config`
  is restored for EL7031/EL7041 which do have these subindices.

## Key Differences: EL7031 vs EL7037

| Feature | EL7031 | EL7037 |
|---------|--------|--------|
| Channels | 2 (dual) | 1 (single) |
| Motor type | DC motor | Stepper motor |
| `0x8013` | Present | **Absent** |
| `0x8011:03` | inner_window exists | **Absent** (max subindex = 0x02) |
| `0x8010:03` unit | 1 mV | **10 mV** |
| Info data 1/2 permitted | 0–7, 101, 103, 104, 150–153 | 0, 7, **11, 13**, 101, 103, 104, 150–153 |
| `0x8012:01` valid values | 0, 1, 2, 3 | 0, 1, **3**, 4, 5, 6 (no value 2) |

## Debug Commands

```bash
# Build with debug prints
sudo CARGO_TARGET_DIR=/tmp/control_build cargo run --features development-build

# Check compilation
CARGO_TARGET_DIR=/tmp/control_build cargo check -p ethercat_hal -p machines
```
