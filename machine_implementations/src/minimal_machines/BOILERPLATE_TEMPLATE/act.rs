// ============================================================================
// act.rs — Machine trait impl (the control-loop entry point)
// ============================================================================
// `act()` is invoked on every control cycle (typically 1 kHz). Keep it cheap:
// drain the inbound message queue, then emit state at a UI-friendly rate
// (~30 Hz here).
//
// `react()` is called once per cycle AFTER all machines have acted. Use it
// for cross-machine reactions via `MachineDataRegistry`. Most minimal machines
// leave it empty.
// ============================================================================

use std::time::{Duration, Instant};

use qitech_lib::machines::{Machine, MachineDataRegistry, MachineError};

use super::MyMachine;
use crate::MachineApi;

impl Machine for MyMachine {
    fn act(
        &mut self,
        _machine_data: Option<&mut MachineDataRegistry>,
    ) -> Result<(), MachineError> {
        let now = Instant::now();

        // Drain one inbound message per cycle (API + subscription traffic).
        if let Ok(msg) = self.receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Emit state at ~30 Hz — tune the divisor if you need a different rate.
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            // TODO: if your machine needs to refresh values FROM hardware
            // before emitting (e.g. read analog/digital inputs into struct
            // fields), do that here before the emit.
            self.emit_state();
            self.last_state_emit = now;
        }

        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {
        // TODO: cross-machine reactions go here. Empty for most minimal machines.
    }

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
}
