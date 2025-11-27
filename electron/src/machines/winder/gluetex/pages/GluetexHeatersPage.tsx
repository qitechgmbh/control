import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { HeatingZone } from "@/machines/extruder/HeatingZone";
import { useGluetex } from "../hooks/useGluetex";

export function GluetexHeatersPage() {
  const {
    state,
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
    setHeatingZone1Temperature,
    setHeatingZone2Temperature,
    setHeatingZone3Temperature,
    setHeatingZone4Temperature,
    setHeatingZone5Temperature,
    setHeatingZone6Temperature,
  } = useGluetex();

  return (
    <Page>
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
      </ControlGrid>
    </Page>
  );
}
