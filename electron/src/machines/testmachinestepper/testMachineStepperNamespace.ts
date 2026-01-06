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
export type TestMachineStepperNamespaceStore = {
  state: StateEvent | null;
};

export const createTestMachineStepperNamespaceStore =
  (): StoreApi<TestMachineStepperNamespaceStore> =>
    create<TestMachineStepperNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function testMachineMessageHandler(
  store: StoreApi<TestMachineStepperNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<TestMachineStepperNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: TestMachineStepperNamespaceStore) => TestMachineStepperNamespaceStore,
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
const useTestMachineStepperNamespaceImplementation =
  createNamespaceHookImplementation<TestMachineStepperNamespaceStore>({
    createStore: createTestMachineStepperNamespaceStore,
    createEventHandler: testMachineMessageHandler,
  });

export function useTestMachineStepperNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): TestMachineStepperNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useTestMachineStepperNamespaceImplementation(namespaceId);
}
