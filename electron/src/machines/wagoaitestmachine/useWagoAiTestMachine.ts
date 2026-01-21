import { useMachineMutate } from "@/client/useClient";
import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
  TimeSeriesWithInsert,
} from "@/lib/timeseries";
import { MachineIdentificationUnique } from "@/machines/types";
import { wagoAiTestMachineSerialRoute } from "@/routes/routes";
import { produce } from "immer";
import { useEffect, useMemo, useState } from "react";
import z from "zod";
import { wagoAiTestMachine } from "../properties";
import {
  StateEvent,
  useWagoAiTestMachineNamespace,
} from "./useWagoAiTestMachineNamespace";

export type AnalogInputTimeSeries = {
  ai1: TimeSeries;
  ai2: TimeSeries;
  ai3: TimeSeries;
  ai4: TimeSeries;
};

export function useWagoAiTestMachine() {
  const { serial: serialString } = wagoAiTestMachineSerialRoute.useParams();
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
      machine_identification: wagoAiTestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const state = useWagoAiTestMachineNamespace({
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

  // Create time series for each analog input
  const timeSeriesInstances: {
    ai1: TimeSeriesWithInsert;
    ai2: TimeSeriesWithInsert;
    ai3: TimeSeriesWithInsert;
    ai4: TimeSeriesWithInsert;
  } = {
    ai1: createTimeSeries({
      sampleIntervalShort: 20,
      sampleIntervalLong: 1000,
      retentionDurationShort: 5000,
      retentionDurationLong: 60 * 60 * 1000,
    }),
    ai2: createTimeSeries({
      sampleIntervalShort: 20,
      sampleIntervalLong: 1000,
      retentionDurationShort: 5000,
      retentionDurationLong: 60 * 60 * 1000,
    }),
    ai3: createTimeSeries({
      sampleIntervalShort: 20,
      sampleIntervalLong: 1000,
      retentionDurationShort: 5000,
      retentionDurationLong: 60 * 60 * 1000,
    }),
    ai4: createTimeSeries({
      sampleIntervalShort: 20,
      sampleIntervalLong: 1000,
      retentionDurationShort: 5000,
      retentionDurationLong: 60 * 60 * 1000,
    }),
  };

  const [seriesData, setSeriesData] = useState<AnalogInputTimeSeries | null>({
    ai1: timeSeriesInstances.ai1.initialTimeSeries,
    ai2: timeSeriesInstances.ai2.initialTimeSeries,
    ai3: timeSeriesInstances.ai3.initialTimeSeries,
    ai4: timeSeriesInstances.ai4.initialTimeSeries,
  });

  useEffect(() => {
    const analogInputs = state.analogInputs;

    if (analogInputs && seriesData) {
      const [ai1, ai2, ai3, ai4, timestampStr] = analogInputs;
      const timestamp = Number(timestampStr);

      const dataPoint1: TimeSeriesValue = {
        value: ai1 * 1000, // Convert to mA
        timestamp,
      };
      const dataPoint2: TimeSeriesValue = {
        value: ai2 * 1000,
        timestamp,
      };
      const dataPoint3: TimeSeriesValue = {
        value: ai3 * 1000,
        timestamp,
      };
      const dataPoint4: TimeSeriesValue = {
        value: ai4 * 1000,
        timestamp,
      };

      setSeriesData({
        ai1: timeSeriesInstances.ai1.insert(seriesData.ai1, dataPoint1),
        ai2: timeSeriesInstances.ai2.insert(seriesData.ai2, dataPoint2),
        ai3: timeSeriesInstances.ai3.insert(seriesData.ai3, dataPoint3),
        ai4: timeSeriesInstances.ai4.insert(seriesData.ai4, dataPoint4),
      });
    }
  }, [state]);

  return {
    seriesData,
    state,
    updateMeasurementRate,
  };
}
