use qitech_framework::{
    MachineIdentificationUnique, machine::{ActError, ActErrorImpact, ActResult, Machine, SubscribeContext, SubscribeResult},
};
use qitech_lib::units::length::millimeter;
use std::time::Instant;

use super::WinderV1;
use crate::machines::winder_v2::{
    SPOOL_PORT,
    types::{LaserSubscription, Mode, SpoolAutomaticActionMode},
};

impl<const VARIANT: usize> Machine for WinderV1<VARIANT> {
    fn act(&mut self, now: Instant) -> ActResult {
        if let Err(kind) = self.tension_arm.update() {
            return Err(ActError {
                kind,
                impact: ActErrorImpact::Degraded,
            });
        };

        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        self.traverse.update(now, self.spool_speed_controller.get_speed());

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        // ---
        if let Some(laser) = &self.laser_subscription {
            self.puller_speed_controller
                .adaptive
                .update_with_measurement(
                    laser.current.get_as::<millimeter>(),
                    laser.target.get_as::<millimeter>(),
                    laser.lower.get_as::<millimeter>(),
                    laser.upper.get_as::<millimeter>(),
                    self.puller_speed_controller.last_speed,
                    Instant::now(),
                );
        }

        // update the resources
        self.update_measurements();

        Ok(())
    }

    fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        self.laser_subscription = Some(LaserSubscription {
            ident: ctx.provider(),
            current: ctx.measurement("diameter")?,
            target: ctx.config("diameter.target")?,
            upper: ctx.config("diameter.tolerance.upper")?,
            lower: ctx.config("diameter.tolerance.lower")?,
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

impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn sync_spool_speed(&mut self, t: Instant) {
        let angular_velocity = self.spool_speed_controller.update_speed(
            t,
            &self.tension_arm,
            &self.puller_speed_controller,
        );

        // Apply direction based on forward setting
        let directed_angular_velocity = if self.spool_speed_controller.get_forward() {
            angular_velocity
        } else {
            -angular_velocity
        };

        let steps_per_second = self
            .spool_step_converter
            .angular_velocity_to_steps(directed_angular_velocity);
        let spool_ref = &mut *self.spool.borrow_mut();
        let _ = spool_ref.set_speed(SPOOL_PORT, steps_per_second);
    }

    pub fn stop_or_pull_spool(&mut self, now: Instant) {
        if matches!(
            self.spool_automatic_action.mode.get(),
            SpoolAutomaticActionMode::NoAction
        ) {
            self.calculate_spool_auto_progress_(now);
            return;
        }

        match self.mode.get() {
            Mode::Pull => self.calculate_spool_auto_progress_(now),
            Mode::Wind => self.calculate_spool_auto_progress_(now),
            _ => {
                self.spool_automatic_action.progress_last_check = now;
                return;
            }
        }

        if self.spool_automatic_action.progress >= self.spool_automatic_action.target_length.get() {
            match self.spool_automatic_action.mode.get() {
                SpoolAutomaticActionMode::NoAction => (),
                SpoolAutomaticActionMode::Pull => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(Mode::Pull);
                }
                SpoolAutomaticActionMode::Hold => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(Mode::Hold);
                }
            }
        }
    }
}
