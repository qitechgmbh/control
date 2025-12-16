import { useMachineMutate } from "@/client/useClient";
import { MiniGraph } from "@/components/graph/MiniGraph";
import { Page } from "@/components/Page";
import { toastError } from "@/components/Toast";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
  TimeSeriesWithInsert,
} from "@/lib/timeseries";
import { analogInputTestMachineSerialRoute } from "@/routes/routes";
import React, { useEffect, useMemo, useState } from "react";
import z from "zod";
import { analogInputTestMachine } from "../properties";
import { MachineIdentificationUnique } from "../types";
import { createNamespaceHookImplementation } from "@/client/socketioStore";
import { StoreApi, create } from "zustand";

const createMachineStore = (): StoreApi<{
  measurementRateHz?: number;
  measurement?: [number, string]; //[0]: measurement value, [1]: measurement timestamp
}> =>
  create<{
    measurementRateHz?: number;
    measurement?: [number, string];
  }>(() => {
    return {};
  });

const useMachineNamespace = createNamespaceHookImplementation({
  createStore: () => createMachineStore(),
  createEventHandler: (store, throttledUpdater) => (event) => {
    const oldState = store.getState();
    const newMeasurementDataHz = event.data["MeasurementDataHz"];
    const newMeasurement: [number, string] = event.data["Measurement"];
    switch (event.name) {
      case "MeasurementRateHz":
        if (
          newMeasurementDataHz &&
          newMeasurementDataHz !== oldState.measurementRateHz
        )
          store.setState({
            ...oldState,
            measurementRateHz: newMeasurementDataHz,
          });
        break;
      case "Measurement":
        if (newMeasurement && newMeasurement !== oldState.measurement)
          store.setState({
            ...oldState,
            measurement: newMeasurement,
          });
        break;
    }
  },
});

export function AnalogInputTestMachineControl(): React.JSX.Element {
  const { serial: serialString } =
    analogInputTestMachineSerialRoute.useParams();
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);
    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );
      return {
        machine_identification: { vendor: 0, machine: 0 },
        serial: 0,
      };
    }
    return {
      machine_identification: analogInputTestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const websocketStore = useMachineNamespace({
    type: "machine",
    machine_identification_unique: machineIdentification,
  });

  const mutateMachine = useMachineMutate(
    z.object({
      measurement_rate_hz: z.number(),
    }),
  );

  const [measurementRate, setMeasurementRate] = useState(1);

  const foo: TimeSeriesWithInsert = createTimeSeries(
    20,
    1000,
    5000,
    60 * 60 * 1000,
  );

  const [seriesData, setSeriesData] = useState<TimeSeries | null>(
    foo.initialTimeSeries,
  );

  useEffect(() => {
    console.log(websocketStore);
    const measurement = websocketStore.measurement;
    if (measurement && seriesData) {
      const dataPoint: TimeSeriesValue = {
        value: measurement[0] * 1000,
        timestamp: Number(measurement[1]),
      };
      setSeriesData(foo.insert(seriesData, dataPoint));
    }
  }, [websocketStore]);

  if (!seriesData) return <div>Initializing Sensor...</div>;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Measurement Control">
          <div
            style={{
              display: "flex",
              flexDirection: "row",
              justifyContent: "space-between",
            }}
          >
            <div>Measurement Rate (Hz)</div>
            <input
              type="number"
              style={{
                border: "2px solid transparent",
                borderRadius: "4px",
                padding: "3px",
              }}
              className={"bg-neutral-100 shadow"}
              value={measurementRate}
              onChange={(v) => {
                const val = Number(v.currentTarget.value);
                mutateMachine.request({
                  machine_identification_unique: machineIdentification,
                  data: { measurement_rate_hz: val },
                });
                setMeasurementRate(val);
              }}
            ></input>
          </div>
        </ControlCard>
        <ControlCard title="Results">
          <div className="flex flex-row">
            <MiniGraph
              newData={seriesData}
              width={500}
              renderValue={(v) => v.toFixed(2)}
            ></MiniGraph>
            <div className="flex flex-col justify-center">
              {seriesData.current?.value.toFixed(2)}mA
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
