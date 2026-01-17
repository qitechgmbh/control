import React, { useEffect, useRef } from "react";
import { toast } from "sonner";
import { useExtruder3Namespace } from "./extruder3Namespace";
import type { MachineIdentificationUnique } from "@/machines/types";
import { Icon } from "@/components/Icon";

/**
 * Global manager that toasts heating fault events.
 * Shows a persistent toast when a heating fault is detected.
 */
export function GlobalHeatingFaultToastManager({
  machineIdentification,
}: {
  machineIdentification: MachineIdentificationUnique;
}) {
  return (
    <HeatingFaultToastWatcher machineIdentification={machineIdentification} />
  );
}

/**
 * Watches StateEvent for heating faults and shows toasts.
 * Component is non-visual.
 */
function HeatingFaultToastWatcher({
  machineIdentification,
}: {
  machineIdentification: MachineIdentificationUnique;
}) {
  const { state } = useExtruder3Namespace(machineIdentification);

  // Deduplicate toasts by event timestamp
  const lastToastTs = useRef<number | string | null>(null);

  useEffect(() => {
    if (!state) return;

    const eventTs = (state as any)?.ts ?? null;
    const faultZone = (state as any)?.data?.heating_fault_state?.fault_zone;
    const faultAcknowledged =
      (state as any)?.data?.heating_fault_state?.fault_acknowledged;
    const isDefault = !!(state as any)?.data?.is_default_state;
    const toastId = `heating-fault-${faultZone}-${eventTs?.toString() ?? Date.now()}`;

    if (isDefault) return;

    const zoneName =
      faultZone === "front"
        ? "Front"
        : faultZone === "middle"
          ? "Middle"
          : faultZone === "back"
            ? "Back"
            : faultZone === "nozzle"
              ? "Nozzle"
              : "Unknown";

    // Show toast if fault detected and not acknowledged
    if (
      faultZone &&
      !faultAcknowledged &&
      lastToastTs.current !== eventTs
    ) {
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
                Heating Fault
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
            The <strong>{zoneName}</strong> heating zone did not increase in
            temperature by at least 5°C within 30 seconds. The extruder has been
            automatically set to standby mode.
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
    // Dismiss toast if fault is acknowledged or cleared
    if (faultAcknowledged || !faultZone) {
      toast.dismiss(toastId);
      lastToastTs.current = null;
    }
  }, [state]);

  return null;
}
