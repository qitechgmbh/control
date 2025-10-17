import React, { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { useMainNamespace } from "@/client/mainNamespace";
import { useLaser1Namespace } from "@/machines/laser/laser1/laser1Namespace";
import { laser1 } from "@/machines/properties";
import type { MachineIdentificationUnique } from "@/machines/types";

/**
 * Global manager that toasts laser StateEvent changes (non-default) for a single laser.
 * If machineIdentification is omitted it will attempt to discover the laser via the main namespace.
 *
 * Mount once in App.tsx so it persists across routes.
 */
export function GlobalLaserToastManager({
  machineIdentification,
}: {
  machineIdentification?: MachineIdentificationUnique;
}) {
  const main = useMainNamespace();
  const [discovered, setDiscovered] = useState<MachineIdentificationUnique | null>(null);

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
    if (Array.isArray(machinesEvent?.data?.machines)) list = machinesEvent.data.machines;
    else if (Array.isArray(machinesEvent?.data)) list = machinesEvent.data;
    else if (Array.isArray(machinesEvent?.machines)) list = machinesEvent.machines;
    else if (machinesEvent && typeof machinesEvent === "object") {
      list = Object.values(machinesEvent).filter((v) => v && typeof v === "object");
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
        match?.device_machine_identification?.machine_identification_unique?.serial ??
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

  return effectiveId ? <LaserToastWatcher machineIdentification={effectiveId} /> : null;
}

/**
 * Watches StateEvent for the laser namespace and shows toasts for non-default states.
 * Component is non-visual.
 */
function LaserToastWatcher({ machineIdentification }: { machineIdentification: MachineIdentificationUnique }) {
  const { state } = useLaser1Namespace(machineIdentification);

  // Deduplicate toasts by event timestamp
  const lastToastTs = useRef<number | string | null>(null);

  useEffect(() => {
    if (!state) return;

    const eventTs = (state as any)?.ts ?? null;
    const isDefault = !!(state as any)?.data?.is_default_state;

    // skip the default snapshot emitted on connect
    if (isDefault) return;

    // dedupe identical events
    if (eventTs != null && lastToastTs.current === eventTs) return;
    lastToastTs.current = eventTs;

    try {
      const data = (state as any).data;
      const title = (state as any)?.name ?? "Laser State";
      const ls = data?.laser_state;
      const parts: string[] = [];
      if (ls) {
        if (typeof ls.target_diameter === "number") parts.push(`target ${ls.target_diameter.toFixed(2)} mm`);
        if (typeof ls.higher_tolerance === "number") parts.push(`+${ls.higher_tolerance.toFixed(2)} mm`);
        if (typeof ls.lower_tolerance === "number") parts.push(`-${ls.lower_tolerance.toFixed(2)} mm`);
      }

      const message = parts.length > 0 ? parts.join(" · ") : undefined;

      // Sonner toast call — adjust options if you want persistent/critical styling
      toast(
                <div className="bg-red-500 text-white p-4 rounded-lg shadow-lg flex flex-col gap-1 w-80">
                  <strong>${title}</strong>
                  <span>${message ? ` — ${message}` : ""}</span>
                  <button
                    className="self-end font-bold mt-2 hover:text-gray-200"
                    onClick={() => {
                      toast.dismiss(toastRef.current!);
                      toastRef.current = null;
                    }}
                  >
                    ×
                  </button>
                </div>,
                {
                  duration: Infinity,
                  position: "top-center",
                  style: { background: "transparent", padding: 0, boxShadow: "none", border: "none" },
                }
            );
    } catch (err) {
      console.error("GlobalLaserToastManager: failed to build toast", err);
      toast("Laser state changed");
    }
  }, [state]);

  return null;
}
