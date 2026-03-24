import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { StatusBadge } from "@/control/StatusBadge";
import React from "react";
import { useWago750460Machine } from "./useWago750460Machine";

const CHANNEL_LABELS = ["CH 1", "CH 2", "CH 3", "CH 4"] as const;

const DEFAULT_STATE = {
  temperatures: [null, null, null, null] as (number | null)[],
  errors: [false, false, false, false],
};

export function Wago750460MachineControlPage() {
  const { state } = useWago750460Machine();
  const safeState = state ?? DEFAULT_STATE;

  return (
    <Page>
      <ControlCard title="Pt1000 Temperature Inputs">
        <div className="grid grid-cols-2 gap-6">
          {CHANNEL_LABELS.map((label, index) => {
            const temperature = safeState.temperatures[index];
            const hasError = safeState.errors[index];

            return (
              <div
                key={index}
                className="flex flex-col gap-2 rounded-2xl border border-gray-100 bg-gray-50 p-4"
              >
                <span className="text-sm font-semibold text-gray-500">
                  {label}
                </span>

                {hasError || temperature === null ? (
                  <div className="flex items-center gap-2">
                    <StatusBadge variant="error">Wire Break</StatusBadge>
                  </div>
                ) : (
                  <div className="flex items-baseline gap-1">
                    <span className="text-4xl font-bold tabular-nums">
                      {temperature.toFixed(1)}
                    </span>
                    <span className="text-lg text-gray-500">°C</span>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </ControlCard>
    </Page>
  );
}
