//! Maps domain field names on [`crate::gluetex::Gluetex`] to wire/API identifiers.
//!
//! JSON `StateEvent` / `LiveValuesEvent` field names are unchanged for frontend compatibility.

use crate::gluetex::api::TensionArmState;
use crate::gluetex::controllers::tension::TensionArm;

/// Winder tension arm (`winder_tension_arm`) → `tension_arm_state` on the wire.
pub fn tension_arm_state_from_winder(arm: &TensionArm) -> TensionArmState {
    TensionArmState { zeroed: arm.zeroed }
}
