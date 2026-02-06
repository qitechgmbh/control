import React from "react";
import { useGluetex } from "../hooks/useGluetex";
import { cn } from "@/lib/utils";

/**
 * Error banner component that displays system errors for the Gluetex winder
 * Shows errors from tension arm monitoring, sleep timer, and other safety systems
 */
export function GluetexErrorBanner() {
  const { state } = useGluetex();

  // Check for various error conditions
  const tensionArmTriggered = state?.tension_arm_monitor_state?.triggered;
  const sleepTimerTriggered = state?.sleep_timer_state?.triggered;

  // If no errors, don't render anything
  if (!tensionArmTriggered && !sleepTimerTriggered) {
    return null;
  }

  return (
    <div className="space-y-3">
      {tensionArmTriggered && (
        <div className="rounded-lg border-2 border-red-500/50 bg-red-500/10 p-4">
          <div className="flex items-start gap-3">
            <div className="flex-shrink-0 text-2xl">⚠️</div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-red-600 dark:text-red-400">
                Tension Arm Safety Limit Exceeded
              </h3>
              <p className="mt-1 text-sm text-red-600/90 dark:text-red-400/90">
                One or more tension arms have exceeded their configured safety
                limits. The machine has been automatically stopped to prevent
                damage. Check the tension arm positions and ensure they are
                within the configured limits before resuming operation.
              </p>
              <p className="mt-2 text-xs text-red-600/80 dark:text-red-400/80">
                Configure limits in Settings → Tension Arm Monitor
              </p>
            </div>
          </div>
        </div>
      )}

      {sleepTimerTriggered && (
        <div className="rounded-lg border-2 border-amber-500/50 bg-amber-500/10 p-4">
          <div className="flex items-start gap-3">
            <div className="flex-shrink-0 text-2xl">💤</div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-amber-600 dark:text-amber-400">
                Sleep Timer Activated
              </h3>
              <p className="mt-1 text-sm text-amber-600/90 dark:text-amber-400/90">
                The machine has automatically entered standby mode due to
                inactivity. This helps save energy and prevent unnecessary wear.
                Switch to a run mode or perform any action to reactivate the
                machine.
              </p>
              <p className="mt-2 text-xs text-amber-600/80 dark:text-amber-400/80">
                Configure timeout in Settings → Sleep Timer
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
