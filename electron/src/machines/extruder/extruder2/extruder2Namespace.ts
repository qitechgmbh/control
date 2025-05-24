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

  heatingNozzleState: HeatingStateEvent | null;
  heatingFrontState: HeatingStateEvent | null;
  heatingBackState: HeatingStateEvent | null;
  heatingMiddleState: HeatingStateEvent | null;

  motorRpmState: MotorRpmStateEvent | null;
  motorBarState: MotorPressureStateEvent | null;
  motorRegulationState: MotorRegulationStateEvent | null;

  // Metric Events (cached for 1 hour )
  rpm: TimeSeries;
  bar: TimeSeries;

  nozzleTemperature: TimeSeries;
  frontTemperature: TimeSeries;
  backTemperature: TimeSeries;
  middleTemperature: TimeSeries;
};

// Constants for time durations
const ONE_SECOND = 1000;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: backTemperature, insert: addBackTemperature } =
  createTimeSeries(ONE_SECOND, ONE_HOUR);

const { initialTimeSeries: frontTemperature, insert: addFrontTemperature } =
  createTimeSeries(ONE_SECOND, ONE_HOUR);

const { initialTimeSeries: middleTemperature, insert: addMiddleTemperature } =
  createTimeSeries(ONE_SECOND, ONE_HOUR);

const { initialTimeSeries: nozzleTemperature, insert: addNozzleTemperature } =
  createTimeSeries(ONE_SECOND, ONE_HOUR);

const { initialTimeSeries: rpm, insert: addRpm } = createTimeSeries(
  ONE_SECOND,
  ONE_HOUR,
);

const { initialTimeSeries: bar, insert: addBar } = createTimeSeries(
  ONE_SECOND,
  ONE_HOUR,
);

export function extruder2MessageHandler(
  store: StoreApi<Extruder2NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;
    try {
      if (eventName == "InverterStatusEvent") {
        // TODO: Handle if needed
      } else if (eventName == "RotationStateEvent") {
        store.setState(
          produce(store.getState(), (state) => {
            state.rotationState = inverterRotationEventSchema.parse(event);
          }),
        );
      } else if (eventName == "ModeStateEvent") {
        store.setState(
          produce(store.getState(), (state) => {
            state.modeState = modeStateEventSchema.parse(event);
          }),
        );
      } else if (eventName == "FrontHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.temperature,
          timestamp: event.ts,
        };

        store.setState(
          produce(store.getState(), (state) => {
            state.heatingFrontState = parsed;
            state.frontTemperature = addFrontTemperature(
              state.frontTemperature,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName == "NozzleHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.temperature,
          timestamp: event.ts,
        };

        store.setState(
          produce(store.getState(), (state) => {
            state.heatingNozzleState = parsed;
            state.nozzleTemperature = addNozzleTemperature(
              state.nozzleTemperature,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName == "BackHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.temperature,
          timestamp: event.ts,
        };

        store.setState(
          produce(store.getState(), (state) => {
            state.heatingBackState = parsed;
            state.backTemperature = addBackTemperature(
              state.backTemperature,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName == "MiddleHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.temperature,
          timestamp: event.ts,
        };

        store.setState(
          produce(store.getState(), (state) => {
            state.heatingMiddleState = parsed;
            state.middleTemperature = addMiddleTemperature(
              state.middleTemperature,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName == "RegulationStateEvent") {
        store.setState(
          produce(store.getState(), (state) => {
            state.motorRegulationState =
              motorRegulationEventSchema.parse(event);
          }),
        );
      } else if (eventName == "PressureStateEvent") {
        const parsed = motorPressureStateEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.bar,
          timestamp: event.ts,
        };

        store.setState(
          produce(store.getState(), (state) => {
            state.motorBarState = parsed;
            state.bar = addBar(state.bar, timeseriesValue);
          }),
        );
      } else if (eventName == "RpmStateEvent") {
        const parsed = motorRpmStateEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.rpm,
          timestamp: event.ts,
        };

        store.setState(
          produce(store.getState(), (state) => {
            state.motorRpmState = parsed;
            state.rpm = addRpm(state.rpm, timeseriesValue);
          }),
        );
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

        heatingNozzleState: null,
        heatingFrontState: null,
        heatingBackState: null,
        heatingMiddleState: null,

        motorRpmState: null,
        motorRegulationState: null,
        motorBarState: null,

        rpm,
        bar,
        nozzleTemperature,
        frontTemperature,
        backTemperature,
        middleTemperature,
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
    machine_identification_unique,
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
  wiring_error: z.boolean(),
});

export const heatingTargetTemperatureDataSchema = z.object({
  target_temperature: z.number(),
});

export const motorRpmStateEventDataSchema = z.object({
  rpm: z.number(),
  target_rpm: z.number(),
});

export const motorBarStateEventDataSchema = z.object({
  bar: z.number(),
  target_bar: z.number(),
});

export const motorRegulationEventDataSchema = z.object({
  uses_rpm: z.boolean(),
});

// Event Schemas
export const heatingTargetEventSchema = eventSchema(
  heatingTargetTemperatureDataSchema,
);

export const motorRpmStateEventSchema = eventSchema(
  motorRpmStateEventDataSchema,
);

export const motorPressureStateEventSchema = eventSchema(
  motorBarStateEventDataSchema,
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
export type MotorPressureStateEvent = z.infer<
  typeof motorPressureStateEventSchema
>;
export type InverterStatusEvent = z.infer<typeof inverterStatusEventSchema>;
export type InverterRotationEvent = z.infer<typeof inverterRotationEventSchema>;

export type MotorRegulationStateEvent = z.infer<
  typeof motorRegulationEventSchema
>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;
export type HeatingStateEvent = z.infer<typeof heatingStateEventSchema>;

export type HeatingType = z.infer<typeof heatingTypeSchema>;
export type Heating = z.infer<typeof heatingStateDataSchema>;

export type MotorPressure = z.infer<typeof motorBarStateEventDataSchema>;
export type MotorRpm = z.infer<typeof motorRpmStateEventDataSchema>;

export type Mode = z.infer<typeof modeSchema>;
