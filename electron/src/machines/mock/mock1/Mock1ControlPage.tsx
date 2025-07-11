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
  const { state, defaultState, sineWave, setFrequency, setMode, isDisabled } =
    useMock1();

  // Controlled local states synced with consolidated state
  const frequency = state?.sine_wave_state?.frequency ?? 1.0;
  const mode = state?.mode_state?.mode ?? "Standby";

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>

        <ControlCard title="Frequency">
          <div className="flex flex-col gap-4">
            <EditValue
              title="Frequency"
              unit="mHz"
              value={frequency}
              defaultValue={defaultState?.sine_wave_state.frequency}
              min={0.0}
              max={1000}
              step={0.1}
              renderValue={(value) => value.toFixed(0)}
              onChange={setFrequency}
            />
          </div>
        </ControlCard>

        <ControlCard title="Mode">
          <div className="flex flex-col gap-2">
            <div className="text-sm font-medium">Mode</div>
            <SelectionGroup
              value={mode}
              onChange={(newMode: Mode) => setMode(newMode)}
              disabled={isDisabled}
              options={{
                Standby: { children: "Standby" },
                Running: { children: "Running" },
              }}
            />
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
