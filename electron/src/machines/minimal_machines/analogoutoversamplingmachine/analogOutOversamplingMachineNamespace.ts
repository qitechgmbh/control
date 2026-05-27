import { z } from "zod";
import { create, StoreApi } from "zustand";
import {
  EventHandler,
  eventSchema,
  Event,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
  ThrottledStoreUpdater,
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { useMemo } from "react";
import { createTimeSeries, TimeSeries, TimeSeriesValue } from "@/lib/timeseries";

// ========== Schemas ==========

export const waveformTypeSchema = z.enum(["Sine", "Sawtooth", "Square", "Constant"]);
export type WaveformType = z.infer<typeof waveformTypeSchema>;

export const channelConfigSchema = z.object({
  waveform: waveformTypeSchema,
  frequency_hz: z.number(),
  amplitude: z.number(),
  offset: z.number(),
});
export type ChannelConfig = z.infer<typeof channelConfigSchema>;

export const stateEventDataSchema = z.object({
  channels: z.tuple([channelConfigSchema, channelConfigSchema]),
  oversample_factor: z.number(),
  cycle_time_us: z.number(),
});

export const liveValuesEventDataSchema = z.object({
  ch1_samples: z.array(z.number()),
  ch2_samples: z.array(z.number()),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventDataSchema>;

// ========== Store ==========

export type AnalogOutOversamplingNamespaceStore = {
  state: StateEvent | null;
  // One TimeSeries per channel — tracks the mean voltage of all oversampled
  // slots each cycle, giving a readable "current output level" over time.
  ch1Voltage: TimeSeries;
  ch2Voltage: TimeSeries;
  // Raw latest samples for the bar visualiser
  ch1Samples: number[];
  ch2Samples: number[];
};

const { initialTimeSeries: ch1VoltageInit, insert: addCh1Voltage } = createTimeSeries();
const { initialTimeSeries: ch2VoltageInit, insert: addCh2Voltage } = createTimeSeries();

export const createAnalogOutOversamplingNamespaceStore =
  (): StoreApi<AnalogOutOversamplingNamespaceStore> =>
    create<AnalogOutOversamplingNamespaceStore>(() => ({
      state: null,
      ch1Voltage: ch1VoltageInit,
      ch2Voltage: ch2VoltageInit,
      ch1Samples: [],
      ch2Samples: [],
    }));

// ========== Message Handler ==========

export function analogOutOversamplingMessageHandler(
  store: StoreApi<AnalogOutOversamplingNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<AnalogOutOversamplingNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (s: AnalogOutOversamplingNamespaceStore) => AnalogOutOversamplingNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);
        updateStore((s) => ({ ...s, state: parsed.data }));
      } else if (event.name === "LiveValuesEvent") {
        const parsed = liveValuesEventSchema.parse(event);
        const ch1 = parsed.data.ch1_samples;
        const ch2 = parsed.data.ch2_samples;

        // Mean of all slots → single voltage reading for the time series
        const mean = (arr: number[]) =>
          arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : 0;

        const ch1Val: TimeSeriesValue = { value: mean(ch1) * 10, timestamp: event.ts };
        const ch2Val: TimeSeriesValue = { value: mean(ch2) * 10, timestamp: event.ts };

        updateStore((s) => ({
          ...s,
          ch1Voltage: addCh1Voltage(s.ch1Voltage, ch1Val),
          ch2Voltage: addCh2Voltage(s.ch2Voltage, ch2Val),
          ch1Samples: ch1,
          ch2Samples: ch2,
        }));
      } else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Error processing ${event.name}:`, error);
      throw error;
    }
  };
}

// ========== Namespace Hook ==========

const useAnalogOutOversamplingNamespaceImplementation =
  createNamespaceHookImplementation<AnalogOutOversamplingNamespaceStore>({
    createStore: createAnalogOutOversamplingNamespaceStore,
    createEventHandler: analogOutOversamplingMessageHandler,
  });

export function useAnalogOutOversamplingNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): AnalogOutOversamplingNamespaceStore {
  const namespaceId = useMemo<NamespaceId>(
    () => ({ type: "machine", machine_identification_unique }),
    [machine_identification_unique],
  );
  return useAnalogOutOversamplingNamespaceImplementation(namespaceId);
}
