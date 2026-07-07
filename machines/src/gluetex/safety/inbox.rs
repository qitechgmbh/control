use std::time::Instant;

use super::stop::{SafetyStop, StopReason};

/// Severity carried by a pending safety message — determines the aggregate
/// mode/motor/heater side effects while the message is pending (see
/// `Gluetex::reconcile_safety_stops`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetySeverity {
    /// Motors disabled, mode -> Hold, operation_mode -> Setup; heaters unchanged.
    MotorsOnly,
    /// Same as MotorsOnly, plus heaters disabled.
    Full,
}

impl From<SafetyStop> for SafetySeverity {
    fn from(stop: SafetyStop) -> Self {
        if stop.disables_heaters() {
            Self::Full
        } else {
            Self::MotorsOnly
        }
    }
}

/// A single pending (unacknowledged) safety message.
#[derive(Debug, Clone)]
pub struct SafetyMessage {
    /// Unique for the process lifetime; not persisted across restarts.
    pub id: u64,
    pub reason: StopReason,
    pub severity: SafetySeverity,
    /// When this occurrence was first raised (used to compute wire `age_ms`).
    pub first_occurred_at: Instant,
    /// How many times the underlying condition has re-triggered while this
    /// message was already pending, folded in rather than duplicated.
    pub occurrence_count: u32,
}

/// Result of pushing a newly-detected safety condition into the inbox.
pub enum PushOutcome {
    /// A brand-new distinct message was created.
    New(u64),
    /// A message for this reason kind was already pending; updated in place.
    Updated(u64),
}

/// Backend-owned collection of currently pending (unacknowledged) safety
/// messages for one Gluetex machine instance. In-memory only, exactly like
/// the rest of Gluetex's runtime state — reset on process restart.
#[derive(Debug, Default)]
pub struct SafetyInbox {
    messages: Vec<SafetyMessage>,
    next_id: u64,
}

impl SafetyInbox {
    /// Dedup key: the same `StopReason` *variant* (ignoring payload, e.g. the
    /// `HeaterOverTemperature` zone bitmask) counts as the same pending
    /// issue, per "only the first is kept until acknowledged."
    fn key(reason: &StopReason) -> std::mem::Discriminant<StopReason> {
        std::mem::discriminant(reason)
    }

    /// Record a rising-edge occurrence of `stop`. If a message of the same
    /// reason kind is already pending, it is updated in place instead of
    /// duplicated.
    pub fn push(&mut self, stop: SafetyStop, now: Instant) -> PushOutcome {
        let reason = stop.reason();
        let severity = SafetySeverity::from(stop);
        let key = Self::key(&reason);

        if let Some(existing) = self
            .messages
            .iter_mut()
            .find(|m| Self::key(&m.reason) == key)
        {
            existing.occurrence_count += 1;
            if let (
                StopReason::HeaterOverTemperature {
                    zones: existing_zones,
                },
                StopReason::HeaterOverTemperature { zones: new_zones },
            ) = (&mut existing.reason, reason)
            {
                *existing_zones |= new_zones;
            }
            return PushOutcome::Updated(existing.id);
        }

        let id = self.next_id;
        self.next_id += 1;
        self.messages.push(SafetyMessage {
            id,
            reason,
            severity,
            first_occurred_at: now,
            occurrence_count: 1,
        });
        PushOutcome::New(id)
    }

    /// Union a freshly observed zone mask into an already-pending
    /// `HeaterOverTemperature` message without treating it as a new
    /// occurrence (used while the monitor stays latched and additional zones
    /// go over temperature). Returns whether the mask actually gained any
    /// new zone bits, so the caller knows whether to emit state.
    pub fn touch_heater_overtemp_zone_mask(&mut self, zones: u8) -> bool {
        let mut changed = false;
        for m in &mut self.messages {
            if let StopReason::HeaterOverTemperature { zones: existing } = &mut m.reason {
                let updated = *existing | zones;
                if updated != *existing {
                    changed = true;
                    *existing = updated;
                }
            }
        }
        changed
    }

    /// Remove and return the message with the given id, if pending.
    pub fn acknowledge(&mut self, id: u64) -> Option<SafetyMessage> {
        let idx = self.messages.iter().position(|m| m.id == id)?;
        Some(self.messages.remove(idx))
    }

    /// Remove and return every pending message.
    pub fn acknowledge_all(&mut self) -> Vec<SafetyMessage> {
        std::mem::take(&mut self.messages)
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn any_full_severity(&self) -> bool {
        self.messages
            .iter()
            .any(|m| m.severity == SafetySeverity::Full)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, SafetyMessage> {
        self.messages.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn winder_ta_motors_only() -> SafetyStop {
        SafetyStop::MotorsOnly {
            reason: StopReason::WinderTensionArm,
        }
    }

    #[test]
    fn push_creates_new_message_for_distinct_reason() {
        let mut inbox = SafetyInbox::default();
        let now = Instant::now();

        match inbox.push(winder_ta_motors_only(), now) {
            PushOutcome::New(_) => {}
            PushOutcome::Updated(_) => panic!("expected New"),
        }
        assert_eq!(inbox.len(), 1);
    }

    #[test]
    fn repeated_push_of_same_reason_updates_instead_of_duplicating() {
        let mut inbox = SafetyInbox::default();
        let now = Instant::now();

        let PushOutcome::New(id) = inbox.push(winder_ta_motors_only(), now) else {
            panic!("expected New");
        };
        let PushOutcome::Updated(updated_id) = inbox.push(winder_ta_motors_only(), now) else {
            panic!("expected Updated");
        };

        assert_eq!(id, updated_id);
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox.iter().next().unwrap().occurrence_count, 2);
    }

    #[test]
    fn distinct_reasons_produce_distinct_messages() {
        let mut inbox = SafetyInbox::default();
        let now = Instant::now();

        inbox.push(winder_ta_motors_only(), now);
        inbox.push(
            SafetyStop::MotorsOnly {
                reason: StopReason::TapeFeederTensionArm,
            },
            now,
        );

        assert_eq!(inbox.len(), 2);
    }

    #[test]
    fn heater_overtemp_zone_mask_unions_on_repeat_push() {
        let mut inbox = SafetyInbox::default();
        let now = Instant::now();

        inbox.push(
            SafetyStop::Full {
                reason: StopReason::HeaterOverTemperature { zones: 0b0001 },
            },
            now,
        );
        inbox.push(
            SafetyStop::Full {
                reason: StopReason::HeaterOverTemperature { zones: 0b0010 },
            },
            now,
        );

        assert_eq!(inbox.len(), 1);
        match inbox.iter().next().unwrap().reason {
            StopReason::HeaterOverTemperature { zones } => assert_eq!(zones, 0b0011),
            _ => panic!("expected HeaterOverTemperature"),
        }
    }

    #[test]
    fn acknowledge_removes_message_and_reports_full_severity() {
        let mut inbox = SafetyInbox::default();
        let now = Instant::now();

        let PushOutcome::New(motors_only_id) = inbox.push(winder_ta_motors_only(), now) else {
            panic!("expected New");
        };
        inbox.push(
            SafetyStop::Full {
                reason: StopReason::SleepTimer,
            },
            now,
        );

        assert!(inbox.any_full_severity());
        let acknowledged = inbox.acknowledge(motors_only_id).unwrap();
        assert_eq!(acknowledged.reason, StopReason::WinderTensionArm);
        assert_eq!(inbox.len(), 1);
        assert!(inbox.any_full_severity());
    }

    #[test]
    fn acknowledge_all_clears_everything() {
        let mut inbox = SafetyInbox::default();
        let now = Instant::now();

        inbox.push(winder_ta_motors_only(), now);
        inbox.push(
            SafetyStop::Full {
                reason: StopReason::SleepTimer,
            },
            now,
        );

        let acknowledged = inbox.acknowledge_all();
        assert_eq!(acknowledged.len(), 2);
        assert!(inbox.is_empty());
        assert!(!inbox.any_full_severity());
    }
}
