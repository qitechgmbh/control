use super::Dre;
use control_core::actors::Actor;
use std::{
    future::Future, pin::Pin, sync::Arc, time::Instant
};

impl Actor for Dre {
    fn act(
        &mut self,
        _now_ts: Instant,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let diameter = Arc::clone(&self.diameter); 
        Box::pin(async move {
            let diam = diameter.read().await;
            if diam.is_ok(){
                println!("Measured Diameter is {:?}", diam);
            } else{
                println!("DRE Error: {:?}", diam);
            }
        })
    }
}
