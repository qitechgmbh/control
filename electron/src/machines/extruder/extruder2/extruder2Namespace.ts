import { StoreApi } from "zustand";
import { create } from "zustand";
import { produce } from "immer";
import { number, z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  handleEventValidationError,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { useRef } from "react";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

export type Extruder2NamespaceStore = {
  modeState: ModeStateEvent | null;
  inverterState: InverterStatusEvent | null;
  rotationState: InverterRotationEvent | null;
};

export function extruder2MessageHandler(
  store: StoreApi<Extruder2NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;
    console.log(event);
    try {
      if (eventName == "InverterStatusEvent") {
      } else if (eventName == "RotationStateEvent") {
        store.setState(
          produce(store.getState(), (state) => {
            state.rotationState = inverterRotationEventSchema.parse(event);
          }),
        );
      } else if (eventName == "ModeStateEvent") {
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

export const createExtruder2NamespaceStore =
  (): StoreApi<Extruder2NamespaceStore> =>
    create<Extruder2NamespaceStore>((set) => {
      return {
        inverterState: null,
        rotationState: null,
        modeState: null,
      };
    });

const useExtruder2NamespaceImplementation =
  createNamespaceHookImplementation<Extruder2NamespaceStore>({
    createStore: createExtruder2NamespaceStore,
    createEventHandler: extruder2MessageHandler,
  });

export function useExtruder2Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Extruder2NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useRef<NamespaceId>({
    type: "machine",
    vendor: machine_identification_unique.vendor,
    serial: machine_identification_unique.serial,
    machine: machine_identification_unique.machine,
  });

  // Use the implementation with validated namespace ID
  return useExtruder2NamespaceImplementation(namespaceId.current);
}

export const inverterStatusEventSchema = z.object({});

export const inverterRotationEventDataSchema = z.object({
  forward: z.boolean(),
});

export const inverterRotationEventSchema = eventSchema(
  inverterRotationEventDataSchema,
);

export const modeSchema = z.enum(["Standby", "Heat", "Extrude"]);
export const modeStateEventDataSchema = z.object({
  mode: modeSchema,
});
export const mode = z.object({
  mode: modeSchema,
});

export const modeStateEventSchema = eventSchema(modeStateEventDataSchema);

export type Mode = z.infer<typeof modeSchema>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;

export type InverterStatusEvent = z.infer<typeof inverterStatusEventSchema>;
export type InverterRotationEvent = z.infer<typeof inverterRotationEventSchema>;
