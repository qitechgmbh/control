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
import { roundToDecimals } from "@/lib/decimal";

export function Aquapath1ControlPage() {
  const {
    state,
    defaultState,
    left_flow,
    right_flow,
    left_temperature,
    right_temperature,
    left_power,
    right_power,
    combinedPower,
    totalEnergyKWh,
    left_heating,
    right_heating,
    left_revolutions,
    right_revolutions,
    left_pump_cooldown_active,
    right_pump_cooldown_active,
    left_pump_cooldown_remaining,
    right_pump_cooldown_remaining,
    left_heating_startup_wait_active,
    right_heating_startup_wait_active,
    left_heating_startup_wait_remaining,
    right_heating_startup_wait_remaining,
    setAquapathMode,
    setLeftTemperature,
    setRightTemperature,
    setLeftFlow,
    setRightFlow,
  } = useAquapath1();
  const reservoir1TargetTemperature =
    state?.temperature_states?.right.target_temperature ?? 0;
  const reservoir2TargetTemperature =
    state?.temperature_states?.left.target_temperature ?? 0;
  const minSettableTemperature = state?.ambient_temperature_calibration ?? 22;

  const reservoir1TargetFlow = state?.flow_states.right.should_flow ?? false;
  const reservoir2TargetFlow = state?.flow_states.left.should_flow ?? false;
  const reservoir1HeaterOn = right_heating;
  const reservoir2HeaterOn = left_heating;
  const reservoir1FanOn = (right_revolutions.current?.value ?? 0) > 0;
  const reservoir2FanOn = (left_revolutions.current?.value ?? 0) > 0;
  const reservoir1PumpCooldownActive = right_pump_cooldown_active;
  const reservoir2PumpCooldownActive = left_pump_cooldown_active;
  const reservoir1PumpCooldownRemaining = right_pump_cooldown_remaining;
  const reservoir2PumpCooldownRemaining = left_pump_cooldown_remaining;
  const reservoir1HeatingStartupWaitActive = right_heating_startup_wait_active;
  const reservoir2HeatingStartupWaitActive = left_heating_startup_wait_active;
  const reservoir1HeatingStartupWaitRemaining =
    right_heating_startup_wait_remaining;
  const reservoir2HeatingStartupWaitRemaining =
    left_heating_startup_wait_remaining;
  const reservoir1MaxRevolutions =
    state?.fan_states.right.max_revolutions ?? 100;
  const reservoir2MaxRevolutions =
    state?.fan_states.left.max_revolutions ?? 100;

  const renderThermalDelay = (value: number) =>
    `Thermal Delay ${Math.max(value, 0).toFixed(1)}s`;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Left Reservoir">
          <div className="grid grid-rows-5 gap-4">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Flow"
                unit="l/min"
                timeseries={left_flow}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Temperature"
                unit="C"
                timeseries={left_temperature}
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
                    setLeftTemperature(Math.max(val, minSettableTemperature));
                  }}
                  defaultValue={
                    defaultState?.temperature_states.left.target_temperature
                  }
                />
              </Label>

              {reservoir2HeaterOn && (
                <Badge
                  variant="default"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-red-500 px-4 text-base text-white [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {reservoir2FanOn && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-sky-100 px-4 text-base text-sky-800 [&>svg]:size-5"
                >
                  <Icon name="lu:Fan" className="size-5" />
                  Cooling
                </Badge>
              )}

              {(reservoir2PumpCooldownActive ||
                reservoir2HeatingStartupWaitActive) && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-40 justify-center self-end border-transparent bg-cyan-100 px-4 text-base text-cyan-900 [&>svg]:size-5"
                >
                  <Icon name="lu:WavesHorizontal" className="size-5" />
                  {renderThermalDelay(
                    reservoir2PumpCooldownActive
                      ? reservoir2PumpCooldownRemaining
                      : reservoir2HeatingStartupWaitRemaining,
                  )}
                </Badge>
              )}
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolution Speed"
                unit="%"
                timeseries={left_revolutions}
                renderValue={(value) =>
                  (
                    (value / Math.max(reservoir2MaxRevolutions, 1)) *
                    100
                  ).toFixed(1)
                }
              />
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Heating Wattage"
                unit="W"
                timeseries={left_power}
                renderValue={(value) => roundToDecimals(value, 0)}
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
                    setLeftFlow(value == "On");
                  }}
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Right Reservoir">
          <div className="grid grid-rows-5 gap-4">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Flow"
                unit="l/min"
                timeseries={right_flow}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Temperature"
                unit="C"
                timeseries={right_temperature}
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
                    setRightTemperature(Math.max(val, minSettableTemperature));
                  }}
                  defaultValue={
                    defaultState?.temperature_states.right.target_temperature
                  }
                />
              </Label>

              {reservoir1HeaterOn && (
                <Badge
                  variant="default"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-red-500 px-4 text-base text-white [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {reservoir1FanOn && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-sky-100 px-4 text-base text-sky-800 [&>svg]:size-5"
                >
                  <Icon name="lu:Fan" className="size-5" />
                  Cooling
                </Badge>
              )}

              {(reservoir1PumpCooldownActive ||
                reservoir1HeatingStartupWaitActive) && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-40 justify-center self-end border-transparent bg-cyan-100 px-4 text-base text-cyan-900 [&>svg]:size-5"
                >
                  <Icon name="lu:WavesHorizontal" className="size-5" />
                  {renderThermalDelay(
                    reservoir1PumpCooldownActive
                      ? reservoir1PumpCooldownRemaining
                      : reservoir1HeatingStartupWaitRemaining,
                  )}
                </Badge>
              )}
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolution Speed"
                unit="%"
                timeseries={right_revolutions}
                renderValue={(value) =>
                  (
                    (value / Math.max(reservoir1MaxRevolutions, 1)) *
                    100
                  ).toFixed(1)
                }
              />
            </div>

            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Heating Wattage"
                unit="W"
                timeseries={right_power}
                renderValue={(value) => roundToDecimals(value, 0)}
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
                    setRightFlow(value == "On");
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

        <ControlCard className="bg-blue" title="Heating Power Consumption">
          <TimeSeriesValueNumeric
            label="Total Power"
            unit="W"
            renderValue={(value) => roundToDecimals(value, 0)}
            timeseries={combinedPower}
          />
          <TimeSeriesValueNumeric
            label="Total Energy Consumption"
            unit="kWh"
            renderValue={(value) => roundToDecimals(value, 3)}
            timeseries={totalEnergyKWh}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
