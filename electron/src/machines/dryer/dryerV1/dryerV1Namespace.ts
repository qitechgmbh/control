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
import { useMemo } from "react";

const scheduleDaySchema = z.object({
  start_minutes: z.number(),
  stop_minutes: z.number(),
});

export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),

  status: z.number(),
  alarm: z.number(),
  warning: z.number(),
  is_smart: z.boolean(),

  target_temperature: z.number(),
  air_volume: z.number(),
  drying_timer_minutes: z.number(),

  schedule: z.array(scheduleDaySchema).length(7),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;
export type ScheduleDay = z.infer<typeof scheduleDaySchema>;

export const liveValuesEventDataSchema = z.object({
  temp_process: z.number(),
  temp_safety: z.number(),
  temp_regen_in: z.number(),
  temp_regen_out: z.number(),
  temp_fan_inlet: z.number(),
  temp_return_air: z.number(),
  temp_dew_point: z.number(),
  pwm_fan1: z.number(),
  pwm_fan2: z.number(),
  power_process: z.number(),
  power_regen: z.number(),
  remaining_seconds: z.number().nullable(),
});

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export type LiveValues = z.infer<typeof liveValuesEventDataSchema>;

export type DryerV1NamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;
  liveValues: LiveValues | null;
};

export const createDryerV1NamespaceStore =
  (): StoreApi<DryerV1NamespaceStore> =>
    create<DryerV1NamespaceStore>(() => ({
      state: null,
      defaultState: null,
      liveValues: null,
    }));

export function dryerV1MessageHandler(
  store: StoreApi<DryerV1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<DryerV1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: DryerV1NamespaceStore) => DryerV1NamespaceStore,
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

        updateStore((state) => ({
          ...state,
          liveValues: parsed.data,
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

const useDryerV1NamespaceImplementation =
  createNamespaceHookImplementation<DryerV1NamespaceStore>({
    createStore: createDryerV1NamespaceStore,
    createEventHandler: dryerV1MessageHandler,
  });

export function useDryerV1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): DryerV1NamespaceStore {
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  return useDryerV1NamespaceImplementation(namespaceId);
}
