import { useMachineMutate } from "@/client/useClient";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
  TimeSeriesWithInsert,
} from "@/lib/timeseriesHybrid";
import { useEffect, useMemo, useState } from "react";
import z from "zod";
import {
  useAnalogInputTestMachineNamespace,
  StateEvent,
} from "./useAnalogInputTestMachineNamespace";
import { toastError } from "@/components/Toast";
import { MachineIdentificationUnique } from "../types";
import { analogInputTestMachineSerialRoute } from "@/routes/routes";
import { analogInputTestMachine } from "../properties";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

export function useAnalogInputTestMachine() {
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

  const state = useAnalogInputTestMachineNamespace({
    type: "machine",
    machine_identification_unique: machineIdentification,
  });

  const mutateMachine = useMachineMutate(
    z.object({
      measurement_rate_hz: z.number(),
    }),
  );

  const stateOptimistic = useStateOptimistic<StateEvent>();

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState)
      stateOptimistic.setOptimistic(produce(currentState, producer));
    serverRequest?.();
  };

  const updateMeasurementRate = (val: number) => {
    updateStateOptimistically(
      (current) => {
        current.measurementRateHz = val;
      },
      () => {
        mutateMachine.request({
          machine_identification_unique: machineIdentification,
          data: { measurement_rate_hz: val },
        });
      },
    );
  };

  // Create timeseries with IndexedDB backing
  const timeSeries: TimeSeriesWithInsert = useMemo(
    () =>
      createTimeSeries({
        seriesKey: `analogInputTestMachine:${machineIdentification.serial}:measurement`,
        liveBufferSize: 250,
        retentionDurationShort: 5000,
        retentionDurationLong: 60 * 60 * 1000,
      }),
    [machineIdentification.serial],
  );

  const [seriesData, setSeriesData] = useState<TimeSeries>(
    timeSeries.initialTimeSeries,
  );

  useEffect(() => {
    const measurement = state.currentMeasurement;

    if (measurement && seriesData) {
      const dataPoint: TimeSeriesValue = {
        value: measurement[0] * 1000,
        timestamp: Number(measurement[1]),
      };
      setSeriesData(timeSeries.insert(seriesData, dataPoint));
    }
  }, [state.currentMeasurement, timeSeries]);

  return {
    seriesData,
    state,
    updateMeasurementRate,
  };
}
