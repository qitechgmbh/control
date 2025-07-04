import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import React from "react";

import { Mode } from "./buffer1Namespace";
import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";

export function Buffer1ControlPage() {
  const {
    state,
    sine_wave,

    setBufferFrequency,
    setBufferMode,
  } = useBuffer1();

  const frequency = 1.0;

  return (
    <Page>
      <ControlGrid>
        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "FillingBuffer" | "EmptyingBuffer">
            value={state?.mode_state.mode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              FillingBuffer: {
                children: "FillingBuffer",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              EmptyingBuffer: {
                children: "EmptyingBuffer",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={setBufferMode}
          />
        </ControlCard>

        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Frequency">
          <div className="flex flex-col gap-4">
            <EditValue
              title="Frequency"
              unit="mHz"
              value={state?.sinewave_state.frequency}
              defaultValue={500}
              min={0.0}
              max={999}
              step={0.1}
              renderValue={(value) => value.toFixed(1)}
              onChange={setBufferFrequency}
            />
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
