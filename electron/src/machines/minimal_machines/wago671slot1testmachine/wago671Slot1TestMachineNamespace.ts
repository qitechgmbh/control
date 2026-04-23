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

export const stateEventDataSchema = z.object({
  enabled: z.boolean(),
  target_speed: z.number(),
  target_velocity: z.number(),
  target_speed_steps_per_second: z.number(),
  actual_velocity: z.number(),
  actual_speed_steps_per_second: z.number(),
  acceleration: z.number(),
  freq: z.number(),
  acc_freq: z.number(),
  raw_position: z.number(),
  control_byte1: z.number(),
  control_byte2: z.number(),
  control_byte3: z.number(),
  status_byte1: z.number(),
  status_byte2: z.number(),
  status_byte3: z.number(),
  speed_mode_ack: z.boolean(),
  start_ack: z.boolean(),
  di1: z.boolean(),
  di2: z.boolean(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type Wago671Slot1TestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago671Slot1TestMachineNamespaceStore =
  (): StoreApi<Wago671Slot1TestMachineNamespaceStore> =>
    create<Wago671Slot1TestMachineNamespaceStore>(() => ({
      state: null,
    }));

export function wago671Slot1TestMachineMessageHandler(
  store: StoreApi<Wago671Slot1TestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago671Slot1TestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago671Slot1TestMachineNamespaceStore,
      ) => Wago671Slot1TestMachineNamespaceStore,
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

const useWago671Slot1TestMachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago671Slot1TestMachineNamespaceStore>({
    createStore: createWago671Slot1TestMachineNamespaceStore,
    createEventHandler: wago671Slot1TestMachineMessageHandler,
  });

export function useWago671Slot1TestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago671Slot1TestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago671Slot1TestMachineNamespaceImplementation(namespaceId);
}
