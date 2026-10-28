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
  getEthercatEnabled,
  setEthercatEnabled,
} from "@/helpers/troubleshoot_helpers";
import { useBackendConnected } from "@/client/socketioStore";

function Layout({
  children,
  hint,
}: {
  children: React.ReactNode;
  hint: string;
}) {
  return (
    <div className="flex shrink-0 items-center" title={hint}>
      {children}
    </div>
  );
}

const compact = "px-4 py-2 text-sm";

export function EthercatEnabledToggle() {
  const backendConnected = useBackendConnected();
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [pending, setPending] = useState<boolean | null>(null);
  const [isApplying, setIsApplying] = useState(false);

  // Re-read whenever the backend connection changes
  useEffect(() => {
    let cancelled = false;
    getEthercatEnabled().then((result) => {
      if (cancelled) return;
      if (result.success && result.enabled !== undefined) {
        setEnabled(result.enabled);
        setLoadError(null);
      } else {
        console.error("Could not read the EtherCAT setting", result.error);
        setLoadError(result.error ?? "unknown error");
      }
    });
    return () => {
      cancelled = true;
    };
  }, [backendConnected]);

  const apply = async (next: boolean) => {
    setIsApplying(true);
    const result = await setEthercatEnabled(next);
    if (result.success) {
      setEnabled(next);
      toast.success(
        next
          ? "EtherCAT enabled, restarting backend"
          : "EtherCAT disabled, restarting backend",
      );
    } else {
      toast.error(`Failed to change the EtherCAT setting: ${result.error}`);
    }
    setIsApplying(false);
    setPending(null);
  };

  if (enabled === null) {
    return (
      <Layout
        hint={
          loadError
            ? `Could not read the EtherCAT setting (${loadError})`
            : "Reading the EtherCAT setting"
        }
      >
        <ToggleButton
          enabled={false}
          onEnabledChange={() => {}}
          label="EtherCAT"
          iconOff="lu:Unplug"
          isLoading={!loadError}
          disabled
          className={compact}
        />
      </Layout>
    );
  }

  return (
    <>
      <Layout
        hint={
          enabled
            ? "Turn off for standalone units with only serial machines attached, such as a laser on its own. Left on, the backend waits for an EtherCAT coupler before any machine starts."
            : "Running serial machines only. Turn on if this unit has an EtherCAT coupler."
        }
      >
        <ToggleButton
          enabled={enabled}
          onEnabledChange={(next) => setPending(next)}
          label="EtherCAT"
          iconOn="lu:Network"
          iconOff="lu:Unplug"
          isLoading={isApplying}
          className={compact}
        />
      </Layout>

      <Dialog
        open={pending !== null}
        onOpenChange={(open) => !open && setPending(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {pending ? "Enable EtherCAT?" : "Disable EtherCAT?"}
            </DialogTitle>
            <DialogDescription className="text-base">
              {pending
                ? "The backend will restart and wait for an EtherCAT coupler to answer. If none is connected, it will keep waiting and no machines will start."
                : "The backend will restart and skip EtherCAT entirely. Only serial machines, such as a laser, will run."}{" "}
              Restarting stops every machine that is currently running.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <TouchButton variant="outline" onClick={() => setPending(null)}>
              Cancel
            </TouchButton>
            <TouchButton
              icon="lu:RefreshCw"
              onClick={() => pending !== null && apply(pending)}
              isLoading={isApplying}
            >
              Save and restart
            </TouchButton>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
