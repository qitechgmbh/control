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
  const frontTargetFlow = state?.flow_states.front.target_flow ?? 0;
  const backTargetFlow = state?.flow_states.back.target_flow ?? 0;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Flow Measurements">
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Front Flow"
              unit="l/min"
              timeseries={front_flow}
              renderValue={(value) => value.toFixed(1)}
            />
            <TimeSeriesValueNumeric
              label="Back Flow"
              unit="l/min"
              timeseries={back_flow}
              renderValue={(value) => value.toFixed(1)}
            />
          </div>
        </ControlCard>

        <ControlCard title="Flow Settings">
          <Label label="Set Front Target Flow">
            <EditValue
              title="Set Front Target Flow"
              value={frontTargetFlow}
              unit="l/min"
              max={10}
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontFlow(val);
              }}
              defaultValue={
                defaultState?.temperature_states.front.target_temperature
              }
            />
          </Label>
          <Label label="Set Back Target Flow">
            <EditValue
              title="Set Back Cooling Target Flow"
              value={backTargetFlow}
              unit="l/min"
              max={10}
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackFlow(val);
              }}
              defaultValue={
                defaultState?.temperature_states.back.target_temperature
              }
            />
          </Label>
        </ControlCard>
        <ControlCard title="Temperature Measurements">
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Temperature on Sensor 1"
              unit="C"
              timeseries={front_temperature}
              renderValue={(value) => value.toFixed(1)}
            />
            <TimeSeriesValueNumeric
              label="Temperature on Sensor 2"
              unit="C"
              timeseries={back_temperature}
              renderValue={(value) => value.toFixed(1)}
            />
          </div>

          <div className="flex flex-row items-center gap-6">
            <Label label="Set Front Target Diameter">
              <EditValue
                title="Set Front Target Temperature"
                value={frontTargetTemperature}
                unit="C"
                renderValue={(value) => value.toFixed(1)}
                onChange={(val) => {
                  setFrontTemperature(val);
                }}
                defaultValue={
                  defaultState?.temperature_states.front.target_temperature
                }
              />
            </Label>
            <Label label="Set Back Target Diameter">
              <EditValue
                title="Set Back Target Temperature"
                value={backTargetTemperature}
                unit="C"
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
              // Cool: {
              //   children: "Cooling",
              //   icon: "lu:Snowflake",
              //   isActiveClassName: "bg-green-600",
              //   className: "h-full",
              // },
              // Heat: {
              //   children: "Heating",
              //   icon: "lu:Flame",
              //   isActiveClassName: "bg-green-600",
              //   className: "h-full",
              // },
              Auto: {
                children: "Auto",
                icon: "lu:Flame",
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
