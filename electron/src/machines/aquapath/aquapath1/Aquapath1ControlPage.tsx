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
    flow_sensor1,
    flow_sensor2,
    temperature_sensor1,
    temperature_sensor2,
    setAquapathMode,
    setFrontCoolingTemperature,
    setBackCoolingTemperature,
  } = useAquapath1();
  const frontTargetTemperature =
    state?.cooling_states.front.target_temperature ?? 0;
  const backTargetTemperature =
    state?.cooling_states.back.target_temperature ?? 0;
  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Flow Measurements">
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Flow on Sensor 1"
              unit="l/min"
              timeseries={flow_sensor1}
              renderValue={(value) => value.toFixed(1)}
            />
            <TimeSeriesValueNumeric
              label="Flow on Sensor 2"
              unit="l/min"
              timeseries={flow_sensor2}
              renderValue={(value) => value.toFixed(1)}
            />
          </div>
        </ControlCard>
        <ControlCard title="Temperature Measurements">
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Temperature on Sensor 1"
              unit="C"
              timeseries={temperature_sensor1}
              renderValue={(value) => value.toFixed(1)}
            />
            <TimeSeriesValueNumeric
              label="Temperature on Sensor 2"
              unit="C"
              timeseries={temperature_sensor2}
              renderValue={(value) => value.toFixed(1)}
            />
          </div>
        </ControlCard>
        <ControlCard title="Settings">
          <Label label="Set Front Target Diameter">
            <EditValue
              title="Set Front Cooling Target Temperature"
              value={frontTargetTemperature}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontCoolingTemperature(val);
              }}
              defaultValue={
                defaultState?.cooling_states.front.target_temperature
              }
            />
          </Label>
          <Label label="Set Back Target Diameter">
            <EditValue
              title="Set Back Cooling Target Temperature"
              value={backTargetTemperature}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackCoolingTemperature(val);
              }}
              defaultValue={
                defaultState?.cooling_states.back.target_temperature
              }
            />
          </Label>
        </ControlCard>
        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "Cool" | "Heat">
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
              Cool: {
                children: "Cooling",
                icon: "lu:Snowflake",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Heat: {
                children: "Heating",
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
