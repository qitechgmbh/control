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

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Standby", "On", "Auto", "Interval"]);
export type Mode = z.infer<typeof modeSchema>;

export const stateEventDataSchema = z.object({
  mode: modeSchema,

  interval_time_off: z.number(),
  interval_time_on: z.number(),
});

export const liveValuesEventDataSchema = z.object({});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventDataSchema>;

// ========== Store ==========
export type VacuumNamespaceStore = {
  state: StateEvent | null;
  liveValues: LiveValuesEvent | null;
};

export const createVacuumNamespaceStore =
  (): StoreApi<VacuumNamespaceStore> =>
    create<VacuumNamespaceStore>(() => ({
      state: null,
      liveValues: null,
    }));

// ========== Message Handler ==========
export function vacuumTestMachineMessageHandler(
  store: StoreApi<VacuumNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<VacuumNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: VacuumNamespaceStore,
      ) => VacuumNamespaceStore,
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
const useVacuumNamespaceImplementation =
  createNamespaceHookImplementation<VacuumNamespaceStore>({
    createStore: createVacuumNamespaceStore,
    createEventHandler: vacuumTestMachineMessageHandler,
  });

export function useVacuumNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): VacuumNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useVacuumNamespaceImplementation(namespaceId);
}
