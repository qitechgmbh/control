use super::ExtruderV2;
use control_core::actors::Actor;
use std::time::Instant;

impl Actor for ExtruderV2 {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.inverter.act(now_ts).await;
        })
    }
}
