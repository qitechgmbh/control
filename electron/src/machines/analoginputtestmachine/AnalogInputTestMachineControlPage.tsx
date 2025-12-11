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
} from "@/lib/timeseries";
import { analogInputTestMachineSerialRoute } from "@/routes/routes";
import React, { useEffect, useMemo, useState } from "react";
import z from "zod";
import { analogInputTestMachine, testmachine } from "../properties";
import { MachineIdentificationUnique } from "../types";

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

  const mutateMachine = useMachineMutate(
    z.object({
      measurement_rate_hz: z.number(),
    }),
  );

  const [measurementRate, setMeasurementRate] = useState(1);

  const [seriesData, setSeriesData] = useState<TimeSeries | null>(null);

  useEffect(() => {
    const stopFeed = startDemoDataFeed((newSeriesState) => {
      setSeriesData(newSeriesState);
    });
    return () => stopFeed();
  }, []);

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
            <MiniGraph newData={seriesData} width={500}></MiniGraph>
            <div className="flex flex-col justify-center">
              {seriesData.current?.value.toFixed(4)}A
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}

/**
 * Starts a simulation that pumps random data into a TimeSeries.
 * * @param onUpdate - Callback function that receives the new immutable TimeSeries state every tick.
 * @returns A stop function to clear the interval.
 */
export function startDemoDataFeed(
  onUpdate: (series: TimeSeries) => void,
): () => void {
  // 1. Configuration constants
  const UPDATE_RATE_MS = 50; // How fast the "sensor" pushes data (20Hz)
  const SHORT_WINDOW = 30 * 1000; // 30 seconds retention
  const LONG_WINDOW = 5 * 60 * 1000; // 5 minutes retention

  // 2. Initialize the Series using your factory
  // Short: sample every 100ms (decimates the 50ms input)
  // Long: sample every 1000ms
  const { initialTimeSeries, insert } = createTimeSeries(
    100,
    1000,
    SHORT_WINDOW,
    LONG_WINDOW,
  );

  // 3. Mutable container for our immutable state
  let currentState = initialTimeSeries;

  // Track the last value for "Random Walk" generation
  let lastValue = 50;

  // 4. Start the simulation loop
  const intervalId = setInterval(() => {
    const now = Date.now();

    // Generate a "Random Walk" value (smoother than pure random)
    // New value = Old Value + (random small change)
    const change = (Math.random() - 0.5) * 5;
    let newValue = lastValue + change;

    // Clamp values between 0 and 100 to keep chart pretty
    newValue = Math.max(0, Math.min(100, newValue));
    lastValue = newValue;

    const dataPoint: TimeSeriesValue = {
      timestamp: now,
      value: newValue,
    };

    // 5. Perform the immutable update
    // We overwrite our local 'currentState' with the new draft from Immer
    currentState = insert(currentState, dataPoint);

    // 6. Notify the consumer (e.g., React Component)
    onUpdate(currentState);
  }, UPDATE_RATE_MS);

  // Return a cleanup function
  return () => clearInterval(intervalId);
}
