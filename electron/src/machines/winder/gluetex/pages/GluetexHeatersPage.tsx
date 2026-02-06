import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { HeatingZone } from "@/machines/extruder/HeatingZone";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { useGluetex } from "../hooks/useGluetex";
import { HeatingMode } from "../state/gluetexNamespace";
import { GluetexErrorBanner } from "../components/GluetexErrorBanner";

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
      </ControlGrid>
    </Page>
  );
}
