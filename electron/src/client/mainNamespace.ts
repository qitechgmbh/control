import { StoreApi } from "zustand";
import { create } from "zustand";
import { produce } from "immer";
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
  machineDeviceIdentification,
  machineIdentificationUnique,
} from "@/machines/types";
import { useRef } from "react";
import { rustEnumSchema } from "@/lib/types";

export const ethercatSetupEventDataSchema = rustEnumSchema({
  Initializing: z.boolean(),
  Done: z.object({
    devices: z.array(
      z.object({
        configured_address: z.number().int(),
        name: z.string(),
        vendor_id: z.number().int(),
        product_id: z.number().int(),
        revision: z.number().int(),
        machine_device_identification: machineDeviceIdentification.nullable(),
        subdevice_index: z.number().int(),
      }),
    ),
    machines: z.array(
      z.object({
        machine_identification_unique: machineIdentificationUnique,
        error: z.string().nullable(),
      }),
    ),
  }),
  Error: z.string(),
});

export type EthercatSetupEventData = z.infer<
  typeof ethercatSetupEventDataSchema
>;

export const ethercatSetupEventSchema = eventSchema(
  ethercatSetupEventDataSchema,
);

export type EthercatSetupEvent = z.infer<typeof ethercatSetupEventSchema>;

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

export const mainNamespaceStoreSchema = z.object({
  ethercatSetup: ethercatSetupEventSchema.nullable(),
  ethercatInterfaceDiscovery: ethercatInterfaceDiscoveryEventSchema.nullable(),
});

export type MainNamespaceStore = z.infer<typeof mainNamespaceStoreSchema>;

export const createMainNamespaceStore = (): StoreApi<MainNamespaceStore> => {
  return create<MainNamespaceStore>()(() => ({
    ethercatSetup: null,
    ethercatInterfaceDiscovery: null,
  }));
};

export const eventSchemaMap = {
  EthercatSetupEvent: ethercatSetupEventSchema,
};

export function mainMessageHandler(
  store: StoreApi<MainNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    try {
      // Apply appropriate caching strategy based on event type
      // State events (keep only the latest)
      if (eventName === "EthercatSetupEvent") {
        const validatedEvent = ethercatSetupEventSchema.parse(event);
        console.log("EthercatSetupEvent", validatedEvent);
        store.setState(
          produce((state) => {
            state.ethercatSetup = validatedEvent;
          }),
        );
      } else if (eventName === "EthercatInterfaceDiscoveryEvent") {
        const validatedEvent =
          ethercatInterfaceDiscoveryEventSchema.parse(event);
        console.log("EthercatInterfaceDiscoveryEvent", validatedEvent);
        store.setState(
          produce((state) => {
            state.ethercatInterfaceDiscovery = validatedEvent;
          }),
        );
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

const mainRoomImplementation = createNamespaceHookImplementation({
  createStore: createMainNamespaceStore,
  createEventHandler: mainMessageHandler,
});

export function useMainNamespace(): MainNamespaceStore {
  const namespaceId = useRef({ type: "main" } satisfies NamespaceId);
  return mainRoomImplementation(namespaceId.current);
}
