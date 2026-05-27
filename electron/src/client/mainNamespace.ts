import { StoreApi, useStore } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  createNamespaceHookImplementation,
  eventSchema,
  Event,
  handleEventValidationError,
  handleUnhandledEventError,
  NamespaceId,
  ThrottledStoreUpdater,
} from "./socketioStore";
import {
  deviceIdentification,
  machineIdentificationUnique,
} from "@/machines/types";
import { useRef } from "react";
import { rustEnum } from "@/lib/types";

export type EthercatDevices = z.infer<typeof ethercatDevicesSchema>;

export const ethercatDevicesSchema = z.object({
  devices: z.array(
    z.object({
      configured_address: z.number().int(),
      name: z.string(),
      vendor_id: z.number().int(),
      product_id: z.number().int(),
      revision: z.number().int(),
      device_identification: deviceIdentification,
    }),
  ),
});

export const ethercatDevicesEventDataSchema = z
  .object({
    Initializing: z.boolean(),
    Done: ethercatDevicesSchema,
    Error: z.string(),
  })
  .check(rustEnum);

export type EthercatDevicesEventData = z.infer<
  typeof ethercatDevicesEventDataSchema
>;

export const ethercatDevicesEventSchema = eventSchema(
  ethercatDevicesEventDataSchema,
);

export type EthercatDevicesEvent = z.infer<typeof ethercatDevicesEventSchema>;

export const ethercatStateEventDataSchema = z.object({
  Preop: z.boolean(),
});

export type EthercatStateEventData = z.infer<typeof ethercatStateEventDataSchema>;

export const ethercatStateEventSchema = eventSchema(ethercatStateEventDataSchema);

export type EthercatStateEvent = z.infer<typeof ethercatStateEventSchema>;

export const machinesEventDataSchema = z.object({
  machines: z.array(
    z.object({
      machine_identification_unique: machineIdentificationUnique,
      error: z.string().nullable(),
    }),
  ),
});

export type MachinesEventData = z.infer<typeof machinesEventDataSchema>;

export const machinesEventSchema = eventSchema(machinesEventDataSchema);

export type MachinesEvent = z.infer<typeof machinesEventSchema>;

export const ethercatInterfaceDiscoveryEventDataSchema = z
  .object({
    Discovering: z.boolean(),
    Done: z.string(),
  })
  .check(rustEnum);

export type EthercatInterfaceDiscoveryEventData = z.infer<
  typeof ethercatInterfaceDiscoveryEventDataSchema
>;

export const ethercatInterfaceDiscoveryEventSchema = eventSchema(
  ethercatInterfaceDiscoveryEventDataSchema,
);

export type EthercatInterfaceDiscoveryEvent = z.infer<
  typeof ethercatInterfaceDiscoveryEventSchema
>;

export const mainNamespaceStoreSchema = z.object({
  ethercatDevices: ethercatDevicesEventSchema.nullable(),
  ethercatState: ethercatStateEventSchema.nullable(),
  machines: machinesEventSchema.nullable(),
  ethercatInterfaceDiscovery: ethercatInterfaceDiscoveryEventSchema.nullable(),
});

export type MainNamespaceStore = z.infer<typeof mainNamespaceStoreSchema>;

export const createMainNamespaceStore = (): StoreApi<MainNamespaceStore> => {
  return create<MainNamespaceStore>()(() => ({
    ethercatDevices: null,
    ethercatState: null,
    machines: null,
    ethercatInterfaceDiscovery: null,
  }));
};

export const eventSchemaMap = {
  EthercatDevicesEvent: ethercatDevicesEventSchema,
  EthercatStateEvent: ethercatStateEventSchema,
  MachinesEvent: machinesEventSchema,
};

export function mainMessageHandler(
  store: StoreApi<MainNamespaceStore>,
  _throttledUpdater: ThrottledStoreUpdater<MainNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    try {
      if (eventName === "EthercatDevicesEvent") {
        store.setState((state) => ({
          ...state,
          ethercatDevices: event,
        }));
      } else if (eventName === "EthercatStateEvent") {
        store.setState((state) => ({
          ...state,
          ethercatState: event,
        }));
      } else if (eventName === "MachinesEvent") {
        const validatedEvent = machinesEventSchema.parse(event);        
        const currentMachinesState = store.getState().machines;
        if (currentMachinesState) {
          // Compare the old data array with the incoming data array if mismatch reload          
          const oldDataStr = JSON.stringify(currentMachinesState.data.machines);
          const newDataStr = JSON.stringify(validatedEvent.data.machines);
          if (oldDataStr !== newDataStr) {
            console.log("Detected a change in the machines state! Reloading...");
            window.location.reload();
            return; 
          }
        }
        store.setState((state) => ({
          ...state,
          machines: validatedEvent,
        }));
      } else if (eventName === "EthercatInterfaceDiscoveryEvent") {
        store.setState((state) => ({
          ...state,
          ethercatInterfaceDiscovery: event,
        }));
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (error) {
      if (error instanceof z.ZodError) {
        handleEventValidationError(error, eventName);
      } else {
        console.error(`Unexpected error processing ${eventName} event:`, error);
        throw error;
      }
    }
  };
}

export const mainNamespaceStore: StoreApi<MainNamespaceStore> =
  createMainNamespaceStore();

export const mainMessageHandlerInstance: EventHandler =
  mainMessageHandler(mainNamespaceStore, new ThrottledStoreUpdater(mainNamespaceStore));

const mainRoomImplementation = createNamespaceHookImplementation({
  createStore: () => mainNamespaceStore,
  createEventHandler: mainMessageHandler,
});

export function useMainNamespace(): MainNamespaceStore;
export function useMainNamespace<T>(selector: (s: MainNamespaceStore) => T): T;
export function useMainNamespace<T>(selector?: (s: MainNamespaceStore) => T) {
  const namespaceId = useRef({ type: "main" } satisfies NamespaceId);
  mainRoomImplementation(namespaceId.current);
  return useStore(mainNamespaceStore, selector ?? ((s) => s as unknown as T));
}
