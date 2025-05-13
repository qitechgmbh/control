use super::DreMachine;
use control_core::actors::Actor;
use std::{future::Future, pin::Pin, time::Instant};

impl Actor for DreMachine {
    fn act(&mut self, _now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // let dre_guard = self.dre.lock().await;
            // let diameter = dre_guard.diameter;
            // if diameter.is_ok() {
            //     println!("Measured Diameter is {:?}", diameter);
            // } else {
            //     println!("DRE Error: {:?}", diameter);
            // }
        })
    }
}
