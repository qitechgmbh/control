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

// Mirrors the Rust `StateEvent` in machines/src/minimal_machines/wago_750_460_machine/api.rs
//
//   pub struct StateEvent {
//       pub temperatures: [Option<f64>; 4],   // °C, None on sensor error
//       pub errors:       [bool; 4],           // wire-break / overrange per channel
//   }
export const stateEventDataSchema = z.object({
  temperatures: z.array(z.number().nullable()).length(4),
  errors: z.array(z.boolean()).length(4),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type Wago750460MachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago750460MachineNamespaceStore =
  (): StoreApi<Wago750460MachineNamespaceStore> =>
    create<Wago750460MachineNamespaceStore>(() => ({
      state: null,
    }));

export function wago750460MachineMessageHandler(
  store: StoreApi<Wago750460MachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago750460MachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago750460MachineNamespaceStore,
      ) => Wago750460MachineNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);
        updateStore(() => ({ state: parsed.data }));
      } else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Error processing ${event.name}:`, error);
      throw error;
    }
  };
}

const useWago750460MachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago750460MachineNamespaceStore>({
    createStore: createWago750460MachineNamespaceStore,
    createEventHandler: wago750460MachineMessageHandler,
  });

export function useWago750460MachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago750460MachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago750460MachineNamespaceImplementation(namespaceId);
}
