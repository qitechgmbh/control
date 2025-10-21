import React, { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { useMainNamespace } from "@/client/mainNamespace";
import { useLaser1Namespace } from "@/machines/laser/laser1/laser1Namespace";
import { laser1 } from "@/machines/properties";
import type { MachineIdentificationUnique } from "@/machines/types";

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
    if (!machinesEvent?.data?.machines) {
      setDiscovered(null);
      return;
    }

    const machines: any[] = machinesEvent.data.machines;

    const match = machines.find((entry) => {
      const mi = entry?.machine_identification;
      return (
        mi?.vendor === laser1.machine_identification.vendor &&
        mi?.machine === laser1.machine_identification.machine
      );
    });

    if (!match) {
      setDiscovered(null);
      return;
    }

    const serial = Number(match?.serial);
    if (!Number.isNaN(serial)) {
      setDiscovered({
        machine_identification: laser1.machine_identification,
        serial,
      });
    } else {
      setDiscovered(null);
    }
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
    const isDefault = !!(state as any)?.data?.is_default_state;
    const inTolereance = (state as any)?.data?.laser_state.in_tolerance;

    // skip the default snapshot emitted on connect
    if (isDefault) return;

    // dedupe identical events
    if (eventTs != null && lastToastTs.current === eventTs) return;
    lastToastTs.current = eventTs;

    // skip if diameter is inside Tolerance
    if (inTolereance) return;

    const toastId = eventTs?.toString() ?? String(Date.now());

    try {
      // Sonner toast call
      toast(
        <div className="flex w-80 flex-col gap-1 rounded-lg bg-red-500 p-4 text-base text-white shadow-lg">
          <strong>Warning</strong>
          <span>Laser Diameter out of Tolerance!</span>
          <button
            className="mt-4 self-end text-2xl font-bold hover:text-gray-200"
            onClick={() => {
              toast.dismiss(toastId);
              lastToastTs.current = null;
            }}
          >
            ×
          </button>
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
    } catch (err) {
      console.error("GlobalLaserToastManager: failed to build toast", err);
      toast("Laser state changed");
    }
  }, [state]);

  return null;
}
