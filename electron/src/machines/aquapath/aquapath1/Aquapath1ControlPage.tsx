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
  const rightReservoirTargetTemperature =
    state?.temperature_states?.right.target_temperature ?? 0;
  const leftReservoirTargetTemperature =
    state?.temperature_states?.left.target_temperature ?? 0;
  const minSettableTemperature = state?.ambient_temperature_calibration ?? 22;

  const rightReservoirTargetFlow =
    state?.flow_states.right.should_flow ?? false;
  const leftReservoirTargetFlow = state?.flow_states.left.should_flow ?? false;
  const rightReservoirHeaterOn = right_heating;
  const leftReservoirHeaterOn = left_heating;
  const rightReservoirFanOn = (right_revolutions.current?.value ?? 0) > 0;
  const leftReservoirFanOn = (left_revolutions.current?.value ?? 0) > 0;
  const rightReservoirPumpCooldownActive = right_pump_cooldown_active;
  const leftReservoirPumpCooldownActive = left_pump_cooldown_active;
  const rightReservoirPumpCooldownRemaining = right_pump_cooldown_remaining;
  const leftReservoirPumpCooldownRemaining = left_pump_cooldown_remaining;
  const rightReservoirHeatingStartupWaitActive =
    right_heating_startup_wait_active;
  const leftReservoirHeatingStartupWaitActive =
    left_heating_startup_wait_active;
  const rightReservoirHeatingStartupWaitRemaining =
    right_heating_startup_wait_remaining;
  const leftReservoirHeatingStartupWaitRemaining =
    left_heating_startup_wait_remaining;
  const rightReservoirMaxRevolutions =
    state?.fan_states.right.max_revolutions ?? 100;
  const leftReservoirMaxRevolutions =
    state?.fan_states.left.max_revolutions ?? 100;

  const renderThermalDelay = (value: number) =>
    `Thermal Delay ${Math.max(value, 0).toFixed(1)}s`;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Left Reservoir">
          <div className="grid min-w-0 grid-rows-5 gap-4 overflow-hidden">
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
                  value={leftReservoirTargetTemperature}
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

              {leftReservoirHeaterOn && (
                <Badge
                  variant="default"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-red-500 px-4 text-base text-white [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {leftReservoirFanOn && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-sky-100 px-4 text-base text-sky-800 [&>svg]:size-5"
                >
                  <Icon name="lu:Fan" className="size-5" />
                  Cooling
                </Badge>
              )}

              {(leftReservoirPumpCooldownActive ||
                leftReservoirHeatingStartupWaitActive) && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-40 justify-center self-end border-transparent bg-cyan-100 px-4 text-base text-cyan-900 [&>svg]:size-5"
                >
                  <Icon name="lu:WavesHorizontal" className="size-5" />
                  {renderThermalDelay(
                    leftReservoirPumpCooldownActive
                      ? leftReservoirPumpCooldownRemaining
                      : leftReservoirHeatingStartupWaitRemaining,
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
                    (value / Math.max(leftReservoirMaxRevolutions, 1)) *
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
                  value={leftReservoirTargetFlow ? "On" : "Off"}
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
          <div className="grid min-w-0 grid-rows-5 gap-4 overflow-hidden">
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
                  value={rightReservoirTargetTemperature}
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

              {rightReservoirHeaterOn && (
                <Badge
                  variant="default"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-red-500 px-4 text-base text-white [&>svg]:size-5"
                >
                  <Icon name="lu:Flame" className="size-5" />
                  Heating
                </Badge>
              )}

              {rightReservoirFanOn && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-28 justify-center self-end border-transparent bg-sky-100 px-4 text-base text-sky-800 [&>svg]:size-5"
                >
                  <Icon name="lu:Fan" className="size-5" />
                  Cooling
                </Badge>
              )}

              {(rightReservoirPumpCooldownActive ||
                rightReservoirHeatingStartupWaitActive) && (
                <Badge
                  variant="secondary"
                  className="h-18 min-w-40 justify-center self-end border-transparent bg-cyan-100 px-4 text-base text-cyan-900 [&>svg]:size-5"
                >
                  <Icon name="lu:WavesHorizontal" className="size-5" />
                  {renderThermalDelay(
                    rightReservoirPumpCooldownActive
                      ? rightReservoirPumpCooldownRemaining
                      : rightReservoirHeatingStartupWaitRemaining,
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
                    (value / Math.max(rightReservoirMaxRevolutions, 1)) *
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
                  value={rightReservoirTargetFlow ? "On" : "Off"}
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
