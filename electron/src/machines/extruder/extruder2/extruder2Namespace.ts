import { StoreApi } from "zustand";
import { create } from "zustand";
import { produce } from "immer";
import { boolean, number, z } from "zod";
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
  heatingFrontState: HeatingStateEvent | null;
  heatingBackState: HeatingStateEvent | null;
  heatingMiddleState: HeatingStateEvent | null;
  motorRpmState: MotorRpmStateEvent | null;
  motorRegulationState: MotorRegulationStateEvent | null;
};

export function extruder2MessageHandler(
  store: StoreApi<Extruder2NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;
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
        modeState: null,
        inverterState: null,
        rotationState: null,
        heatingFrontState: null,
        heatingBackState: null,
        heatingMiddleState: null,
        motorRpmState: null,
        motorRegulationState: null,
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
export const modeSchema = z.enum(["Standby", "Heat", "Extrude"]);
export const mode = z.object({
  mode: modeSchema,
});
export const heatingTypeSchema = z.enum(["front", "back", "middle"]);

export const SetRegulationSchema = z.object({
  uses_rpm: z.boolean(),
});

// Data Schemas
export const modeStateEventDataSchema = z.object({
  mode: modeSchema,
});

export const inverterRotationEventDataSchema = z.object({
  forward: z.boolean(),
});

export const heatingStateDataSchema = z.object({
  temperature: z.number(),
  heating: z.boolean(),
  target_temperature: z.number(),
});

export const motorRpmStateEventDataSchema = z.object({
  rpm: z.number(),
});

export const motorRegulationEventDataSchema = z.object({
  uses_rpm: z.boolean(),
});

// Event Schemas
export const motorRpmStateEventSchema = eventSchema(
  motorRpmStateEventDataSchema,
);

export const inverterRotationEventSchema = eventSchema(
  inverterRotationEventDataSchema,
);

export const motorRegulationEventSchema = eventSchema(
  motorRegulationEventDataSchema,
);
export const heatingStateEventSchema = eventSchema(heatingStateDataSchema);
export const modeStateEventSchema = eventSchema(modeStateEventDataSchema);

// type defs
export type MotorRpmStateEvent = z.infer<typeof motorRpmStateEventSchema>;

export type HeatingType = z.infer<typeof heatingTypeSchema>;
export type Heating = z.infer<typeof heatingStateDataSchema>;
export type HeatingStateEvent = z.infer<typeof heatingStateEventSchema>;

export type Mode = z.infer<typeof modeSchema>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;

export type InverterStatusEvent = z.infer<typeof inverterStatusEventSchema>;
export type InverterRotationEvent = z.infer<typeof inverterRotationEventSchema>;

export type MotorRegulationStateEvent = z.infer<
  typeof motorRegulationEventSchema
>;
