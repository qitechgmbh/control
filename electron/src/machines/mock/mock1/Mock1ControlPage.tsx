import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import { useMock1 } from "./useMock";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Mode } from "./mock1Namespace";

export function Mock1ControlPage() {
  const {
    sineWave,
    mockState,
    modeState,
    mockSetFrequency,
    mockSetMode,
    modeStateIsDisabled,
  } = useMock1();

  // Controlled local states synced with mockState and modeState
  const frequency = mockState?.data?.frequency ?? 1.0;
  const mode = modeState?.data?.mode ?? "Standby";

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Sine Wave Generator">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>

        <ControlCard title="Settings">
          <div className="flex flex-col gap-4">
            <EditValue
              title="Frequency (Hz)"
              unit="mHz"
              value={frequency}
              defaultValue={500}
              min={1}
              max={1000}
              step={10}
              renderValue={(value) => value.toFixed(0)}
              onChange={mockSetFrequency}
            />

            <div className="flex flex-col gap-2">
              <div className="text-sm font-medium">Mode</div>
              <SelectionGroup
                value={mode}
                onChange={(newMode: Mode) => mockSetMode(newMode)}
                disabled={modeStateIsDisabled}
                options={{
                  Standby: { children: "Standby" },
                  Running: { children: "Running" },
                }}
              />
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
