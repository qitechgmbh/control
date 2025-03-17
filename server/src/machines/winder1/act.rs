use super::WinderV1;
use ethercat_hal::actors::Actor;

impl Actor for WinderV1 {
    fn act(
        &mut self,
        now_ts: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.traverse_driver.act(now_ts).await;
            self.puller_driver.act(now_ts).await;
            self.winder_driver.act(now_ts).await;
            self.tension_arm_driver.act(now_ts).await;
            self.laser_driver.act(now_ts).await;
        })
    }
}
