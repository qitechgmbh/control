import React, { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { useMainNamespace } from "@/client/mainNamespace";
import { useLaser1Namespace } from "@/machines/laser/laser1/laser1Namespace";
import { laser1 } from "@/machines/properties";
import type { MachineIdentificationUnique } from "@/machines/types";
import { Icon } from "@/components/Icon";

/**
 * Global manager that toasts laser StateEvent changes.
 * If machineIdentification is omitted it will attempt to discover the laser via the main namespace.
 *
 */
export function GlobalLaserToastManager({
  machineIdentification,
}: {
  machineIdentification?: MachineIdentificationUnique;
}) {
  const main = useMainNamespace();
  const [discovered, setDiscovered] =
    useState<MachineIdentificationUnique | null>(null);

  // Prefer explicit prop, otherwise use discovered id
  const effectiveId = machineIdentification ?? discovered;

  // Attempt to discover the single laser machine from the main namespace
  useEffect(() => {
    if (machineIdentification) return;
    const machinesEvent = (main as any)?.machines;
    if (!machinesEvent) {
      setDiscovered(null);
      return;
    }

    // Defensive shapes: try common structures
    let list: any[] = [];
    if (Array.isArray(machinesEvent?.data?.machines))
      list = machinesEvent.data.machines;
    else if (Array.isArray(machinesEvent?.data)) list = machinesEvent.data;
    else if (Array.isArray(machinesEvent?.machines))
      list = machinesEvent.machines;
    else if (machinesEvent && typeof machinesEvent === "object") {
      list = Object.values(machinesEvent).filter(
        (v) => v && typeof v === "object",
      );
    }

    const match = list.find((entry) => {
      const mi =
        entry?.machine_identification ||
        entry?.machine_identification_unique?.machine_identification ||
        entry?.device_machine_identification?.machine_identification;
      if (!mi) return false;
      return (
        mi.vendor === (laser1.machine_identification as any).vendor &&
        mi.machine === (laser1.machine_identification as any).machine
      );
    });

    if (match) {
      const serial =
        match?.serial ??
        match?.machine_identification_unique?.serial ??
        match?.device_machine_identification?.machine_identification_unique
          ?.serial ??
        null;
      if (serial != null && !Number.isNaN(Number(serial))) {
        setDiscovered({
          machine_identification: laser1.machine_identification,
          serial: Number(serial),
        });
        return;
      }
    }

    setDiscovered(null);
  }, [main, machineIdentification]);

  return effectiveId ? (
    <LaserToastWatcher machineIdentification={effectiveId} />
  ) : null;
}

/**
 * Watches StateEvent for the laser namespace and shows toasts for non-default states.
 * Component is non-visual.
 */
function LaserToastWatcher({
  machineIdentification,
}: {
  machineIdentification: MachineIdentificationUnique;
}) {
  const { state } = useLaser1Namespace(machineIdentification);

  // Deduplicate toasts by event timestamp
  const lastToastTs = useRef<number | string | null>(null);

  useEffect(() => {
    if (!state) return;

    const eventTs = (state as any)?.ts ?? null;
    const inTolerance = (state as any)?.data?.laser_state.in_tolerance;
    const isDefault = !!(state as any)?.data?.is_default_state;
    const toastId = eventTs?.toString() ?? String(Date.now());

    if (isDefault) return;

    // Only show toast if laser is out of tolerance and not already shown for this timestamp
    if (!inTolerance && lastToastTs.current !== eventTs) {
      lastToastTs.current = eventTs;
      // Sonner toast call
      toast(
        <div className="flex w-100 flex-col gap-3 rounded-xl border border-red-400 bg-red-600 p-4 text-white shadow-xl backdrop-blur-sm transition-all duration-300">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Icon
                name="lu:TriangleAlert"
                className="h-5 w-5 text-yellow-200"
              />
              <strong className="text-lg font-semibold tracking-wide">
                Warning
              </strong>
            </div>
            <button
              className="rounded-md p-1 text-2xl font-bold text-white/80 hover:bg-red-500 hover:text-white focus:ring-2 focus:ring-white/30 focus:outline-none"
              onClick={() => {
                toast.dismiss(toastId);
                lastToastTs.current = null;
              }}
              aria-label="Close"
            >
              ×
            </button>
          </div>

          <p className="text-base leading-snug text-red-50">
            Laser diameter is <strong>out of tolerance</strong>.<br />
            Please check the filament immediately.
          </p>
        </div>,
        {
          id: toastId,
          duration: Infinity,
          position: "top-center",
          style: {
            background: "transparent",
            padding: 0,
            boxShadow: "none",
            border: "none",
          },
        },
      );
    }
    if (inTolerance) {
      lastToastTs.current = null;
    }
  }, [state]);

  return null;
}
