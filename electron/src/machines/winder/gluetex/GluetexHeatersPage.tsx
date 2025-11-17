import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { HeatingZone } from "@/machines/extruder/HeatingZone";
import { useGluetex } from "./useGluetex";

export function GluetexHeatersPage() {
  const {
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
  } = useGluetex();

  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Temperature 1"}
          heatingState={undefined}
          heatingTimeSeries={temperature1}
          heatingPower={heater1Power}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 1:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 2"}
          heatingState={undefined}
          heatingTimeSeries={temperature2}
          heatingPower={heater2Power}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 2:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 3"}
          heatingState={undefined}
          heatingTimeSeries={temperature3}
          heatingPower={heater3Power}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 3:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 4"}
          heatingState={undefined}
          heatingTimeSeries={temperature4}
          heatingPower={heater4Power}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 4:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 5"}
          heatingState={undefined}
          heatingTimeSeries={temperature5}
          heatingPower={heater5Power}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 5:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 6"}
          heatingState={undefined}
          heatingTimeSeries={temperature6}
          heatingPower={heater6Power}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 6:", temp)}
          min={0}
          max={300}
        />
      </ControlGrid>
    </Page>
  );
}
