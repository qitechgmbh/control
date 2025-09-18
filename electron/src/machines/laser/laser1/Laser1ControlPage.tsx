import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric, NumericValue } from "@/control/TimeSeriesValue";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/Icon";

import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { useLaser1 } from "./useLaser1";

import { DiameterVisualisation } from "../DiameterVisualisation";

export function Laser1ControlPage() {
  const {
    diameter,
    x_value,
    y_value,
    state,
    defaultState,
    setTargetDiameter,
    setLowerTolerance,
    setHigherTolerance,
    diameterMinMax,
    xValueMinMax,
    yValueMinMax,
    minMaxTimeWindow,
    setMinMaxTimeWindow,
  } = useLaser1();

  // Extract values from consolidated state
  const targetDiameter = state?.laser_state?.target_diameter ?? 0;
  const lowerTolerance = state?.laser_state?.lower_tolerance ?? 0;
  const higherTolerance = state?.laser_state?.higher_tolerance ?? 0;
  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Diameter Measurement">
          <DiameterVisualisation
            targetDiameter={targetDiameter}
            lowTolerance={lowerTolerance}
            highTolerance={higherTolerance}
            diameter={diameter}
            x_value={x_value}
            y_value={y_value}
          />
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Diameter"
              unit="mm"
              timeseries={diameter}
              renderValue={(value) => value.toFixed(3)}
            />
          </div>
          <div className="flex flex-row items-center gap-6">
            <NumericValue
              label="Min Diameter"
              unit="mm"
              value={diameterMinMax.min}
              renderValue={(value) => value.toFixed(3)}
            />
            <NumericValue
              label="Max Diameter"
              unit="mm"
              value={diameterMinMax.max}
              renderValue={(value) => value.toFixed(3)}
            />
          </div>
          {x_value?.current && (
            <>
              <div className="flex flex-row items-center gap-6">
                <TimeSeriesValueNumeric
                  label="X-Diameter"
                  unit="mm"
                  timeseries={x_value}
                  renderValue={(value) => value.toFixed(3)}
                />
              </div>
              <div className="flex flex-row items-center gap-6">
                <NumericValue
                  label="Min X-Diameter"
                  unit="mm"
                  value={xValueMinMax.min}
                  renderValue={(value) => value.toFixed(3)}
                />
                <NumericValue
                  label="Max X-Diameter"
                  unit="mm"
                  value={xValueMinMax.max}
                  renderValue={(value) => value.toFixed(3)}
                />
              </div>
            </>
          )}
          {y_value?.current && (
            <>
              <div className="flex flex-row items-center gap-6">
                <TimeSeriesValueNumeric
                  label="Y-Diameter"
                  unit="mm"
                  timeseries={y_value}
                  renderValue={(value) => value.toFixed(3)}
                />
              </div>
              <div className="flex flex-row items-center gap-6">
                <NumericValue
                  label="Min Y-Diameter"
                  unit="mm"
                  value={yValueMinMax.min}
                  renderValue={(value) => value.toFixed(3)}
                />
                <NumericValue
                  label="Max Y-Diameter"
                  unit="mm"
                  value={yValueMinMax.max}
                  renderValue={(value) => value.toFixed(3)}
                />
              </div>
            </>
          )}
        </ControlCard>
        <ControlCard title="Settings">
          <Label label="Min/Max Time Window">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <TouchButton
                  variant="outline"
                  className="h-auto w-full justify-between border-gray-300 bg-white px-3 py-3 text-base text-gray-900 hover:bg-gray-50"
                >
                  {(() => {
                    const timeWindowOptions = [
                      { value: 10 * 1000, label: "10s" },
                      { value: 30 * 1000, label: "30s" },
                      { value: 1 * 60 * 1000, label: "1m" },
                      { value: 5 * 60 * 1000, label: "5m" },
                      { value: 10 * 60 * 1000, label: "10m" },
                      { value: 30 * 60 * 1000, label: "30m" },
                      { value: 1 * 60 * 60 * 1000, label: "1h" },
                    ];
                    const option = timeWindowOptions.find((opt) => opt.value === minMaxTimeWindow);
                    return option ? option.label : "5m";
                  })()}
                  <Icon name="lu:ChevronDown" className="ml-2 size-4" />
                </TouchButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuLabel className="text-base font-medium">
                  Time Window for Min/Max
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                {[
                  { value: 10 * 1000, label: "10s" },
                  { value: 30 * 1000, label: "30s" },
                  { value: 1 * 60 * 1000, label: "1m" },
                  { value: 5 * 60 * 1000, label: "5m" },
                  { value: 10 * 60 * 1000, label: "10m" },
                  { value: 30 * 60 * 1000, label: "30m" },
                  { value: 1 * 60 * 60 * 1000, label: "1h" },
                ].map((option) => (
                  <DropdownMenuItem
                    key={option.value}
                    onClick={() => setMinMaxTimeWindow(option.value)}
                    className={`min-h-[48px] px-4 py-3 text-base ${
                      minMaxTimeWindow === option.value ? "bg-blue-50" : ""
                    }`}
                  >
                    {option.label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
          </Label>
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
