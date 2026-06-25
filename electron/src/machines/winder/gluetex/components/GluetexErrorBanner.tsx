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

function decodeHeaterZones(zoneMask: number): number[] {
  const zones: number[] = [];
  for (let i = 0; i < 6; i++) {
    if (zoneMask & (1 << i)) zones.push(i + 1);
  }
  return zones;
}

export function GluetexErrorBanner() {
  const { lastSafetyStop, state, clearSafetyStop } = useGluetex();

  // Deduplicate by reason content, not timestamp. The backend emits SafetyStop
  // every control loop iteration while the condition is active, so timestamp-based
  // dedup causes the dialog to reopen hundreds of times per second.
  const currentReasonKey = lastSafetyStop
    ? JSON.stringify(lastSafetyStop.reason)
    : null;

  const [showDialog, setShowDialog] = useState(false);

  // Open whenever there is an active stop and the dialog is not already showing.
  useEffect(() => {
    if (currentReasonKey !== null && !showDialog) {
      setShowDialog(true);
    }
  }, [currentReasonKey, showDialog]);

  const handleAcknowledge = () => {
    clearSafetyStop();
    setShowDialog(false);
  };

  if (!lastSafetyStop) return null;

  const { reason, heaters_disabled } = lastSafetyStop;

  const winderTriggered = state?.winder_tension_arm_monitor_state?.triggered;
  const tapeTriggered = state?.tape_feeder_tension_arm_monitor_state?.triggered;
  const inletTriggered =
    state?.inlet_feeder_tension_arm_monitor_state?.triggered;
  const optris1Triggered = state?.optris_1_monitor_state?.triggered;
  const optris2Triggered = state?.optris_2_monitor_state?.triggered;

  const isTensionArm =
    reason === "WinderTensionArm" ||
    reason === "TapeFeederTensionArm" ||
    reason === "InletTensionArm";
  const isVoltage = reason === "Optris1Voltage" || reason === "Optris2Voltage";
  const isSleepTimer = reason === "SleepTimer";
  const isBandueberwachung = reason === "Bandueberwachung";
  const isHeaterOverTemp =
    typeof reason === "object" &&
    reason !== null &&
    "HeaterOverTemperature" in reason;

  const heaterZones = isHeaterOverTemp
    ? decodeHeaterZones(
        (reason as { HeaterOverTemperature: { zones: number } })
          .HeaterOverTemperature.zones,
      )
    : [];

  return (
    <Dialog open={showDialog} onOpenChange={() => {}}>
      <DialogContent
        className="max-w-2xl border-red-500"
        onPointerDownOutside={(e) => e.preventDefault()}
        onEscapeKeyDown={(e) => e.preventDefault()}
      >
        <DialogHeader>
          <div className="flex items-center gap-3">
            <span className="text-4xl">🛑</span>
            <div>
              <DialogTitle className="text-2xl font-bold text-red-600 dark:text-red-400">
                SAFETY STOP
              </DialogTitle>
              <div className="mt-1">
                {heaters_disabled ? (
                  <span className="inline-flex items-center rounded-full bg-red-100 px-3 py-1 text-sm font-semibold text-red-700 dark:bg-red-950 dark:text-red-300">
                    Motors + Heaters disabled
                  </span>
                ) : (
                  <span className="inline-flex items-center rounded-full bg-amber-100 px-3 py-1 text-sm font-semibold text-amber-700 dark:bg-amber-950 dark:text-amber-300">
                    Motors disabled
                  </span>
                )}
              </div>
            </div>
          </div>
        </DialogHeader>

        <DialogDescription className="space-y-4 text-base" asChild>
          <div>
            {isTensionArm && (
              <div className="space-y-3">
                <p className="text-red-600/90 dark:text-red-400/90">
                  One or more tension arms exceeded their configured safety
                  limits. The machine was automatically stopped to prevent
                  damage.
                </p>
                <div className="rounded-md bg-red-500/15 p-3">
                  <p className="mb-2 font-semibold text-red-600 dark:text-red-400">
                    Triggered tension arms:
                  </p>
                  <ul className="list-inside list-disc space-y-1 text-red-600/90 dark:text-red-400/90">
                    {winderTriggered && <li>Winder Tension Arm</li>}
                    {tapeTriggered && <li>TA Tape Feeder</li>}
                    {inletTriggered && <li>TA Inlet Feeder</li>}
                  </ul>
                </div>
                <p className="text-sm text-red-600/80 dark:text-red-400/80">
                  Check tension arm positions and adjust limits in Settings →
                  Winder / Addon / TA Inlet Feeder Monitor.
                </p>
              </div>
            )}

            {isVoltage && (
              <div className="space-y-3">
                <p className="text-red-600/90 dark:text-red-400/90">
                  One or more Optris voltage sensors detected readings outside
                  the configured safe range. The machine was automatically
                  stopped.
                </p>
                <div className="rounded-md bg-red-500/15 p-3">
                  <p className="mb-2 font-semibold text-red-600 dark:text-red-400">
                    Triggered monitors:
                  </p>
                  <ul className="list-inside list-disc space-y-1 text-red-600/90 dark:text-red-400/90">
                    {optris1Triggered && (
                      <li>
                        Optris 1:{" "}
                        {state?.quality_control_state?.optris1.current_voltage?.toFixed(
                          2,
                        ) ?? "N/A"}
                        V (Limits:{" "}
                        {state?.optris_1_monitor_state?.min_voltage?.toFixed(2)}
                        V –{" "}
                        {state?.optris_1_monitor_state?.max_voltage?.toFixed(2)}
                        V)
                      </li>
                    )}
                    {optris2Triggered && (
                      <li>
                        Optris 2:{" "}
                        {state?.quality_control_state?.optris2.current_voltage?.toFixed(
                          2,
                        ) ?? "N/A"}
                        V (Limits:{" "}
                        {state?.optris_2_monitor_state?.min_voltage?.toFixed(2)}
                        V –{" "}
                        {state?.optris_2_monitor_state?.max_voltage?.toFixed(2)}
                        V)
                      </li>
                    )}
                  </ul>
                </div>
                <p className="text-sm text-red-600/80 dark:text-red-400/80">
                  Check Optris sensors and verify readings. Configure limits in
                  Addons → Optris Monitor.
                </p>
              </div>
            )}

            {isHeaterOverTemp && (
              <div className="space-y-3">
                <p className="text-red-600/90 dark:text-red-400/90">
                  One or more heater zones exceeded their configured temperature
                  limit. The machine and heaters were automatically shut down.
                </p>
                <div className="rounded-md bg-red-500/15 p-3">
                  <p className="mb-2 font-semibold text-red-600 dark:text-red-400">
                    Over-temperature zones:
                  </p>
                  <ul className="list-inside list-disc space-y-1 text-red-600/90 dark:text-red-400/90">
                    {heaterZones.map((z) => (
                      <li key={z}>Zone {z}</li>
                    ))}
                  </ul>
                </div>
                <p className="text-sm text-red-600/80 dark:text-red-400/80">
                  Allow the machine to cool before resuming. Check temperature
                  limits in Heaters → Zone Configuration.
                </p>
              </div>
            )}

            {isSleepTimer && (
              <div className="space-y-3">
                <p className="text-amber-600/90 dark:text-amber-400/90">
                  The machine automatically entered standby mode due to
                  inactivity. This helps save energy and prevent unnecessary
                  wear.
                </p>
                <p className="text-sm text-amber-600/80 dark:text-amber-400/80">
                  Configure the inactivity timeout in Settings → Sleep Timer.
                </p>
              </div>
            )}

            {isBandueberwachung && (
              <div className="space-y-3">
                <p className="text-red-600/90 dark:text-red-400/90">
                  The band monitoring system detected that the band is absent or
                  broken. The machine was automatically stopped to prevent
                  damage.
                </p>
                <div className="rounded-md bg-red-500/15 p-3">
                  <p className="font-semibold text-red-600 dark:text-red-400">
                    Band Monitoring Alert
                  </p>
                  <p className="text-sm text-red-600/90 dark:text-red-400/90">
                    Check that the band is properly loaded and intact.
                  </p>
                </div>
              </div>
            )}

            {!isTensionArm &&
              !isVoltage &&
              !isHeaterOverTemp &&
              !isSleepTimer &&
              !isBandueberwachung && (
                <p className="text-red-600/90 dark:text-red-400/90">
                  An unexpected safety condition triggered an automatic stop.
                  Check the machine before resuming.
                </p>
              )}
          </div>
        </DialogDescription>

        <div className="mt-2">
          <TouchButton
            variant="destructive"
            className="w-full"
            onClick={handleAcknowledge}
          >
            Acknowledge
          </TouchButton>
        </div>
      </DialogContent>
    </Dialog>
  );
}
