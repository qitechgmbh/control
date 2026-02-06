import React, { useEffect, useState } from "react";
import { useGluetex } from "../hooks/useGluetex";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { TouchButton } from "@/components/touch/TouchButton";
import { useRouter } from "@tanstack/react-router";
import { gluetexRoute } from "@/routes/routes";

/**
 * Error dialog component that displays critical system errors for the Gluetex winder
 * Shows modal dialogs for tension arm monitoring, sleep timer, and other safety systems
 * User must explicitly dismiss each error before continuing
 */
export function GluetexErrorBanner() {
  const { state, setOperationMode } = useGluetex();
  const router = useRouter();
  const { serial } = gluetexRoute.useParams();

  // Check for various error conditions - check each tension arm separately
  const winderTensionArmTriggered =
    state?.winder_tension_arm_monitor_state?.triggered;
  const addonTensionArmTriggered =
    state?.addon_tension_arm_monitor_state?.triggered;
  const slaveTensionArmTriggered =
    state?.slave_tension_arm_monitor_state?.triggered;
  const anyTensionArmTriggered =
    winderTensionArmTriggered ||
    addonTensionArmTriggered ||
    slaveTensionArmTriggered;
  const sleepTimerTriggered = state?.sleep_timer_state?.triggered;

  // Track if we should show the dialog (set to true when error triggers)
  const [showTensionArmDialog, setShowTensionArmDialog] = useState(false);
  const [showSleepTimerDialog, setShowSleepTimerDialog] = useState(false);

  // Track previous trigger state to detect new errors
  const [prevAnyTensionArmTriggered, setPrevAnyTensionArmTriggered] =
    useState(false);
  const [prevSleepTimerTriggered, setPrevSleepTimerTriggered] = useState(false);

  // Show dialog when error is triggered (transition from false to true)
  useEffect(() => {
    if (anyTensionArmTriggered && !prevAnyTensionArmTriggered) {
      setShowTensionArmDialog(true);
    }
    setPrevAnyTensionArmTriggered(anyTensionArmTriggered ?? false);
  }, [anyTensionArmTriggered, prevAnyTensionArmTriggered]);

  useEffect(() => {
    if (sleepTimerTriggered && !prevSleepTimerTriggered) {
      setShowSleepTimerDialog(true);
    }
    setPrevSleepTimerTriggered(sleepTimerTriggered ?? false);
  }, [sleepTimerTriggered, prevSleepTimerTriggered]);

  // Handler to dismiss tension arm dialog and switch to setup mode
  const dismissTensionArmDialog = () => {
    setOperationMode("Setup");
    setShowTensionArmDialog(false);
    router.navigate({
      to: `/_sidebar/machines/gluetex/${serial}/overview`,
    });
  };

  // Handler to dismiss sleep timer dialog and switch to setup mode
  const dismissSleepTimerDialog = () => {
    setOperationMode("Setup");
    setShowSleepTimerDialog(false);
    router.navigate({
      to: `/_sidebar/machines/gluetex/${serial}/overview`,
    });
  };

  // Determine which dialog to show (priority: tension arm > sleep timer)
  const displayTensionArmDialog = showTensionArmDialog;
  const displaySleepTimerDialog =
    !displayTensionArmDialog && showSleepTimerDialog;

  return (
    <>
      {/* Tension Arm Error Dialog */}
      <Dialog open={displayTensionArmDialog} onOpenChange={() => {}}>
        <DialogContent
          className="max-w-lg border-red-500"
          onPointerDownOutside={(e) => e.preventDefault()}
          onEscapeKeyDown={(e) => e.preventDefault()}
        >
          <DialogHeader>
            <div className="flex items-center gap-3">
              <span className="text-3xl">⚠️</span>
              <DialogTitle className="text-xl text-red-600 dark:text-red-400">
                Tension Arm Safety Limit Exceeded
              </DialogTitle>
            </div>
          </DialogHeader>

          <DialogDescription className="space-y-3 text-base">
            <p className="text-red-600/90 dark:text-red-400/90">
              One or more tension arms have exceeded their configured safety
              limits. The machine has been automatically stopped to prevent
              damage.
            </p>

            {/* Show which specific tension arm(s) triggered */}
            <div className="rounded-md bg-red-500/20 p-3">
              <p className="mb-2 font-semibold text-red-600 dark:text-red-400">
                Triggered Tension Arms:
              </p>
              <ul className="list-inside list-disc space-y-1 text-red-600/90 dark:text-red-400/90">
                {winderTensionArmTriggered && <li>Winder Tension Arm</li>}
                {addonTensionArmTriggered && <li>Addon Tension Arm</li>}
                {slaveTensionArmTriggered && <li>Slave Tension Arm</li>}
              </ul>
            </div>

            <p className="text-red-600/90 dark:text-red-400/90">
              Check the tension arm positions and ensure they are within the
              configured limits before resuming operation.
            </p>
            <p className="text-sm text-red-600/80 dark:text-red-400/80">
              Configure limits in Settings → Winder/Addon/Slave Tension Arm
              Monitor
            </p>
          </DialogDescription>

          <div className="mt-4">
            <TouchButton
              variant="destructive"
              className="w-full"
              onClick={dismissTensionArmDialog}
            >
              Acknowledge & Return to Setup Mode
            </TouchButton>
          </div>
        </DialogContent>
      </Dialog>

      {/* Sleep Timer Dialog */}
      <Dialog open={displaySleepTimerDialog} onOpenChange={() => {}}>
        <DialogContent
          className="max-w-lg border-amber-500"
          onPointerDownOutside={(e) => e.preventDefault()}
          onEscapeKeyDown={(e) => e.preventDefault()}
        >
          <DialogHeader>
            <div className="flex items-center gap-3">
              <span className="text-3xl">💤</span>
              <DialogTitle className="text-xl text-amber-600 dark:text-amber-400">
                Sleep Timer Activated
              </DialogTitle>
            </div>
          </DialogHeader>

          <DialogDescription className="space-y-3 text-base">
            <p className="text-amber-600/90 dark:text-amber-400/90">
              The machine has automatically entered standby mode due to
              inactivity. This helps save energy and prevent unnecessary wear.
            </p>
            <p className="text-amber-600/90 dark:text-amber-400/90">
              Switch to a run mode or perform any action to reactivate the
              machine.
            </p>
            <p className="text-sm text-amber-600/80 dark:text-amber-400/80">
              Configure timeout in Settings → Sleep Timer
            </p>
          </DialogDescription>

          <div className="mt-4">
            <TouchButton className="w-full" onClick={dismissSleepTimerDialog}>
              Acknowledge & Return to Setup Mode
            </TouchButton>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
