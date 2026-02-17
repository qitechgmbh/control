import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { HeatingZone } from "@/machines/extruder/HeatingZone";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { useGluetex } from "../hooks/useGluetex";
import { HeatingMode } from "../state/gluetexNamespace";
import { GluetexErrorBanner } from "../components/GluetexErrorBanner";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

export function GluetexHeatersPage() {
  const {
    state,
    isLoading,
    isDisabled,
    temperature1,
    temperature2,
    temperature3,
    temperature4,
    temperature5,
    temperature6,
    heater1Power,
    heater2Power,
    heater3Power,
    heater4Power,
    heater5Power,
    heater6Power,
    setHeatingMode,
    setHeatingZone1Temperature,
    setHeatingZone2Temperature,
    setHeatingZone3Temperature,
    setHeatingZone4Temperature,
    setHeatingZone5Temperature,
    setHeatingZone6Temperature,
    setValveEnabled,
    setValveManualOverride,
    setValveOnDistanceMm,
    setValveOffDistanceMm,
  } = useGluetex();

  return (
    <Page>
      <GluetexErrorBanner />
      <ControlGrid>
        <HeatingZone
          title={"Temperature 1"}
          heatingState={state?.heating_states.zone_1}
          heatingTimeSeries={temperature1}
          heatingPower={heater1Power}
          onChangeTargetTemp={setHeatingZone1Temperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 2"}
          heatingState={state?.heating_states.zone_2}
          heatingTimeSeries={temperature2}
          heatingPower={heater2Power}
          onChangeTargetTemp={setHeatingZone2Temperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 3"}
          heatingState={state?.heating_states.zone_3}
          heatingTimeSeries={temperature3}
          heatingPower={heater3Power}
          onChangeTargetTemp={setHeatingZone3Temperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 4"}
          heatingState={state?.heating_states.zone_4}
          heatingTimeSeries={temperature4}
          heatingPower={heater4Power}
          onChangeTargetTemp={setHeatingZone4Temperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 5"}
          heatingState={state?.heating_states.zone_5}
          heatingTimeSeries={temperature5}
          heatingPower={heater5Power}
          onChangeTargetTemp={setHeatingZone5Temperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 6"}
          heatingState={state?.heating_states.zone_6}
          heatingTimeSeries={temperature6}
          heatingPower={heater6Power}
          onChangeTargetTemp={setHeatingZone6Temperature}
          min={0}
          max={300}
        />

        <ControlCard className="bg-red" title="Heating Mode">
          <SelectionGroup<HeatingMode>
            value={state?.heating_state?.heating_mode}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setHeatingMode}
            orientation="horizontal"
            className="grid grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
              },
              Heating: {
                children: "Heating",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
              },
            }}
          />
        </ControlCard>

        <ControlCard className="bg-blue-500" title="Valve Control">
          <div className="flex flex-col gap-4">
            <Label label="Enabled">
              <SelectionGroupBoolean
                value={state?.valve_state?.enabled ?? false}
                disabled={isDisabled}
                onChange={setValveEnabled}
                optionTrue={{ children: "Enabled", icon: "lu:Play" }}
                optionFalse={{ children: "Disabled", icon: "lu:CirclePause" }}
              />
            </Label>

            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Manual Control</span>
              <SelectionGroup
                value={
                  state?.valve_state?.manual_override === null
                    ? "auto"
                    : state?.valve_state?.manual_override
                      ? "on"
                      : "off"
                }
                disabled={isDisabled || !state?.valve_state?.enabled}
                loading={isLoading}
                onChange={(value) => {
                  if (value === "auto") {
                    setValveManualOverride(null);
                  } else {
                    setValveManualOverride(value === "on");
                  }
                }}
                orientation="horizontal"
                className="grid grid-cols-3 gap-1"
                options={{
                  auto: {
                    children: "Auto",
                    isActiveClassName: "bg-blue-600",
                  },
                  off: {
                    children: "Off",
                    isActiveClassName: "bg-gray-600",
                  },
                  on: {
                    children: "On",
                    isActiveClassName: "bg-green-600",
                  },
                }}
              />
            </div>

            <Label label="ON Distance (mm)">
              <EditValue
                title="Valve ON Distance"
                value={state?.valve_state?.on_distance_mm ?? 0}
                min={0}
                max={10000}
                step={1}
                unit="mm"
                renderValue={(value) => value.toFixed(0)}
                onChange={(value) => {
                  if (isDisabled || !state?.valve_state?.enabled) {
                    return;
                  }
                  setValveOnDistanceMm(value);
                }}
              />
            </Label>

            <Label label="OFF Distance (mm)">
              <EditValue
                title="Valve OFF Distance"
                value={state?.valve_state?.off_distance_mm ?? 0}
                min={0}
                max={10000}
                step={1}
                unit="mm"
                renderValue={(value) => value.toFixed(0)}
                onChange={(value) => {
                  if (isDisabled || !state?.valve_state?.enabled) {
                    return;
                  }
                  setValveOffDistanceMm(value);
                }}
              />
            </Label>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
