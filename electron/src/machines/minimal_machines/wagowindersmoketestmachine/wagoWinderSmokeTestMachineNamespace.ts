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
  target_velocity: z.number(),
  actual_velocity: z.number(),
  target_acceleration: z.number(),
  freq_range_sel: z.number(),
  acc_range_sel: z.number(),
  mode: z.string().nullable(),
  ready: z.boolean(),
  stop2n_ack: z.boolean(),
  start_ack: z.boolean(),
  speed_mode_ack: z.boolean(),
  standstill: z.boolean(),
  on_speed: z.boolean(),
  direction_positive: z.boolean(),
  error: z.boolean(),
  reset: z.boolean(),
  position: z.number(),
  raw_position: z.number(),
  di1: z.boolean(),
  di2: z.boolean(),
  status_byte1: z.number(),
  status_byte2: z.number(),
  status_byte3: z.number(),
  control_byte1: z.number(),
  control_byte2: z.number(),
  control_byte3: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type WagoWinderSmokeTestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWagoWinderSmokeTestMachineNamespaceStore =
  (): StoreApi<WagoWinderSmokeTestMachineNamespaceStore> =>
    create<WagoWinderSmokeTestMachineNamespaceStore>(() => ({
      state: null,
    }));

export function wagoWinderSmokeTestMachineMessageHandler(
  _store: StoreApi<WagoWinderSmokeTestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoWinderSmokeTestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: WagoWinderSmokeTestMachineNamespaceStore,
      ) => WagoWinderSmokeTestMachineNamespaceStore,
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

const useWagoWinderSmokeTestMachineNamespaceImplementation =
  createNamespaceHookImplementation<WagoWinderSmokeTestMachineNamespaceStore>({
    createStore: createWagoWinderSmokeTestMachineNamespaceStore,
    createEventHandler: wagoWinderSmokeTestMachineMessageHandler,
  });

export function useWagoWinderSmokeTestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): WagoWinderSmokeTestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWagoWinderSmokeTestMachineNamespaceImplementation(namespaceId);
}
