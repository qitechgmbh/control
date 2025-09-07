use control_core::machines::new::MachineAct;
use smol::future;
use std::pin::Pin;
use std::time::Instant;

use crate::machines::extruder1::ExtruderV2;

impl MachineAct for ExtruderV2 {
    fn act(&mut self, now: Instant) -> Pin<&mut (dyn Future<Output = ()> + Send + '_)> {
        // run all your synchronous logic here...
        // (left out for brevity)

        // refresh the slot so it's a "completed" Ready future again
        self.future_slot = future::ready(());

        // return pinned &mut reference as a trait object
        Pin::new(&mut self.future_slot) as Pin<&mut (dyn Future<Output = ()> + Send + '_)>
    }
}
