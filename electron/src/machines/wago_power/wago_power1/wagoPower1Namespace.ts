import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
  ThrottledStoreUpdater,
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { useMemo } from "react";
import { createTimeSeries, TimeSeries, TimeSeriesValue } from "@/lib/timeseries";

export const liveValuesEventDataSchema = z.object({
  voltage: z.number(),
  current: z.number(),
});

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);

export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;

export type WagoPower1NamespaceStore = {
    current: TimeSeries,
    voltage: TimeSeries,
};

const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;
const { initialTimeSeries: voltage, insert: addVoltage } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);
const { initialTimeSeries: current, insert: addCurrent } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

const createWagoPower1NamespaceStore =
  (): StoreApi<WagoPower1NamespaceStore> =>
    create<WagoPower1NamespaceStore>(() => {
      return {
        voltage,
        current,
      };
    });

function wagoPower1MessageHandler(
  store: StoreApi<WagoPower1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoPower1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: WagoPower1NamespaceStore) => WagoPower1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      if (eventName === "LiveValuesEvent") {
          const liveValues = liveValuesEventSchema.parse(event);
          const { voltage, current } = liveValues.data;
          const voltageValue: TimeSeriesValue = {
              value: voltage,
              timestamp: liveValues.ts,
          };
          const currentValue: TimeSeriesValue = {
              value: current,
              timestamp: liveValues.ts,
          };
          updateStore((store: WagoPower1NamespaceStore) => ({
              ...store,
              voltage: addVoltage(store.voltage, voltageValue),
              current: addCurrent(store.current, currentValue),
          }));
      } else {
          handleUnhandledEventError(eventName);
      }
    } catch (e) {
      console.error(e);
    }
  };
}

const useWagoPower1NamespaceImplementation =
  createNamespaceHookImplementation<WagoPower1NamespaceStore>({
    createStore: createWagoPower1NamespaceStore,
    createEventHandler: wagoPower1MessageHandler,
  });

export function useWagoPower1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): WagoPower1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useWagoPower1NamespaceImplementation(namespaceId);
}
