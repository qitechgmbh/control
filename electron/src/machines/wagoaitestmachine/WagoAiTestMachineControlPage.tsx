import { MiniGraph } from "@/components/graph/MiniGraph";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";

import React from "react";
import { useWagoAiTestMachine } from "./useWagoAiTestMachine";

export function WagoAiTestMachineControl(): React.JSX.Element {
  const { seriesData, state, updateMeasurementRate } = useWagoAiTestMachine();

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
        <ControlCard title="Analog Input 1 (4-20mA)">
          <div className="flex flex-row">
            <MiniGraph
              newData={seriesData.ai1}
              width={400}
              renderValue={(v) => v.toFixed(2)}
            ></MiniGraph>
            <div className="flex flex-col justify-center ml-4">
              <div>{seriesData.ai1.current?.value.toFixed(2)} mA</div>
              {state.wiringErrors?.[0] && (
                <div className="mt-2 px-2 py-1 bg-red-100 text-red-800 rounded text-sm font-semibold">
                  Wiring Error
                </div>
              )}
            </div>
          </div>
        </ControlCard>
        <ControlCard title="Analog Input 2 (4-20mA)">
          <div className="flex flex-row">
            <MiniGraph
              newData={seriesData.ai2}
              width={400}
              renderValue={(v) => v.toFixed(2)}
            ></MiniGraph>
            <div className="flex flex-col justify-center ml-4">
              <div>{seriesData.ai2.current?.value.toFixed(2)} mA</div>
              {state.wiringErrors?.[1] && (
                <div className="mt-2 px-2 py-1 bg-red-100 text-red-800 rounded text-sm font-semibold">
                  Wiring Error
                </div>
              )}
            </div>
          </div>
        </ControlCard>
        <ControlCard title="Analog Input 3 (4-20mA)">
          <div className="flex flex-row">
            <MiniGraph
              newData={seriesData.ai3}
              width={400}
              renderValue={(v) => v.toFixed(2)}
            ></MiniGraph>
            <div className="flex flex-col justify-center ml-4">
              <div>{seriesData.ai3.current?.value.toFixed(2)} mA</div>
              {state.wiringErrors?.[2] && (
                <div className="mt-2 px-2 py-1 bg-red-100 text-red-800 rounded text-sm font-semibold">
                  Wiring Error
                </div>
              )}
            </div>
          </div>
        </ControlCard>
        <ControlCard title="Analog Input 4 (4-20mA)">
          <div className="flex flex-row">
            <MiniGraph
              newData={seriesData.ai4}
              width={400}
              renderValue={(v) => v.toFixed(2)}
            ></MiniGraph>
            <div className="flex flex-col justify-center ml-4">
              <div>{seriesData.ai4.current?.value.toFixed(2)} mA</div>
              {state.wiringErrors?.[3] && (
                <div className="mt-2 px-2 py-1 bg-red-100 text-red-800 rounded text-sm font-semibold">
                  Wiring Error
                </div>
              )}
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
