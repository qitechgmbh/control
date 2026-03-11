import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { useAquapath1 } from "./useAquapath";
import { SelectionGroup } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Badge } from "@/components/ui/badge";
import { Icon } from "@/components/Icon";
import { Label } from "@/control/Label";

export function Aquapath1ControlPage() {
  const {
    state,
    defaultState,
    front_flow,
    back_flow,
    front_temperature,
    back_temperature,
    front_heating,
    back_heating,
    front_revolutions,
    back_revolutions,
    setAquapathMode,
    setFrontTemperature,
    setBackTemperature,
    setFrontFlow,
    setBackFlow,
  } = useAquapath1();
  const reservoir1TargetTemperature =
    state?.temperature_states?.back.target_temperature ?? 0;
  const reservoir2TargetTemperature =
    state?.temperature_states?.front.target_temperature ?? 0;
  const minSettableTemperature = state?.ambient_temperature_calibration ?? 22;

  const reservoir1TargetFlow = state?.flow_states.back.should_flow ?? false;
  const reservoir2TargetFlow = state?.flow_states.front.should_flow ?? false;
  const reservoir1HeaterOn = back_heating;
  const reservoir2HeaterOn = front_heating;
  const reservoir1FanOn = (back_revolutions.current?.value ?? 0) > 0;
  const reservoir2FanOn = (front_revolutions.current?.value ?? 0) > 0;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Reservoir 1 (Back)">
          <div className="grid grid-rows-5 gap-4">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Flow"
                unit="l/min"
                timeseries={back_flow}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Temperature"
                unit="C"
                timeseries={back_temperature}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row items-end gap-4">
              <Label label="Set Target Temperature">
                <EditValue
                  title="Set Target Temperature"
                  min={minSettableTemperature}
                  value={reservoir1TargetTemperature}
                  max={80}
                  unit="C"
                  step={0.1}
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setBackTemperature(Math.max(val, minSettableTemperature));
                  }}
                  defaultValue={
                    defaultState?.temperature_states.back.target_temperature
                  }
                />
              </Label>

              {reservoir1HeaterOn && (
                <Badge
                  variant="default"
                  className="h-21 min-w-28 justify-center self-end px-4 text-base [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {reservoir1FanOn && (
                <Badge
                  variant="secondary"
                  className="h-21 min-w-28 justify-center self-end px-4 text-base [&>svg]:size-5"
                >
                  <Icon name="lu:Fan" className="size-5" />
                  Cooling
                </Badge>
              )}
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolution Speed"
                unit="%"
                timeseries={back_revolutions}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <Label label="Pump">
                <SelectionGroup<"On" | "Off">
                  value={reservoir1TargetFlow ? "On" : "Off"}
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
                    setBackFlow(value == "On");
                  }}
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Reservoir 2 (Front)">
          <div className="grid grid-rows-5 gap-4">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Flow"
                unit="l/min"
                timeseries={front_flow}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Temperature"
                unit="C"
                timeseries={front_temperature}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row items-end gap-4">
              <Label label="Set Target Temperature">
                <EditValue
                  title="Set Target Temperature"
                  min={minSettableTemperature}
                  value={reservoir2TargetTemperature}
                  max={80}
                  unit="C"
                  step={0.1}
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setFrontTemperature(Math.max(val, minSettableTemperature));
                  }}
                  defaultValue={
                    defaultState?.temperature_states.front.target_temperature
                  }
                />
              </Label>

              {reservoir2HeaterOn && (
                <Badge
                  variant="default"
                  className="h-21 min-w-28 justify-center self-end px-4 text-base [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {reservoir2FanOn && (
                <Badge
                  variant="secondary"
                  className="h-21 min-w-28 justify-center self-end px-4 text-base [&>svg]:size-5"
                >
                  <Icon name="lu:Fan" className="size-5" />
                  Cooling
                </Badge>
              )}
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolution Speed"
                unit="%"
                timeseries={front_revolutions}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <Label label="Pump">
                <SelectionGroup<"On" | "Off">
                  value={reservoir2TargetFlow ? "On" : "Off"}
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
                    setFrontFlow(value == "On");
                  }}
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
