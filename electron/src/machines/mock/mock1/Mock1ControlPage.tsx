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
    title: string,
    timeseries: TimeSeries;
}

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
    sineWaves,
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
      <ControlGrid>
        <SineWaveCard title="SineWaveSum" timeseries={sineWaves.sineWaveSum} />
        <SineWaveCard title="Sine Wave 1" timeseries={sineWaves.sineWave1} />
        <SineWaveCard title="Sine Wave 2" timeseries={sineWaves.sineWave2} />
        <SineWaveCard title="Sine Wave 3" timeseries={sineWaves.sineWave3} />

        <ControlCard title="Frequency">
          <div className="flex flex-col gap-4">
            <EditValue
              title="Frequency"
              unit="mHz"
              value={frequency}
              defaultValue={500}
              min={0.0}
              max={1000}
              step={0.1}
              renderValue={(value) => value.toFixed(0)}
              onChange={mockSetFrequency}
            />
          </div>
        </ControlCard>

        <ControlCard title="Mode">
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
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
