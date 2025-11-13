import { StoreApi } from "zustand";
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
} from "./socketioStore";
import {
  deviceIdentification,
  machineIdentificationUnique,
} from "@/machines/types";
import { useRef } from "react";
import { rustEnumSchema } from "@/lib/types";

// Update the EthercatDevicesEventData schema
export const ethercatDevicesEventDataSchema = rustEnumSchema({
  Initializing: z.boolean(),
  Done: z.object({
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
  }),
  Error: z.string(),
});

export type EthercatDevicesEventData = z.infer<
  typeof ethercatDevicesEventDataSchema
>;

export const ethercatDevicesEventSchema = eventSchema(
  ethercatDevicesEventDataSchema,
);

export type EthercatDevicesEvent = z.infer<typeof ethercatDevicesEventSchema>;

// Create a new schema for MachinesEvent
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

// Keep the EthercatInterfaceDiscovery event
export const ethercatInterfaceDiscoveryEventDataSchema = rustEnumSchema({
  Discovering: z.boolean(),
  Done: z.string(),
});

export type EthercatInterfaceDiscoveryEventData = z.infer<
  typeof ethercatInterfaceDiscoveryEventDataSchema
>;

export const ethercatInterfaceDiscoveryEventSchema = eventSchema(
  ethercatInterfaceDiscoveryEventDataSchema,
);

export type EthercatInterfaceDiscoveryEvent = z.infer<
  typeof ethercatInterfaceDiscoveryEventSchema
>;

// Update the main namespace store schema
export const mainNamespaceStoreSchema = z.object({
  ethercatDevices: ethercatDevicesEventSchema.nullable(),
  machines: machinesEventSchema.nullable(),
  ethercatInterfaceDiscovery: ethercatInterfaceDiscoveryEventSchema.nullable(),
});

export type MainNamespaceStore = z.infer<typeof mainNamespaceStoreSchema>;

export const createMainNamespaceStore = (): StoreApi<MainNamespaceStore> => {
  return create<MainNamespaceStore>()(() => ({
    ethercatDevices: null,
    machines: null,
    ethercatInterfaceDiscovery: null,
  }));
};

export const eventSchemaMap = {
  EthercatDevicesEvent: ethercatDevicesEventSchema,
  MachinesEvent: machinesEventSchema,
};

export function mainMessageHandler(
  store: StoreApi<MainNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    try {
      // Apply appropriate caching strategy based on event type
      if (eventName === "EthercatDevicesEvent") {
        const validatedEvent = ethercatDevicesEventSchema.parse(event);
        console.log("EthercatDevicesEvent", validatedEvent);
        store.setState((state) => ({
          ...state,
          ethercatDevices: validatedEvent,
        }));
      } else if (eventName === "MachinesEvent") {
        const validatedEvent = machinesEventSchema.parse(event);
        console.log("MachinesEvent", validatedEvent);
        store.setState((state) => ({
          ...state,
          machines: validatedEvent,
        }));
      } else if (eventName === "EthercatInterfaceDiscoveryEvent") {
        const validatedEvent =
          ethercatInterfaceDiscoveryEventSchema.parse(event);
        console.log("EthercatInterfaceDiscoveryEvent", validatedEvent);
        store.setState((state) => ({
          ...state,
          ethercatInterfaceDiscovery: validatedEvent,
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

// Create the store instance once and export it
export const mainNamespaceStore: StoreApi<MainNamespaceStore> =
  createMainNamespaceStore();

// Message handler stays the same
export const mainMessageHandlerInstance: EventHandler =
  mainMessageHandler(mainNamespaceStore);

// Hook that uses the exported store
const mainRoomImplementation = createNamespaceHookImplementation({
  createStore: () => mainNamespaceStore, // use the exported store
  createEventHandler: mainMessageHandler,
});

export function useMainNamespace(): MainNamespaceStore {
  const namespaceId = useRef({ type: "main" } satisfies NamespaceId);
  return mainRoomImplementation(namespaceId.current);
}
