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

// ========== Event Schema (Muss zu  StateEvent passen) ==========
export const stateEventDataSchema = z.object({
  motor_enabled: z.boolean(),
  motor_velocity: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type TestMotorNamespaceStore = {
  state: StateEvent | null;
};

export const createTestMotorNamespaceStore =
  (): StoreApi<TestMotorNamespaceStore> =>
    create<TestMotorNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function testMotorMessageHandler(
  chstore: StoreApi<TestMotorNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<TestMotorNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: TestMotorNamespaceStore) => TestMotorNamespaceStore,
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

// ========== Namespace Hook ==========
const useTestMotorNamespaceImplementation =
  createNamespaceHookImplementation<TestMotorNamespaceStore>({
    createStore: createTestMotorNamespaceStore,
    createEventHandler: testMotorMessageHandler,
  });

export function useTestMotorNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): TestMotorNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };
  return useTestMotorNamespaceImplementation(namespaceId);
}
