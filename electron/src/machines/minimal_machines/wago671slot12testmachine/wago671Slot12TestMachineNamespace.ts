import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  Event,
  EventHandler,
  NamespaceId,
  ThrottledStoreUpdater,
  createNamespaceHookImplementation,
  eventSchema,
  handleUnhandledEventError,
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";

export const axisStateEventDataSchema = z.object({
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

export const stateEventDataSchema = z.object({
  slot1: axisStateEventDataSchema,
  slot2: axisStateEventDataSchema,
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type AxisStateEvent = z.infer<typeof axisStateEventDataSchema>;
export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type Wago671Slot12TestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago671Slot12TestMachineNamespaceStore =
  (): StoreApi<Wago671Slot12TestMachineNamespaceStore> =>
    create<Wago671Slot12TestMachineNamespaceStore>(() => ({
      state: null,
    }));

export function wago671Slot12TestMachineMessageHandler(
  store: StoreApi<Wago671Slot12TestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago671Slot12TestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago671Slot12TestMachineNamespaceStore,
      ) => Wago671Slot12TestMachineNamespaceStore,
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

const useWago671Slot12TestMachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago671Slot12TestMachineNamespaceStore>({
    createStore: createWago671Slot12TestMachineNamespaceStore,
    createEventHandler: wago671Slot12TestMachineMessageHandler,
  });

export function useWago671Slot12TestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago671Slot12TestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago671Slot12TestMachineNamespaceImplementation(namespaceId);
}
