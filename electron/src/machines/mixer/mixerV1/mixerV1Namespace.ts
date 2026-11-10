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
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import { useMemo } from "react";

const hopperStateSchema = z.object({
  enabled: z.boolean(),
  ready: z.boolean(),
  error: z.boolean(),
  target_rpm: z.number(),
  forward: z.boolean(),
  dosing_percent: z.number(),
  calibration_steps_per_kgh: z.number(),
});

export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  mixing_motor_state: z.object({
    on: z.boolean(),
  }),
  hopper_a_state: hopperStateSchema,
  hopper_b_state: hopperStateSchema,
  extruder_kg_per_rpm: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;

export const liveValuesEventDataSchema = z.object({
  hopper_a_rpm: z.number(),
  hopper_b_rpm: z.number(),
});

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);

const { initialTimeSeries: hopperARpm, insert: addHopperARpm } =
  createTimeSeries();
const { initialTimeSeries: hopperBRpm, insert: addHopperBRpm } =
  createTimeSeries();

export type MixerV1NamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;
  hopperARpm: TimeSeries;
  hopperBRpm: TimeSeries;
};

export const createMixerV1NamespaceStore =
  (): StoreApi<MixerV1NamespaceStore> =>
    create<MixerV1NamespaceStore>(() => ({
      state: null,
      defaultState: null,
      hopperARpm,
      hopperBRpm,
    }));

export function mixerV1MessageHandler(
  store: StoreApi<MixerV1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<MixerV1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: MixerV1NamespaceStore) => MixerV1NamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);

        updateStore((state) => ({
          ...state,
          state: parsed.data,
          defaultState: parsed.data.is_default_state
            ? parsed.data
            : state.defaultState,
        }));
      } else if (event.name === "LiveValuesEvent") {
        const parsed = liveValuesEventSchema.parse(event);
        const timestamp = event.ts;

        updateStore((state) => ({
          ...state,
          hopperARpm: addHopperARpm(state.hopperARpm, {
            value: parsed.data.hopper_a_rpm,
            timestamp,
          }),
          hopperBRpm: addHopperBRpm(state.hopperBRpm, {
            value: parsed.data.hopper_b_rpm,
            timestamp,
          }),
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

const useMixerV1NamespaceImplementation =
  createNamespaceHookImplementation<MixerV1NamespaceStore>({
    createStore: createMixerV1NamespaceStore,
    createEventHandler: mixerV1MessageHandler,
  });

export function useMixerV1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): MixerV1NamespaceStore {
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  return useMixerV1NamespaceImplementation(namespaceId);
}
