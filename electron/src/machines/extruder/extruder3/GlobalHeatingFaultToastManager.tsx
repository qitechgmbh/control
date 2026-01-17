import React, { useEffect, useRef } from "react";
import { toast } from "sonner";
import { z } from "zod";
import { useExtruder3Namespace } from "./extruder3Namespace";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
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
  const { request: requestAcknowledgeHeatingFault } = useMachineMutation(
    z.object({ AcknowledgeHeatingFault: z.literal(true) }),
  );

  // Deduplicate toasts by event timestamp
  const lastToastTs = useRef<number | string | null>(null);
  const currentToastId = useRef<string | null>(null);

  useEffect(() => {
    if (!state) return;

    const eventTs = (state as { ts?: number | string })?.ts ?? null;
    const faultZone = (state as { data?: { heating_fault_state?: { fault_zone?: string | null } } })
      ?.data?.heating_fault_state?.fault_zone ?? null;
    const faultAcknowledged =
      (state as { data?: { heating_fault_state?: { fault_acknowledged?: boolean } } })
        ?.data?.heating_fault_state?.fault_acknowledged ?? false;
    const isDefault = !!(state as { data?: { is_default_state?: boolean } })
      ?.data?.is_default_state;

    if (isDefault) return;

    const eventTsString = eventTs != null ? String(eventTs) : "no-ts";
    const faultZoneStr = faultZone ?? "unknown";
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
      const toastId = `heating-fault-${faultZoneStr}-${eventTsString}`;
      currentToastId.current = toastId;

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
                requestAcknowledgeHeatingFault({
                  machine_identification_unique: machineIdentification,
                  data: { AcknowledgeHeatingFault: true },
                });
                toast.dismiss(toastId);
                lastToastTs.current = null;
                currentToastId.current = null;
              }}
              aria-label="Close"
            >
              ×
            </button>
          </div>

          <p className="text-base leading-snug text-red-50">
            The <strong>{zoneName}</strong> heating zone did not increase in
            temperature by at least 5°C within 60 seconds. The extruder has been
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
      if (currentToastId.current) {
        toast.dismiss(currentToastId.current);
        currentToastId.current = null;
      }
      lastToastTs.current = null;
    }
  }, [
    state,
    requestAcknowledgeHeatingFault,
    machineIdentification,
  ]);

  return null;
}
