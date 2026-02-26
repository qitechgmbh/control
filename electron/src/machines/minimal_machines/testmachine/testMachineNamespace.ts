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
  led_on: z.array(z.boolean()).length(4),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type TestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createTestMachineNamespaceStore =
  (): StoreApi<TestMachineNamespaceStore> =>
    create<TestMachineNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function testMachineMessageHandler(
  store: StoreApi<TestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<TestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: TestMachineNamespaceStore) => TestMachineNamespaceStore,
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
const useTestMachineNamespaceImplementation =
  createNamespaceHookImplementation<TestMachineNamespaceStore>({
    createStore: createTestMachineNamespaceStore,
    createEventHandler: testMachineMessageHandler,
  });

export function useTestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): TestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useTestMachineNamespaceImplementation(namespaceId);
}
