import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  Event,
  NamespaceId,
  ThrottledStoreUpdater,
  createNamespaceHookImplementation,
  eventSchema,
  handleUnhandledEventError,
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";

export const stateEventDataSchema = z.object({
  enabled: z.boolean(),
  mode: z.string(),
  control_mode: z.string(),
  controller_state: z.string(),
  is_homed: z.boolean(),
  speed_mode_ack: z.boolean(),
  di1: z.boolean(),
  di2: z.boolean(),
  switch_output_on: z.boolean(),
  target_velocity_register: z.number(),
  target_speed_steps_per_second: z.number(),
  actual_velocity_register: z.number(),
  actual_speed_steps_per_second: z.number(),
  actual_speed_mm_per_second: z.number(),
  reference_mode_ack: z.boolean(),
  reference_ok: z.boolean(),
  busy: z.boolean(),
  target_acceleration: z.number(),
  speed_scale: z.number(),
  direction_multiplier: z.number(),
  freq_range_sel: z.number(),
  acc_range_sel: z.number(),
  raw_position_steps: z.number(),
  wrapper_position_steps: z.number(),
  raw_position_mm: z.number(),
  wrapper_position_mm: z.number(),
  controller_position_mm: z.number().nullable(),
  limit_inner_mm: z.number(),
  limit_outer_mm: z.number(),
  manual_speed_mm_per_second: z.number(),
  manual_velocity_register: z.number(),
  control_byte1: z.number(),
  control_byte2: z.number(),
  control_byte3: z.number(),
  status_byte1: z.number(),
  status_byte2: z.number(),
  status_byte3: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type WagoTraverseTestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWagoTraverseTestMachineNamespaceStore =
  (): StoreApi<WagoTraverseTestMachineNamespaceStore> =>
    create<WagoTraverseTestMachineNamespaceStore>(() => ({
      state: null,
    }));

export function wagoTraverseTestMachineMessageHandler(
  _store: StoreApi<WagoTraverseTestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoTraverseTestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: WagoTraverseTestMachineNamespaceStore,
      ) => WagoTraverseTestMachineNamespaceStore,
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

const useWagoTraverseTestMachineNamespaceImplementation =
  createNamespaceHookImplementation<WagoTraverseTestMachineNamespaceStore>({
    createStore: createWagoTraverseTestMachineNamespaceStore,
    createEventHandler: wagoTraverseTestMachineMessageHandler,
  });

export function useWagoTraverseTestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): WagoTraverseTestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWagoTraverseTestMachineNamespaceImplementation(namespaceId);
}
