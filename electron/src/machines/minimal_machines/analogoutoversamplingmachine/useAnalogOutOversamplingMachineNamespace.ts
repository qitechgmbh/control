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

// ========== Event Schema ==========

export const waveformTypeSchema = z.enum([
  "Sine",
  "Sawtooth",
  "Square",
  "Constant",
]);
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
  liveValues: LiveValuesEvent | null;
};

export const createAnalogOutOversamplingNamespaceStore =
  (): StoreApi<AnalogOutOversamplingNamespaceStore> =>
    create<AnalogOutOversamplingNamespaceStore>(() => ({
      state: null,
      liveValues: null,
    }));

// ========== Message Handler ==========

export function analogOutOversamplingMessageHandler(
  store: StoreApi<AnalogOutOversamplingNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<AnalogOutOversamplingNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: AnalogOutOversamplingNamespaceStore,
      ) => AnalogOutOversamplingNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);
        updateStore((current) => ({ ...current, state: parsed.data }));
      } else if (event.name === "LiveValuesEvent") {
        const parsed = liveValuesEventSchema.parse(event);
        updateStore((current) => ({ ...current, liveValues: parsed.data }));
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
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useAnalogOutOversamplingNamespaceImplementation(namespaceId);
}
