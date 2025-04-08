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
  handleUnknownEventError,
  handleUnhandledEventError,
  NamespaceId,
} from "./socketioStore";
import {
  machineDeviceIdentification,
  machineIdentificationUnique,
} from "@/machines/types";
import { useRef } from "react";

export const ethercatSetupEventDataSchema = z.object({
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
});

export type EthercatSetupEventData = z.infer<
  typeof ethercatSetupEventDataSchema
>;

export const ethercatSetupEventSchema = eventSchema(
  ethercatSetupEventDataSchema,
);

export type EthercatSetupEvent = z.infer<typeof ethercatSetupEventSchema>;

export const mainNamespaceStoreSchema = z.object({
  ethercatSetup: ethercatSetupEventSchema.nullable(),
});

export type MainNamespaceStore = z.infer<typeof mainNamespaceStoreSchema>;

export const createMainNamespaceStore = (): StoreApi<MainNamespaceStore> => {
  return create<MainNamespaceStore>()(() => ({
    ethercatSetup: null,
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

    // Validate that this is an event type we know about
    if (!(eventName in eventSchemaMap)) {
      handleUnknownEventError(eventName);
    }

    const schema = eventSchemaMap[eventName as keyof typeof eventSchemaMap];

    try {
      // Validate the event against its schema
      const validatedEvent = schema.parse(event);

      // Apply appropriate caching strategy based on event type
      // State events (keep only the latest)
      if (eventName === "EthercatSetupEvent") {
        store.setState(
          produce((state) => {
            state.ethercatSetup = validatedEvent;
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
