import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { useAquapath1 } from "./useAquapath";
import { SelectionGroup } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

export function Aquapath1ControlPage() {
  const {
    state,
    defaultState,
    front_flow,
    back_flow,
    front_temperature,
    back_temperature,
    front_temp_reservoir,
    back_temp_reservoir,
    setAquapathMode,
    setFrontTemperature,
    setBackTemperature,
    setFrontFlow,
    setBackFlow,
  } = useAquapath1();
  const frontTargetTemperature =
    state?.temperature_states?.front.target_temperature ?? 0;
  const backTargetTemperature =
    state?.temperature_states?.back.target_temperature ?? 0;

  const frontTargetFlow = state?.flow_states.front.should_flow ?? false;
  const backTargetFlow = state?.flow_states.back.should_flow ?? false;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Flow Measurements">
          <div className="grid grid-cols-2 gap-6">
            <div className="flex flex-col gap-8">
              <TimeSeriesValueNumeric
                label="Flow 1"
                unit="l/min"
                timeseries={front_flow}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-col gap-8">
              <TimeSeriesValueNumeric
                label="Flow 2"
                unit="l/min"
                timeseries={back_flow}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>
          </div>

          <div className="mt-8">
            <Label label="Pump 1">
              <SelectionGroup<"On" | "Off">
                value={frontTargetFlow ? "On" : "Off"}
                orientation="vertical"
                className="grid h-full grid-cols-2 gap-2"
                options={{
                  Off: {
                    children: "Off",
                    icon: "lu:CirclePause",
                    isActiveClassName: "bg-green-600",
                    className: "h-full",
                  },
                  On: {
                    children: "On",
                    icon: "lu:CirclePlay",
                    isActiveClassName: "bg-green-600",
                    className: "h-full",
                  },
                }}
                onChange={(value) => {
                  if (value == "On") {
                    setFrontFlow(true);
                  } else {
                    setFrontFlow(false);
                  }
                }}
              />
            </Label>

            <Label label="Pump 2">
              <SelectionGroup<"On" | "Off">
                value={backTargetFlow ? "On" : "Off"}
                orientation="vertical"
                className="grid h-full grid-cols-2 gap-2"
                options={{
                  Off: {
                    children: "Off",
                    icon: "lu:CirclePause",
                    isActiveClassName: "bg-green-600",
                    className: "h-full",
                  },
                  On: {
                    children: "On",
                    icon: "lu:CirclePlay",
                    isActiveClassName: "bg-green-600",
                    className: "h-full",
                  },
                }}
                onChange={(value) => {
                  if (value == "On") {
                    setBackFlow(true);
                  } else {
                    setBackFlow(false);
                  }
                }}
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard title="Temperature Measurements">
          <div className="grid grid-cols-2 gap-6">
            <div className="flex flex-col gap-8">
              <TimeSeriesValueNumeric
                label="Reservoir 1"
                unit="C"
                timeseries={front_temperature}
                renderValue={(value) => value.toFixed(1)}
              />
              <Label label="Set Reservoir 1 Target Temperature">
                <EditValue
                  title="Set Reservoir 1 Target Temperature"
                  value={frontTargetTemperature}
                  unit="C"
                  step={0.1}
                  min={0.0}
                  max={80.0}
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setFrontTemperature(val);
                  }}
                  defaultValue={
                    defaultState?.temperature_states.front.target_temperature
                  }
                />
              </Label>
            </div>

            <div className="flex flex-col gap-8">
              <TimeSeriesValueNumeric
                label="Reservoir 2"
                unit="C"
                timeseries={back_temperature}
                renderValue={(value) => value.toFixed(1)}
              />
              <Label label="Set Reservoir 2 Target Temperature">
                <EditValue
                  title="Set Reservoir 2 Target Temperature"
                  value={backTargetTemperature}
                  unit="C"
                  step={0.1}
                  min={0.0}
                  max={80.0}
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setBackTemperature(val);
                  }}
                  defaultValue={
                    defaultState?.temperature_states.back.target_temperature
                  }
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "Auto">
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
              Auto: {
                children: "Auto",
                icon: "lu:CirclePlay",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={(value) => setAquapathMode(value)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
