import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { MinMaxValue, TIMEFRAME_OPTIONS } from "@/control/MinMaxValue";

import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { useLaser1 } from "./useLaser1";

import { DiameterVisualisation } from "../DiameterVisualisation";

export function Laser1ControlPage() {
  const {
    diameter,
    x_diameter,
    y_diameter,
    roundness,
    state,
    defaultState,
    setTargetDiameter,
    setLowerTolerance,
    setHigherTolerance,
  } = useLaser1();

  // Extract values from consolidated state
  const targetDiameter = state?.laser_state?.target_diameter ?? 0;
  const lowerTolerance = state?.laser_state?.lower_tolerance ?? 0;
  const higherTolerance = state?.laser_state?.higher_tolerance ?? 0;

  // Detect if this is a 2-axis laser
  const isTwoAxis = !!x_diameter?.current || !!y_diameter?.current;
  // Shared timeframe state (default 5 minutes)
  const [timeframe, setTimeframe] = React.useState<number>(5 * 60 * 1000);

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Diameter Measurement">
          <DiameterVisualisation
            targetDiameter={targetDiameter}
            lowTolerance={lowerTolerance}
            highTolerance={higherTolerance}
            diameter={diameter}
            x_diameter={x_diameter}
            y_diameter={y_diameter}
          />
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Diameter"
              unit="mm"
              timeseries={diameter}
              renderValue={(value) => value.toFixed(3)}
            />
          </div>
          {x_diameter?.current && (
            <div className="flex flex-row items-center gap-6">
              <TimeSeriesValueNumeric
                label="X-Diameter"
                unit="mm"
                timeseries={x_diameter}
                renderValue={(value) => value.toFixed(3)}
              />
            </div>
          )}
          {y_diameter?.current && (
            <div className="flex flex-row items-center gap-6">
              <TimeSeriesValueNumeric
                label="Y-Diameter"
                unit="mm"
                timeseries={y_diameter}
                renderValue={(value) => value.toFixed(3)}
              />
            </div>
          )}
          {roundness?.current && (
            <div className="flex flex-row items-center gap-6">
              <TimeSeriesValueNumeric
                label="Roundness"
                unit="%"
                timeseries={roundness}
                renderValue={(value) => (value * 100).toFixed(2)}
              />
            </div>
          )}
          <div className="mt-4 border-t pt-4">
            {isTwoAxis ? (
              // For 2-axis lasers: show diameter and roundness min/max side by side
              <div className="grid grid-cols-2 gap-4">
                <MinMaxValue
                  label="Diameter Range"
                  unit="mm"
                  timeseries={diameter}
                  renderValue={(value) => value.toFixed(3)}
                  timeframe={timeframe}
                  hideSelector
                />
                {roundness?.current && (
                  <MinMaxValue
                    label="Roundness Range"
                    unit="%"
                    timeseries={roundness}
                    renderValue={(value) => (value * 100).toFixed(2)}
                    timeframe={timeframe}
                    hideSelector
                  />
                )}
              </div>
            ) : (
              // For single-axis lasers: show only diameter min/max with shared selector
              <MinMaxValue
                label="Diameter Range"
                unit="mm"
                timeseries={diameter}
                renderValue={(value) => value.toFixed(3)}
                timeframe={timeframe}
                hideSelector
              />
            )}

            {/* Shared timeframe selector for diameter/roundness */}
            <div className="mt-3 flex flex-row flex-wrap gap-2">
              {TIMEFRAME_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  onClick={() => setTimeframe(option.value)}
                  className={`rounded-md px-3 py-1 text-sm transition-colors ${
                    timeframe === option.value
                      ? "bg-blue-500 text-white"
                      : "bg-gray-200 text-gray-700 hover:bg-gray-300"
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>
        </ControlCard>
        <ControlCard title="Settings">
          <Label label="Set Target Diameter">
            <EditValue
              title="Set Target Diameter"
              value={targetDiameter}
              unit="mm"
              step={0.01}
              min={0}
              max={5}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                if (val < lowerTolerance) {
                  setLowerTolerance(val);
                }
                setTargetDiameter(val);
              }}
              defaultValue={defaultState?.laser_state.target_diameter}
            />
          </Label>
          <Label label="Set Lower Tolerance">
            <EditValue
              title="Set Lower Tolerance"
              value={lowerTolerance}
              unit="mm"
              step={0.01}
              min={0}
              max={Math.min(targetDiameter, 1)}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setLowerTolerance(val)}
              defaultValue={defaultState?.laser_state.lower_tolerance}
            />
          </Label>
          <Label label="Set Higher Tolerance">
            <EditValue
              title="Set Higher Tolerance"
              value={higherTolerance}
              unit="mm"
              step={0.01}
              min={0}
              max={1}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setHigherTolerance(val)}
              defaultValue={defaultState?.laser_state.higher_tolerance}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
