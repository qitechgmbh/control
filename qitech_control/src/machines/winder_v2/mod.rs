use std::time::Duration;
use std::time::Instant;

use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::machine::OperationCapability;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine::SubscribeContext;
use qitech_framework::machine::SubscribeResult;
use qitech_framework::vendors;
use qitech_lib::units::Length;

use crate::machines::winder_v2::types::AutomaticActionSpoolAction;
use crate::machines::winder_v2::types::LaserSubscription;
use crate::machines::winder_v2::types::Mode;

mod build;
mod types;
mod utils;

mod traverse;
use traverse::Traverse;

mod puller;
use puller::Puller;

mod spool;
use spool::Spool;

mod tension_arm;
use tension_arm::TensionArm;

mod laser_pointer;
use laser_pointer::LaserPointer;

pub const VARIANT_REGULAR: usize = 0;
pub const VARIANT_7031_SPOOL: usize = 1;

#[allow(non_camel_case_types)]
pub type WinderV1_Regular = WinderV1<VARIANT_REGULAR>;

#[allow(non_camel_case_types)]
pub type WinderV1_7031_Spool = WinderV1<VARIANT_7031_SPOOL>;

pub struct WinderV1<const VARIANT: usize> {
    // ---- components ---
    spool: Spool,
    puller: Puller,
    traverse: Traverse,
    tension_arm: TensionArm,
    laser_pointer: LaserPointer,

    // --- state ---
    pub(super) mode: StateProperty<Mode>,
    // pub(super) spool_automatic_action: SpoolAutomaticAction,

    // --- subscriptions ---
    pub(super) laser_subscription: Option<LaserSubscription>,
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
    fn act(&mut self, dt: Duration) -> ActResult {
        if let Err(kind) = self.tension_arm.update() {
            return Err(ActError {
                kind,
                impact: ActErrorImpact::Degraded,
            });
        };

        self.spool.update(dt, &self.puller, &self.tension_arm);

        if let Err(kind) = self.puller.update(dt, self.laser_subscription.as_ref()) {
            return Err(ActError {
                kind,
                impact: ActErrorImpact::Degraded,
            });
        };

        self.traverse.update(dt, self.spool.velocity.get());

        Ok(())
    }

    fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        self.laser_subscription = Some(LaserSubscription {
            ident: ctx.provider(),
            diameter: ctx.measurement("diameter")?,
            diameter_target: ctx.config("diameter.target")?,
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
    fn can_wind(&self) -> OperationCapability {
        if !self.tension_arm.zeroed() {
            return OperationCapability::forbidden("tension arm is not zeroed");
        }

        if !self.traverse.is_homed() {
            return OperationCapability::forbidden("traverse is not homed");
        }

        if self.traverse.is_homing() {
            return OperationCapability::forbidden("traverse is homing");
        }

        OperationCapability::Allowed
    }

    fn set_mode(&mut self, mode: Mode) -> ActResult {
        self.mode.set(mode);
        self.spool.set_mode(mode.into());
        self.puller.set_mode(mode.into());

        if let Err(kind) = self.traverse.set_mode(mode.into()) {
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
