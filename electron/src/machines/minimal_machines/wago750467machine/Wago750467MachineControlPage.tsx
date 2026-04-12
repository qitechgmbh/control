import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { StatusBadge } from "@/control/StatusBadge";
import React from "react";
import { useWago750467Machine } from "./useWago750467Machine";

const CHANNEL_LABELS = ["AI 1", "AI 2"] as const;

const DEFAULT_STATE = {
  voltages: [0, 0],
  normalized: [0, 0],
  raw_words: [0, 0],
  wiring_errors: [false, false],
};

export function Wago750467MachineControlPage(): React.JSX.Element {
  const { state } = useWago750467Machine();
  const safeState = state ?? DEFAULT_STATE;

  return (
    <Page>
      <ControlCard title="Wago 750-467 Analog Inputs">
        <div className="grid grid-cols-2 gap-6">
          {CHANNEL_LABELS.map((label, index) => (
            <div
              key={label}
              className="flex flex-col gap-3 rounded-2xl border border-gray-100 bg-gray-50 p-4"
            >
              <span className="text-sm font-semibold text-gray-500">
                {label}
              </span>

              <div className="flex items-baseline gap-1">
                <span className="text-4xl font-bold tabular-nums">
                  {safeState.voltages[index].toFixed(3)}
                </span>
                <span className="text-lg text-gray-500">V</span>
              </div>

              <div className="text-sm text-gray-600">
                Normalized: {safeState.normalized[index].toFixed(4)}
              </div>
              <div className="text-sm text-gray-600">
                Raw Word: {safeState.raw_words[index]}
              </div>

              <div className="flex items-center gap-2">
                {safeState.wiring_errors[index] ? (
                  <StatusBadge variant="error">Wiring Error</StatusBadge>
                ) : (
                  <StatusBadge variant="success">OK</StatusBadge>
                )}
              </div>
            </div>
          ))}
        </div>
      </ControlCard>
    </Page>
  );
}
