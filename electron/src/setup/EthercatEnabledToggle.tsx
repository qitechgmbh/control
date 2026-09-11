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
  EthercatSetting,
  getEthercatSetting,
  setEthercatSetting,
} from "@/helpers/ethercat_settings";
import { restartBackend } from "@/helpers/troubleshoot_helpers";
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
  const [setting, setSetting] = useState<EthercatSetting | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [pending, setPending] = useState<boolean | null>(null);
  const [isApplying, setIsApplying] = useState(false);

  // Re-read whenever the backend connection changes
  useEffect(() => {
    let cancelled = false;
    getEthercatSetting()
      .then((value) => {
        if (cancelled) return;
        setSetting(value);
        setLoadError(null);
      })
      .catch((error) => {
        if (cancelled) return;
        console.error("Could not read the EtherCAT setting", error);
        setLoadError(error instanceof Error ? error.message : String(error));
      });
    return () => {
      cancelled = true;
    };
  }, [backendConnected]);

  const apply = async (enabled: boolean) => {
    setIsApplying(true);
    try {
      const next = await setEthercatSetting(enabled);
      setSetting(next);

      const result = await restartBackend();
      if (result.success) {
        toast.success(
          enabled
            ? "EtherCAT enabled, restarting backend"
            : "EtherCAT disabled, restarting backend",
        );
      } else {
        // The setting is saved either way, so say what still needs doing.
        toast.error(
          `Setting saved, but the backend restart failed: ${result.error}. Restart it from Troubleshoot.`,
        );
      }
    } catch (error) {
      toast.error(`Failed to change the EtherCAT setting: ${error}`);
    } finally {
      setIsApplying(false);
      setPending(null);
    }
  };

  if (!setting) {
    return (
      <Layout
        hint={
          loadError
            ? `The backend did not answer (${loadError}). It may predate this setting — update and restart it.`
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
          setting.enabled
            ? "Turn off for standalone units with only serial machines attached, such as a laser on its own. Left on, the backend waits for an EtherCAT coupler before any machine starts."
            : "Running serial machines only. Turn on if this unit has an EtherCAT coupler."
        }
      >
        <ToggleButton
          enabled={setting.enabled}
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
