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

export const liveValuesEventDataSchema = z.object({
  status: z.number(),
  alarm: z.number(),
  warning: z.number(),
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
  target_temperature: z.number(),
});

export type LiveValuesEventData = z.infer<typeof liveValuesEventDataSchema>;

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;

export type DrywellNamespaceStore = {
  liveValues: LiveValuesEvent | null;
};

export function drywellMessageHandler(
  store: StoreApi<DrywellNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<DrywellNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;
    try {
      if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        throttledUpdater.updateWith((state) => ({
          ...state,
          liveValues: liveValuesEvent,
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

export const createDrywellNamespaceStore =
  (): StoreApi<DrywellNamespaceStore> =>
    create<DrywellNamespaceStore>(() => ({
      liveValues: null,
    }));

const useDrywellNamespaceImplementation =
  createNamespaceHookImplementation<DrywellNamespaceStore>({
    createStore: createDrywellNamespaceStore,
    createEventHandler: drywellMessageHandler,
  });

export function useDrywellNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): DrywellNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };
  return useDrywellNamespaceImplementation(namespaceId);
}
