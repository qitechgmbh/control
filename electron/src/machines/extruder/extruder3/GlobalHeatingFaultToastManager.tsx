import React, { useEffect, useRef } from "react";
import { toast } from "sonner";
import { Icon } from "@/components/Icon";

type HeatingFaultState = {
  heating_fault_state?: {
    fault_zone?: string | null;
    fault_acknowledged?: boolean;
  };
  is_default_state?: boolean;
};

type HeatingFaultToastProps = {
  zoneName: string;
  onAcknowledge: () => void;
  children?: React.ReactNode;
};

function HeatingFaultToast({
  zoneName,
  onAcknowledge,
  children,
}: HeatingFaultToastProps) {
  return (
    <div className="flex w-100 flex-col gap-3 rounded-xl border border-red-400 bg-red-600 p-4 text-white shadow-xl backdrop-blur-sm transition-all duration-300">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Icon name="lu:TriangleAlert" className="h-5 w-5 text-yellow-200" />
          <strong className="text-lg font-semibold tracking-wide">
            Heating Fault
          </strong>
        </div>
        <button
          className="rounded-md p-1 text-2xl font-bold text-white/80 hover:bg-red-500 hover:text-white focus:ring-2 focus:ring-white/30 focus:outline-none"
          onClick={onAcknowledge}
          aria-label="Close"
        >
          ×
        </button>
      </div>

      {children ?? (
        <p className="text-base leading-snug text-red-50">
          The <strong>{zoneName}</strong> heating zone did not increase in
          temperature by at least 5°C within 60 seconds. The extruder has been
          automatically set to standby mode.
        </p>
      )}
    </div>
  );
}

function toZoneName(faultZone: string | null): string {
  switch (faultZone) {
    case "front":
      return "Front";
    case "middle":
      return "Middle";
    case "back":
      return "Back";
    case "nozzle":
      return "Nozzle";
    default:
      return "Unknown";
  }
}

function useHeatingFaultToastEffect({
  state,
  onAcknowledgeHeatingFault,
}: {
  state: HeatingFaultState | null | undefined;
  onAcknowledgeHeatingFault: () => void;
}) {
  // Deduplicate by active fault zone
  const activeFaultZone = useRef<string | null>(null);
  const currentToastId = useRef<string | null>(null);

  useEffect(() => {
    if (!state) return;
    const faultZone = state.heating_fault_state?.fault_zone ?? null;
    const faultAcknowledged =
      state.heating_fault_state?.fault_acknowledged ?? false;
    const isDefault = !!state.is_default_state;

    if (isDefault) return;

    const faultZoneStr = faultZone ?? "unknown";
    const zoneName = toZoneName(faultZone);

    if (
      faultZone &&
      !faultAcknowledged &&
      activeFaultZone.current !== faultZone
    ) {
      activeFaultZone.current = faultZone;
      const toastId = `heating-fault-${faultZoneStr}`;
      currentToastId.current = toastId;

      toast(
        <HeatingFaultToast
          zoneName={zoneName}
          onAcknowledge={() => {
            onAcknowledgeHeatingFault();
            toast.dismiss(toastId);
            activeFaultZone.current = null;
            currentToastId.current = null;
          }}
        />,
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

    if (faultAcknowledged || !faultZone) {
      if (currentToastId.current) {
        toast.dismiss(currentToastId.current);
        currentToastId.current = null;
      }
      activeFaultZone.current = null;
    }
  }, [state, onAcknowledgeHeatingFault]);
}

/**
 * Watches StateEvent for heating faults and shows toasts.
 * Component is non-visual.
 */
export function GlobalHeatingFaultToastManager({
  state,
  onAcknowledgeHeatingFault,
}: {
  state:
    | {
        heating_fault_state?: {
          fault_zone?: string | null;
          fault_acknowledged?: boolean;
        };
        is_default_state?: boolean;
      }
    | null
    | undefined;
  onAcknowledgeHeatingFault: () => void;
}) {
  useHeatingFaultToastEffect({ state, onAcknowledgeHeatingFault });

  return null;
}
