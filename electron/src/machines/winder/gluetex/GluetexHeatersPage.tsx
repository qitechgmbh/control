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
  // Create mock timeseries with simulated data for each heater
  const heater1Temp = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 150 + Math.sin(i / 5) * 2;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater1Power = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 500 + Math.random() * 50 - 25;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater2Temp = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 160 + Math.sin(i / 5) * 2;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater2Power = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 520 + Math.random() * 50 - 25;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater3Temp = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 155 + Math.sin(i / 5) * 2;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater3Power = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 510 + Math.random() * 50 - 25;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater4Temp = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 165 + Math.sin(i / 5) * 2;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater4Power = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 530 + Math.random() * 50 - 25;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater5Temp = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 158 + Math.sin(i / 5) * 2;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater5Power = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 515 + Math.random() * 50 - 25;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater6Temp = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 162 + Math.sin(i / 5) * 2;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  const heater6Power = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 525 + Math.random() * 50 - 25;
      series = insert(series, { value, timestamp });
    }
    return series;
  }, []);

  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Heater 1"}
          heatingState={undefined}
          heatingTimeSeries={heater1Temp}
          heatingPower={heater1Power}
          onChangeTargetTemp={(temp) => console.log("Set Heater 1 temp:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heater 2"}
          heatingState={undefined}
          heatingTimeSeries={heater2Temp}
          heatingPower={heater2Power}
          onChangeTargetTemp={(temp) => console.log("Set Heater 2 temp:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heater 3"}
          heatingState={undefined}
          heatingTimeSeries={heater3Temp}
          heatingPower={heater3Power}
          onChangeTargetTemp={(temp) => console.log("Set Heater 3 temp:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heater 4"}
          heatingState={undefined}
          heatingTimeSeries={heater4Temp}
          heatingPower={heater4Power}
          onChangeTargetTemp={(temp) => console.log("Set Heater 4 temp:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heater 5"}
          heatingState={undefined}
          heatingTimeSeries={heater5Temp}
          heatingPower={heater5Power}
          onChangeTargetTemp={(temp) => console.log("Set Heater 5 temp:", temp)}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heater 6"}
          heatingState={undefined}
          heatingTimeSeries={heater6Temp}
          heatingPower={heater6Power}
          onChangeTargetTemp={(temp) => console.log("Set Heater 6 temp:", temp)}
          min={0}
          max={300}
        />
      </ControlGrid>
    </Page>
  );
}
