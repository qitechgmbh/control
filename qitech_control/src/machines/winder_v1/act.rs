use std::time::Instant;

use qitech_framework::MachineIdentificationUnique;
use qitech_framework::machine::Machine;
use qitech_framework::machine::SubscribeContext;
use qitech_framework::machine::error::ActResult;
use qitech_framework::machine::error::SubscribeResult;

use super::Winder_V1;
use crate::machines::winder_v1::LaserSubscription;

impl Machine for Winder_V1 {
    fn act(&mut self) -> ActResult {
        let now = Instant::now();

        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        // update the resources
        self.update_states();
        self.update_measurements();

        Ok(())
    }

    fn subscribe(&mut self, mut ctx: SubscribeContext) -> SubscribeResult<()> {
        self.laser_subscription = Some(LaserSubscription {
            ident: ctx.producer(),
            current: ctx.subscribe_measurement("diameter")?,
            target: ctx.subscribe_config("diameter.target")?,
            upper: ctx.subscribe_config("diameter.tolerance.upper")?,
            lower: ctx.subscribe_config("diameter.tolerance.lower")?,
        });

        Ok(())
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        if let Some(sub) = &mut self.laser_subscription
            && sub.ident == ident
        {
            self.laser_subscription = None;
        }
    }
}
