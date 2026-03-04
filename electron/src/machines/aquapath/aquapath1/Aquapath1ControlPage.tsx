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
  const frontTargetTemperature =
    state?.temperature_states?.front.target_temperature ?? 0;
  const backTargetTemperature =
    state?.temperature_states?.back.target_temperature ?? 0;
  const minSettableTemperature = state?.ambient_temperature_calibration ?? 22;

  const frontTargetFlow = state?.flow_states.front.should_flow ?? false;
  const backTargetFlow = state?.flow_states.back.should_flow ?? false;
  const frontHeaterOn = front_heating;
  const backHeaterOn = back_heating;
  const frontFanOn = (front_revolutions.current?.value ?? 0) > 0;
  const backFanOn = (back_revolutions.current?.value ?? 0) > 0;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Reservoir 1">
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
                  value={frontTargetTemperature}
                  max={80}
                  unit="C"
                  step={0.1}
                  triggerClassName="h-21"
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setFrontTemperature(Math.max(val, minSettableTemperature));
                  }}
                  defaultValue={
                    defaultState?.temperature_states.front.target_temperature
                  }
                />
              </Label>

              {frontHeaterOn && (
                <Badge
                  variant="hot"
                  className="h-21 min-w-28 justify-center self-end px-4 text-base [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {frontFanOn && (
                <Badge
                  variant="cold"
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
                    setFrontFlow(value == "On");
                  }}
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Reservoir 2">
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
                  value={backTargetTemperature}
                  max={80}
                  unit="C"
                  step={0.1}
                  triggerClassName="h-21"
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setBackTemperature(Math.max(val, minSettableTemperature));
                  }}
                  defaultValue={
                    defaultState?.temperature_states.back.target_temperature
                  }
                />
              </Label>

              {backHeaterOn && (
                <Badge
                  variant="hot"
                  className="h-21 min-w-28 justify-center self-end px-4 text-base [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {backFanOn && (
                <Badge
                  variant="cold"
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
                    setBackFlow(value == "On");
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
