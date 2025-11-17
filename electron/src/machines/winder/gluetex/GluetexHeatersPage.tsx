import { Page } from "@/components/Page";
import React, { useMemo } from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { HeatingZone } from "@/machines/extruder/HeatingZone";
import { useGluetex } from "./useGluetex";
import { createTimeSeries } from "@/lib/timeseries";

const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

export function GluetexHeatersPage() {
  const {
    temperature1,
    temperature2,
    temperature3,
    temperature4,
    temperature5,
    temperature6,
  } = useGluetex();

  // Create empty time series for power (not measured yet)
  // TODO: Add power measurement if needed in the future
  const emptyPowerTimeSeries = useMemo(() => {
    const { initialTimeSeries } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    return initialTimeSeries;
  }, []);

  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Temperature 1"}
          heatingState={undefined}
          heatingTimeSeries={temperature1}
          heatingPower={emptyPowerTimeSeries}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 1:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 2"}
          heatingState={undefined}
          heatingTimeSeries={temperature2}
          heatingPower={emptyPowerTimeSeries}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 2:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 3"}
          heatingState={undefined}
          heatingTimeSeries={temperature3}
          heatingPower={emptyPowerTimeSeries}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 3:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 4"}
          heatingState={undefined}
          heatingTimeSeries={temperature4}
          heatingPower={emptyPowerTimeSeries}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 4:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 5"}
          heatingState={undefined}
          heatingTimeSeries={temperature5}
          heatingPower={emptyPowerTimeSeries}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 5:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Temperature 6"}
          heatingState={undefined}
          heatingTimeSeries={temperature6}
          heatingPower={emptyPowerTimeSeries}
          onChangeTargetTemp={(temp) => console.log("Set Temperature 6:", temp)}
          min={0}
          max={300}
        />
      </ControlGrid>
    </Page>
  );
}
