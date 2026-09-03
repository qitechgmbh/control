import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import { useMock1 } from "./useMock";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Mode } from "./mock1Namespace";
import { TimeSeries } from "@/lib/timeseries";

type SineWaveCardProps = {
  title: string;
  timeseries: TimeSeries;
};

function SineWaveCard({ title, timeseries }: SineWaveCardProps) {
  return (
    <ControlCard title={title}>
      <TimeSeriesValueNumeric
        label="Current Value"
        timeseries={timeseries}
        renderValue={(value) => value.toFixed(3)}
      />
    </ControlCard>
  );
}

export function Mock1ControlPage() {
  const {
    state,
    defaultState,
    sineWave1,
    sineWave2,
    sineWave3,
    sineWaveSum,
    setFrequency1,
    setFrequency2,
    setFrequency3,
    setMode,
    isDisabled,
  } = useMock1();

  // Controlled local states synced with consolidated state
  const frequency1 = state?.frequency1 ?? 1.0;
  const frequency2 = state?.frequency2 ?? 2.0;
  const frequency3 = state?.frequency3 ?? 5.0;
  const mode = state?.mode_state?.mode ?? "Standby";

  return (
    <Page>
      <ControlGrid columns={2}>
        <SineWaveCard title="Sine Wave Sum" timeseries={sineWaveSum} />
        <SineWaveCard title="Sine Wave 1" timeseries={sineWave1} />
        <SineWaveCard title="Sine Wave 2" timeseries={sineWave2} />
        <SineWaveCard title="Sine Wave 3" timeseries={sineWave3} />

        <ControlCard title="Frequency">
          <div className="flex flex-row gap-2">
            <EditValue
              title="Frequency 1"
              unit="mHz"
              value={frequency1}
              defaultValue={defaultState?.frequency1}
              min={0.0}
              max={1000}
              step={0.1}
              renderValue={(value) => value.toFixed(0)}
              onChange={setFrequency1}
            />
            <EditValue
              title="Frequency 2"
              unit="mHz"
              value={frequency2}
              defaultValue={defaultState?.frequency2}
              min={0.0}
              max={1000}
              step={0.1}
              renderValue={(value) => value.toFixed(0)}
              onChange={setFrequency2}
            />
            <EditValue
              title="Frequency 3"
              unit="mHz"
              value={frequency3}
              defaultValue={defaultState?.frequency3}
              min={0.0}
              max={1000}
              step={0.1}
              renderValue={(value) => value.toFixed(0)}
              onChange={setFrequency3}
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
