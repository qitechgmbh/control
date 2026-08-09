import { TouchButton } from "@/components/touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import { StatusBadge } from "@/control/StatusBadge";
import { useNavigate } from "@tanstack/react-router";
import React from "react";

export type ExtruderLinkState = "ready" | "no-demand" | "fault";

export function ConnectedExtruderPanel({
  state,
  onStateChange,
}: {
  state: ExtruderLinkState;
  onStateChange: (state: ExtruderLinkState) => void;
}) {
  const navigate = useNavigate();
  return (
    <div className="grid gap-3">
      {state === "ready" ? (
        <StatusBadge variant="success">Ready for material</StatusBadge>
      ) : state === "fault" ? (
        <StatusBadge variant="error">Extruder fault</StatusBadge>
      ) : (
        <StatusBadge variant="warning">No material demand</StatusBadge>
      )}
      <div className="grid grid-cols-1 gap-2">
        {[
          { value: "ready" as const, label: "Ready for material" },
          { value: "no-demand" as const, label: "No demand" },
          { value: "fault" as const, label: "Fault" },
        ].map((option) => (
          <button
            key={option.value}
            className={`min-h-10 rounded-lg border px-3 text-left text-sm font-medium ${
              state === option.value
                ? "border-blue-300 bg-blue-50 text-blue-800"
                : "border-gray-200 bg-gray-50 text-gray-700"
            }`}
            onClick={() => onStateChange(option.value)}
          >
            {option.label}
          </button>
        ))}
      </div>
      <TouchButton
        variant="outline"
        icon="lu:ExternalLink"
        onClick={() =>
          navigate({ to: "/_sidebar/machines/extruder3/0/control" })
        }
      >
        Open Extruder
      </TouchButton>
    </div>
  );
}

export function ConnectedExtruderCard(props: {
  state: ExtruderLinkState;
  onStateChange: (state: ExtruderLinkState) => void;
}) {
  return (
    <ControlCard title="Connected Extruder">
      <ConnectedExtruderPanel {...props} />
    </ControlCard>
  );
}
