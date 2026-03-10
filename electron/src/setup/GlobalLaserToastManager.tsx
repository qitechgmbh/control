import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { useMainNamespace } from "@/client/mainNamespace";
import { useLaser1Namespace } from "@/machines/laser/laser1/laser1Namespace";
import { laser1 } from "@/machines/properties";
import type { MachinesEvent } from "@/client/mainNamespace";
import type { MachineIdentificationUnique } from "@/machines/types";
import { Icon } from "@/components/Icon";
import React from "react";

// Constants

const TOAST_ID = "laser-out-of-tolerance";

// Discovery

function discoverLaser(
  machines: MachinesEvent | null,
): MachineIdentificationUnique | null {
  const { vendor, machine } = laser1.machine_identification;

  return (
    machines?.data.machines.find(
      (m) =>
        m.machine_identification_unique.machine_identification.vendor ===
          vendor &&
        m.machine_identification_unique.machine_identification.machine ===
          machine,
    )?.machine_identification_unique ?? null
  );
}

// Toast UI

function LaserErrorToast({
  errorCount,
  onDismiss,
}: {
  errorCount: number;
  onDismiss: () => void;
}) {
  return (
    <div className="flex w-100 flex-col gap-3 rounded-xl border border-red-400 bg-red-600 p-4 text-white shadow-xl backdrop-blur-sm">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Icon name="lu:TriangleAlert" className="h-5 w-5 text-yellow-200" />
          <strong className="text-lg font-semibold tracking-wide">
            Warning
          </strong>
          {errorCount > 1 && (
            <span className="rounded-full bg-red-800 px-2 py-0.5 text-sm font-bold">
              ({errorCount})
            </span>
          )}
        </div>
        <button
          className="rounded-md p-1 text-2xl font-bold text-white/80 hover:bg-red-500 hover:text-white focus:ring-2 focus:ring-white/30 focus:outline-none"
          onClick={() => {
            toast.dismiss(TOAST_ID);
            onDismiss();
          }}
          aria-label="Close"
        >
          ×
        </button>
      </div>
      <p className="text-base leading-snug text-red-50">
        Laser diameter is <strong>out of tolerance</strong>.
        <br />
        Please check the filament immediately.
      </p>
    </div>
  );
}

function showLaserErrorToast(errorCount: number, onDismiss: () => void) {
  toast(<LaserErrorToast errorCount={errorCount} onDismiss={onDismiss} />, {
    id: TOAST_ID,
    duration: Infinity,
    position: "top-center",
    style: {
      background: "transparent",
      padding: 0,
      boxShadow: "none",
      border: "none",
    },
  });
}

// Watcher

function LaserToastWatcher({
  machineIdentification,
}: {
  machineIdentification: MachineIdentificationUnique;
}) {
  const { state } = useLaser1Namespace(machineIdentification);

  const errorCountRef = useRef(0);
  const lastSeenTs = useRef<number | string | null>(null);

  const resetRefs = () => {
    errorCountRef.current = 0;
    lastSeenTs.current = null;
  };

  // Dismiss any active toast when the component unmounts.
  useEffect(() => {
    return () => {
      toast.dismiss(TOAST_ID);
      resetRefs();
    };
  }, []);

  useEffect(() => {
    if (!state) return;

    const ts: number | string | null = (state as any)?.ts ?? null;
    const inTolerance: boolean = (state as any)?.data?.laser_state?.in_tolerance;
    const isDefault: boolean = !!(state as any)?.data?.is_default_state;
    const globalWarning: boolean = (state as any)?.data?.laser_state?.global_warning ?? false;

    // If global warning is disabled, dismiss any active toast and bail.
    if (!globalWarning) {
      toast.dismiss(TOAST_ID);
      resetRefs();
      return;
    }

    if (isDefault) return;

    if (!inTolerance && ts !== lastSeenTs.current) {
      lastSeenTs.current = ts;
      errorCountRef.current += 1;
      showLaserErrorToast(errorCountRef.current, resetRefs);
    }
  }, [state]);

  return null;
}

// Public export

export function GlobalLaserToastManager({
  machineIdentification,
}: {
  machineIdentification?: MachineIdentificationUnique;
}) {
  const machines = useMainNamespace((s) => s.machines);
  const [discovered, setDiscovered] =
    useState<MachineIdentificationUnique | null>(null);

  useEffect(() => {
    if (machineIdentification) return;
    setDiscovered(discoverLaser(machines));
  }, [machines, machineIdentification]);

  const effectiveId = machineIdentification ?? discovered;

  return effectiveId ? (
    <LaserToastWatcher machineIdentification={effectiveId} />
  ) : null;
}
