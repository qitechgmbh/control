import React, { useEffect, useState } from "react";
import { toast } from "sonner";
import { ToggleButton } from "@/components/touch/TouchToggleButton";
import { TouchButton } from "@/components/touch/TouchButton";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  getDemoMachinesEnabled,
  setDemoMachinesEnabled,
} from "@/helpers/troubleshoot_helpers";
import { useBackendConnected } from "@/client/socketioStore";

const compact = "px-4 py-2 text-sm";

/** Turns the demo production line (`DEMO_MACHINES=true`) on or off; restarts the backend. */
export function DemoMachinesToggle() {
  const backendConnected = useBackendConnected();
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [pending, setPending] = useState<boolean | null>(null);
  const [isApplying, setIsApplying] = useState(false);

  // Re-read whenever the backend connection changes
  useEffect(() => {
    let cancelled = false;
    getDemoMachinesEnabled().then((result) => {
      if (cancelled) return;
      if (result.success && result.enabled !== undefined) {
        setEnabled(result.enabled);
        setLoadError(null);
      } else {
        console.error("Could not read the demo setting", result.error);
        setLoadError(result.error ?? "unknown error");
      }
    });
    return () => {
      cancelled = true;
    };
  }, [backendConnected]);

  const apply = async (next: boolean) => {
    setIsApplying(true);
    const result = await setDemoMachinesEnabled(next);
    if (result.success) {
      setEnabled(next);
      toast.success(
        next
          ? "Demo machines enabled, restarting backend"
          : "Demo machines disabled, restarting backend",
      );
    } else {
      toast.error(`Failed to change the demo setting: ${result.error}`);
    }
    setIsApplying(false);
    setPending(null);
  };

  if (enabled === null) {
    return (
      <div
        title={
          loadError
            ? `Could not read the demo setting (${loadError})`
            : "Reading the demo setting"
        }
      >
        <ToggleButton
          enabled={false}
          onEnabledChange={() => {}}
          label="Demo machines"
          iconOff="lu:FlaskConical"
          isLoading={!loadError}
          disabled
          className={compact}
        />
      </div>
    );
  }

  return (
    <>
      <ToggleButton
        enabled={enabled}
        onEnabledChange={(next) => setPending(next)}
        label="Demo machines"
        iconOn="lu:FlaskConical"
        iconOff="lu:FlaskConical"
        isLoading={isApplying}
        className={compact}
      />

      <Dialog
        open={pending !== null}
        onOpenChange={(open) => !open && setPending(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {pending ? "Enable demo machines?" : "Disable demo machines?"}
            </DialogTitle>
            <DialogDescription className="text-base">
              {pending
                ? "The backend will restart with a simulated production line (Extruder, Aquapath, Laser, Winder) next to the real machines. It then writes 30 days of history at the real data rate into the database. This takes several GB of disk space and keeps the CPU busy for a while."
                : "The backend will restart without the demo line. Its data stays in the database until you delete it below or it expires after 30 days."}{" "}
              Restarting stops every machine that is currently running.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <TouchButton variant="outline" onClick={() => setPending(null)}>
              Cancel
            </TouchButton>
            <TouchButton
              icon="lu:RefreshCw"
              isLoading={isApplying}
              onClick={() => pending !== null && apply(pending)}
            >
              Restart backend
            </TouchButton>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
