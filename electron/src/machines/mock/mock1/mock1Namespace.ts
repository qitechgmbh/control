/**
 * @file mock1Namespace.ts
 * @description TypeScript implementation of Mock1 namespace with Zod schema validation.
 */

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
import { TimeSeries, TimeSeriesValue } from "@/lib/timeseries";
import {
  createNamespacePersistentTimeSeries,
  NamespaceSeriesResult,
} from "@/lib/namespacePersistence";
import { serializeNamespaceId } from "../../../client/socketioStore";

// ========== Event Schema Definitions ==========
/**
 * Mode enum for Mock Machine
 */
export const modeSchema = z.enum(["Standby", "Running"]);

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});

/**
 * Live values event schema (time-series data)
 */
export const liveValuesEventDataSchema = z.object({
  amplitude_sum: z.number(),
  amplitude1: z.number(),
  amplitude2: z.number(),
  amplitude3: z.number(),
});

/**
 * State event schema (consolidated state)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  frequency1: z.number(),
  frequency2: z.number(),
  frequency3: z.number(),
  mode_state: modeStateSchema,
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type Mode = z.infer<typeof modeSchema>;
export type ModeState = z.infer<typeof modeStateSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type Mock1NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  sineWaveSum: TimeSeries;
  sineWave1: TimeSeries;
  sineWave2: TimeSeries;
  sineWave3: TimeSeries;
};

// Store-level cache for persistent series
const seriesCache = new Map<
  string,
  NamespaceSeriesResult<{
    sineWaveSum: string;
    sineWave1: string;
    sineWave2: string;
    sineWave3: string;
  }>
>();

/**
 * Factory function to create a new Mock1 namespace store
 * @param namespaceId The namespace identifier for persistence
 * @returns A new Zustand store instance for Mock1 namespace
 */
export const createMock1NamespaceStore = (
  namespaceId: NamespaceId,
): StoreApi<Mock1NamespaceStore> => {
  const cacheKey = serializeNamespaceId(namespaceId);

  if (!seriesCache.has(cacheKey)) {
    const seriesResult = createNamespacePersistentTimeSeries(namespaceId, {
      sineWaveSum: "sineWaveSum",
      sineWave1: "sineWave1",
      sineWave2: "sineWave2",
      sineWave3: "sineWave3",
    });
    seriesCache.set(cacheKey, seriesResult);
  }

  const seriesResult = seriesCache.get(cacheKey)!;

  const store = create<Mock1NamespaceStore>(() => ({
    state: null,
    defaultState: null,
    sineWaveSum: seriesResult.initialState.sineWaveSum,
    sineWave1: seriesResult.initialState.sineWave1,
    sineWave2: seriesResult.initialState.sineWave2,
    sineWave3: seriesResult.initialState.sineWave3,
  }));

  // With lazy loading, onHistoryLoaded fires when historical data is ready
  // Live data starts flowing immediately, then history merges in background
  seriesResult.onHistoryLoaded((historicalSeries) => {
    store.setState(historicalSeries);
  });

  return store;
};

/**
 * Get insert functions for a namespace
 */
function getInsertFunctions(namespaceId: NamespaceId) {
  const cacheKey = serializeNamespaceId(namespaceId);
  const seriesResult = seriesCache.get(cacheKey);
  if (!seriesResult) {
    throw new Error(`Series not initialized for namespace: ${cacheKey}`);
  }
  return seriesResult.insertFns;
}

/**
 * Creates a message handler for Mock1 namespace events with validation and appropriate caching strategies
 * @param namespaceId The namespace identifier
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function mock1MessageHandler(
  namespaceId: NamespaceId,
  store: StoreApi<Mock1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Mock1NamespaceStore>,
): EventHandler {
  const insertFns = getInsertFunctions(namespaceId);

  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Mock1NamespaceStore) => Mock1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      // State events (latest only)
      if (eventName === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);
        console.log("StateEvent", stateEvent);
        updateStore((state) => ({
          ...state,
          state: stateEvent,
          // only set default state if is_default_state is true
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
        }));
      }
      // Live values events (time-series data)
      else if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const wave1Value: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude1 ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        const wave2Value: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude2 ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        const wave3Value: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude3 ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        const waveSumValue: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude_sum ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        updateStore((state) => ({
          ...state,
          sineWaveSum: insertFns.sineWaveSum(state.sineWaveSum, waveSumValue),
          sineWave1: insertFns.sineWave1(state.sineWave1, wave1Value),
          sineWave2: insertFns.sineWave2(state.sineWave2, wave2Value),
          sineWave3: insertFns.sineWave3(state.sineWave3, wave3Value),
        }));
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (error) {
      console.error(`Unexpected error processing ${eventName} event:`, error);
      throw error;
    }
  };
}

/**
 * Hook to use Mock1 namespace with persistent timeseries
 */
export function useMock1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Mock1NamespaceStore {
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  const useImpl = useMemo(() => {
    return createNamespaceHookImplementation<Mock1NamespaceStore>({
      createStore: () => createMock1NamespaceStore(namespaceId),
      createEventHandler: (store, throttledUpdater) =>
        mock1MessageHandler(namespaceId, store, throttledUpdater),
    });
  }, [namespaceId]);

  return useImpl(namespaceId);
}
