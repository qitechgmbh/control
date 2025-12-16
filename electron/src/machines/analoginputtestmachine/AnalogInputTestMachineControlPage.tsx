import { MiniGraph } from "@/components/graph/MiniGraph";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";

import React from "react";
import { useAnalogInputTestMachine } from "./useAnalogInputTestMachine";

export function AnalogInputTestMachineControl(): React.JSX.Element {
  const { seriesData, state, updateMeasurementRate } =
    useAnalogInputTestMachine();

  if (!seriesData) return <div>Initializing Sensor...</div>;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Measurement Control">
          <div
            style={{
              display: "flex",
              flexDirection: "row",
              justifyContent: "space-between",
            }}
          >
            <div>Measurement Rate (Hz)</div>
            <input
              type="number"
              style={{
                border: "2px solid transparent",
                borderRadius: "4px",
                padding: "3px",
              }}
              className={"bg-neutral-100 shadow"}
              value={state.measurementRateHz}
              onChange={(v) => {
                const val = Number(v.currentTarget.value);
                updateMeasurementRate(val);
              }}
              min={1}
            ></input>
          </div>
        </ControlCard>
        <ControlCard title="Results">
          <div className="flex flex-row">
            <MiniGraph
              newData={seriesData}
              width={500}
              renderValue={(v) => v.toFixed(2)}
            ></MiniGraph>
            <div className="flex flex-col justify-center">
              {seriesData.current?.value.toFixed(2)}mA
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
