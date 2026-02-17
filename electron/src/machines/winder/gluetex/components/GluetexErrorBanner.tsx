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

  // Check voltage monitors
  const optris1MonitorTriggered = state?.optris_1_monitor_state?.triggered;
  const optris2MonitorTriggered = state?.optris_2_monitor_state?.triggered;
  const anyVoltageMonitorTriggered =
    optris1MonitorTriggered || optris2MonitorTriggered;

  const sleepTimerTriggered = state?.sleep_timer_state?.triggered;

  // Track if we should show the dialog (set to true when error triggers)
  const [showTensionArmDialog, setShowTensionArmDialog] = useState(false);
  const [showVoltageMonitorDialog, setShowVoltageMonitorDialog] =
    useState(false);
  const [showSleepTimerDialog, setShowSleepTimerDialog] = useState(false);

  // Track previous trigger state to detect new errors
  const [prevAnyTensionArmTriggered, setPrevAnyTensionArmTriggered] =
    useState(false);
  const [prevAnyVoltageMonitorTriggered, setPrevAnyVoltageMonitorTriggered] =
    useState(false);
  const [prevSleepTimerTriggered, setPrevSleepTimerTriggered] = useState(false);

  // Show dialog when error is triggered (transition from false to true)
  // Also immediately switch to Setup mode
  useEffect(() => {
    if (anyTensionArmTriggered && !prevAnyTensionArmTriggered) {
      setShowTensionArmDialog(true);
      // Immediately switch to Setup mode when error is detected
      setOperationMode("Setup");
    }
    setPrevAnyTensionArmTriggered(anyTensionArmTriggered ?? false);
  }, [anyTensionArmTriggered, prevAnyTensionArmTriggered, setOperationMode]);

  useEffect(() => {
    if (anyVoltageMonitorTriggered && !prevAnyVoltageMonitorTriggered) {
      setShowVoltageMonitorDialog(true);
      // Immediately switch to Setup mode when error is detected
      setOperationMode("Setup");
    }
    setPrevAnyVoltageMonitorTriggered(anyVoltageMonitorTriggered ?? false);
  }, [anyVoltageMonitorTriggered, prevAnyVoltageMonitorTriggered, setOperationMode]);

  useEffect(() => {
    if (sleepTimerTriggered && !prevSleepTimerTriggered) {
      setShowSleepTimerDialog(true);
      // Immediately switch to Setup mode when timer expires
      setOperationMode("Setup");
    }
    setPrevSleepTimerTriggered(sleepTimerTriggered ?? false);
  }, [sleepTimerTriggered, prevSleepTimerTriggered, setOperationMode]);

  // Handler to dismiss tension arm dialog (mode already switched to Setup)
  const dismissTensionArmDialog = () => {
    setShowTensionArmDialog(false);
    router.navigate({
      to: `/_sidebar/machines/gluetex/${serial}/overview`,
    });
  };

  // Handler to dismiss voltage monitor dialog (mode already switched to Setup)
  const dismissVoltageMonitorDialog = () => {
    setShowVoltageMonitorDialog(false);
    router.navigate({
      to: `/_sidebar/machines/gluetex/${serial}/addons`,
    });
  };

  // Handler to dismiss sleep timer dialog (mode already switched to Setup)
  const dismissSleepTimerDialog = () => {
    setShowSleepTimerDialog(false);
    router.navigate({
      to: `/_sidebar/machines/gluetex/${serial}/overview`,
    });
  };

  // Determine which dialog to show (priority: tension arm > voltage > sleep timer)
  const displayTensionArmDialog = showTensionArmDialog;
  const displayVoltageMonitorDialog =
    !displayTensionArmDialog && showVoltageMonitorDialog;
  const displaySleepTimerDialog =
    !displayTensionArmDialog &&
    !displayVoltageMonitorDialog &&
    showSleepTimerDialog;

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

      {/* Voltage Monitor Error Dialog */}
      <Dialog open={displayVoltageMonitorDialog} onOpenChange={() => {}}>
        <DialogContent
          className="max-w-lg border-red-500"
          onPointerDownOutside={(e) => e.preventDefault()}
          onEscapeKeyDown={(e) => e.preventDefault()}
        >
          <DialogHeader>
            <div className="flex items-center gap-3">
              <span className="text-3xl">⚠️</span>
              <DialogTitle className="text-xl text-red-600 dark:text-red-400">
                Voltage Monitor Safety Limit Exceeded
              </DialogTitle>
            </div>
          </DialogHeader>
          <DialogDescription className="space-y-4">
            <div className="text-base">
              <p className="mb-3">
                One or more Optris voltage sensors have detected readings
                outside the configured safe operating range. The machine has
                been automatically stopped to prevent potential damage.
              </p>

              <div className="rounded-md bg-red-50 p-4 dark:bg-red-950">
                <p className="font-semibold text-red-700 dark:text-red-400">
                  Triggered monitors:
                </p>
                <ul className="mt-2 list-inside list-disc space-y-1 text-red-700 dark:text-red-400">
                  {optris1MonitorTriggered && (
                    <li>
                      Optris 1:{" "}
                      {state?.quality_control_state?.optris1.current_voltage?.toFixed(
                        2,
                      ) ?? "N/A"}
                      V (Limits:{" "}
                      {state?.optris_1_monitor_state?.min_voltage?.toFixed(2)}V
                      - {state?.optris_1_monitor_state?.max_voltage?.toFixed(2)}
                      V)
                    </li>
                  )}
                  {optris2MonitorTriggered && (
                    <li>
                      Optris 2:{" "}
                      {state?.quality_control_state?.optris2.current_voltage?.toFixed(
                        2,
                      ) ?? "N/A"}
                      V (Limits:{" "}
                      {state?.optris_2_monitor_state?.min_voltage?.toFixed(2)}V
                      - {state?.optris_2_monitor_state?.max_voltage?.toFixed(2)}
                      V)
                    </li>
                  )}
                </ul>
              </div>

              <p className="mt-3">
                Please check the Optris sensors and verify the readings are
                correct before resuming operation.
              </p>
            </div>

            <div className="flex flex-col gap-3">
              <TouchButton
                onClick={dismissVoltageMonitorDialog}
                variant="default"
                className="w-full"
              >
                Switch to Setup Mode & Check Sensors
              </TouchButton>
            </div>
          </DialogDescription>
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
