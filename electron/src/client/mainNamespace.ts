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
  serializeNamespaceId,
  useSocketioStore,
} from "./socketioStore";
import {
  deviceIdentification,
  machineIdentificationUnique,
} from "@/machines/types";
import { useRef } from "react";
import { rustEnum } from "@/lib/types";
import { produce } from "immer"; // <-- Added Import for state updates

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
  State: z.string(),
});

export type EthercatStateEventData = z.infer<
  typeof ethercatStateEventDataSchema
>;

export const ethercatStateEventSchema = eventSchema(
  ethercatStateEventDataSchema,
);

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
          const oldMachines = currentMachinesState.data.machines;
          const newMachines = validatedEvent.data.machines;

          // Find machines that existed before but are missing in the new event payload
          const removedMachines = oldMachines.filter(
            (oldM) =>
              !newMachines.some(
                (newM) =>
                  newM.machine_identification_unique.serial ===
                  oldM.machine_identification_unique.serial,
              ),
          );

          // Disconnect and clean up each missing machine's socket namespace dynamically
          removedMachines.forEach((machine) => {
            const targetNamespaceId: NamespaceId = {
              type: "machine",
              machine_identification_unique:
                machine.machine_identification_unique,
            };
            const namespace_path = serializeNamespaceId(targetNamespaceId);

            // Access your global socketio store state out of context
            const socketStoreState = useSocketioStore.getState();
            const namespace = socketStoreState.namespaces[namespace_path];

            if (namespace) {
              console.log(
                `Cleaning up removed machine namespace: ${namespace_path}`,
              );
              useSocketioStore.setState(
                produce((state) => {
                  const ns = state.namespaces[namespace_path];
                  if (ns) {
                    ns.socket.disconnect();
                    ns.throttledUpdater.destroy();
                    delete state.namespaces[namespace_path];
                  }
                }),
              );
            }
          });
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

export const mainMessageHandlerInstance: EventHandler = mainMessageHandler(
  mainNamespaceStore,
  new ThrottledStoreUpdater(mainNamespaceStore),
);

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
