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

// ========== Event Schema ==========

export const stateEventDataSchema = z.object({
  outputs: z.array(z.boolean()).length(8),
  stepper_speed_mm_s: z.number(),
  stepper_enabled: z.boolean(),
});

export const liveValuesEventDataSchema = z.object({
  stepper_position: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventDataSchema>;

// ========== Store ==========
export type BottleSorterNamespaceStore = {
  state: StateEvent | null;
  liveValues: LiveValuesEvent | null;
};

export const createBottleSorterNamespaceStore =
  (): StoreApi<BottleSorterNamespaceStore> =>
    create<BottleSorterNamespaceStore>(() => ({
      state: null,
      liveValues: null,
    }));

// ========== Message Handler ==========
export function bottleSorterMessageHandler(
  store: StoreApi<BottleSorterNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<BottleSorterNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: BottleSorterNamespaceStore,
      ) => BottleSorterNamespaceStore,
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
const useBottleSorterNamespaceImplementation =
  createNamespaceHookImplementation<BottleSorterNamespaceStore>({
    createStore: createBottleSorterNamespaceStore,
    createEventHandler: bottleSorterMessageHandler,
  });

export function useBottleSorterNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): BottleSorterNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useBottleSorterNamespaceImplementation(namespaceId);
}
