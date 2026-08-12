use std::time::Instant;

use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine::SubscribeContext;
use qitech_framework::machine::SubscribeResult;
use qitech_framework::vendors;
use qitech_lib::units::Length;

use crate::machines::winder_v2::types::AutomaticActionSpoolAction;
use crate::machines::winder_v2::types::LaserSubscription;
use crate::machines::winder_v2::types::Mode;

mod build;

mod traverse;
use traverse::Traverse;

mod tension_arm;
use tension_arm::TensionArm;

mod puller;
use puller::Puller;

mod spool;
use spool::Spool;

mod types;
mod utils;

pub const LASER_PORT: usize = 0;
pub const SPOOL_PORT: usize = 0;

pub const VARIANT_REGULAR: usize = 0;
pub const VARIANT_7031_SPOOL: usize = 1;

#[allow(non_camel_case_types)]
pub type WinderV1_Regular = WinderV1<VARIANT_REGULAR>;

#[allow(non_camel_case_types)]
pub type WinderV1_7031_Spool = WinderV1<VARIANT_7031_SPOOL>;

pub struct WinderV1<const VARIANT: usize> {
    pub spool: Spool,
    pub puller: Puller,
    pub traverse: Traverse,
    pub tension_arm: TensionArm,

    pub mode: StateProperty<Mode>,
    pub spool_automatic_action: SpoolAutomaticAction,

    pub laser_subscription: Option<LaserSubscription>,
}

impl MachineDescriptor for WinderV1<VARIANT_REGULAR> {
    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 2,
    };

    const SCHEMA: &'static str = include_str!("../../../schemas/winder_v1.yaml");
}

impl MachineDescriptor for WinderV1<VARIANT_7031_SPOOL> {
    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 98,
    };

    const SCHEMA: &'static str = include_str!("../../../schemas/winder_v1_7031_0030_spool.yaml");
}

impl<const VARIANT: usize> Machine for WinderV1<VARIANT> {
    fn act(&mut self, now: Instant) -> ActResult {
        if let Err(kind) = self.tension_arm.update() {
            return Err(ActError {
                kind,
                impact: ActErrorImpact::Degraded,
            });
        };

        // self.sync_spool_speed(now);

        self.puller.update(now, &self.laser_subscription);
        self.traverse.update(now, self.spool.velocity.get());

        // self.stop_or_pull_spool(now);

        Ok(())
    }

    fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        self.laser_subscription = Some(LaserSubscription {
            ident: ctx.provider(),
            diameter: ctx.measurement("diameter")?,
            diameter_target: ctx.config("diameter.target")?,
            tolerance_upper: ctx.config("diameter.tolerance.upper")?,
            tolerance_lower: ctx.config("diameter.tolerance.lower")?,
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
    pub fn set_mode(&mut self, mode: Mode) -> ActResult {
        self.mode.set(mode);
        self.spool.apply_mode(mode.into());
        self.puller.apply_mode(mode.into());

        if let Err(kind) = self.traverse.apply_mode(mode.into()) {
            return Err(ActError {
                kind,
                impact: ActErrorImpact::Degraded,
            });
        }

        Ok(())
    }
}

pub struct SpoolAutomaticAction {
    pub progress: Length,
    progress_last_check: Instant,
    pub target_length: ConfigProperty<Length>,
    pub mode: ConfigProperty<AutomaticActionSpoolAction>,
}
