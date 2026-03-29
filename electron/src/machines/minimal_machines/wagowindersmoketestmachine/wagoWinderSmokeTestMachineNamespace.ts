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

const axisStateSchema = z.object({
  enabled: z.boolean(),
  target_velocity: z.number(),
  target_acceleration: z.number(),
  freq_range_sel: z.number(),
  acc_range_sel: z.number(),
  mode: z.string().nullable(),
  speed_mode_ack: z.boolean(),
  di1: z.boolean(),
  di2: z.boolean(),
  status_byte1: z.number(),
  status_byte2: z.number(),
  status_byte3: z.number(),
});

export const stateEventDataSchema = z.object({
  axes: z.array(axisStateSchema).length(1),
  digital_output1: z.boolean(),
  digital_output2: z.boolean(),
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
