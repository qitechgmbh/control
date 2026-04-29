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

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Standby", "Hold", "Turn"]);
export type Mode = z.infer<typeof modeSchema>;

/**
 * Frequency Prescaler
 */
export const frequencySchema = z.enum(["Default", "Low", "Mid", "High"]);
export type Frequency = z.infer<typeof frequencySchema>;

/**
 * Acceleration Factor
 */
export const accelerationSchema = z.enum(["Default", "Low", "Mid", "High"]);
export type AccelerationFactor = z.infer<typeof accelerationSchema>;

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});

/**
 * Frequency Prescaler Schema
 */
export const frequencyStateSchema = z.object({
  frequency: frequencySchema,
});

/**
 * Accleration Factor Schema
 */
export const accelerationStateSchema = z.object({
  factor: accelerationSchema,
});

// ========== Event Schema ==========

export const stateEventDataSchema = z.object({
  target_speed: z.number(),
  mode_state: modeStateSchema,
  frequency_state: frequencyStateSchema,
  acceleration_state: accelerationStateSchema,
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
export function testMachineStepperMessageHandler(
  store: StoreApi<TestMachineStepperNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<TestMachineStepperNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: TestMachineStepperNamespaceStore,
      ) => TestMachineStepperNamespaceStore,
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
    createEventHandler: testMachineStepperMessageHandler,
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
