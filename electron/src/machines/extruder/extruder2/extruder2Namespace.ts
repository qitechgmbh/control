import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  NamespaceId,
  createNamespaceHookImplementation,
  ThrottledStoreUpdater,
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

export type Extruder2NamespaceStore = {
  modeState: ModeStateEvent | null;
  inverterState: InverterStatusEvent | null;
  rotationState: InverterRotationEvent | null;
  motorRpmState: MotorScrewStateEvent | null;
  motorBarState: MotorPressureStateEvent | null;
  motorRegulationState: MotorRegulationStateEvent | null;
  extruderSettingsState: ExtruderSettingsStateEvent | null;
  pressurePidSettings: PidSettingsEvent | null;

  nozzleTargetTemperature: number | null;
  frontTargetTemperature: number | null;
  middleTargetTemperature: number | null;
  backTargetTemperature: number | null;

  nozzleWiringError: boolean | null;
  frontWiringError: boolean | null;
  backWiringError: boolean | null;
  middleWiringError: boolean | null;

  // Metric Events (cached for 1 hour )
  rpm: TimeSeries;
  bar: TimeSeries;
  nozzleTemperature: TimeSeries;
  frontTemperature: TimeSeries;
  backTemperature: TimeSeries;
  middleTemperature: TimeSeries;
  nozzlePower: TimeSeries;
  frontPower: TimeSeries;
  middlePower: TimeSeries;
  backPower: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: backTemperature, insert: addBackTemperature } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: frontTemperature, insert: addFrontTemperature } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: middleTemperature, insert: addMiddleTemperature } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: nozzleTemperature, insert: addNozzleTemperature } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: nozzlePower, insert: addNozzlePower } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: frontPower, insert: addFrontPower } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: middlePower, insert: addMiddlePower } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: backPower, insert: addBackPower } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

const { initialTimeSeries: rpm, insert: addRpm } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

const { initialTimeSeries: bar, insert: addBar } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

export function extruder2MessageHandler(
  store: StoreApi<Extruder2NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Extruder2NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Extruder2NamespaceStore) => Extruder2NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      if (eventName == "PressurePidSettingsEvent") {
        updateStore((state) => ({
          ...state,
          pressurePidSettings: pidSettingsEventSchema.parse(event),
        }));
      }

      if (eventName == "ExtruderSettingsStateEvent") {
        updateStore((state) => ({
          ...state,
          extruderSettingsState: extruderSettingsStateEventSchema.parse(event),
        }));
      }

      if (eventName == "InverterStatusEvent") {
        console.log(eventName);
        updateStore((state) => ({
          ...state,
          inverterState: event as InverterStatusEvent,
        }));
      } else if (eventName == "RotationStateEvent") {
        updateStore((state) => ({
          ...state,
          rotationState: event as InverterRotationEvent,
        }));
      } else if (eventName == "ModeStateEvent") {
        updateStore((state) => ({
          ...state,
          modeState: event as ModeStateEvent,
        }));
      } else if (eventName == "FrontHeatingTemperatureEvent") {
        const timeseriesValue: TimeSeriesValue = {
          value: event.data.temperature,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          frontTemperature: addFrontTemperature(
            state.frontTemperature,
            timeseriesValue,
          ),
        }));
      } else if (eventName == "NozzleHeatingTemperatureEvent") {
        const timeseriesValue: TimeSeriesValue = {
          value: event.data.temperature,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          nozzleTemperature: addNozzleTemperature(
            state.nozzleTemperature,
            timeseriesValue,
          ),
        }));
      } else if (eventName == "BackHeatingTemperatureEvent") {
        const timeseriesValue: TimeSeriesValue = {
          value: event.data.temperature,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          backTemperature: addBackTemperature(
            state.backTemperature,
            timeseriesValue,
          ),
        }));
      } else if (eventName == "MiddleHeatingTemperatureEvent") {
        const timeseriesValue: TimeSeriesValue = {
          value: event.data.temperature,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          middleTemperature: addMiddleTemperature(
            state.middleTemperature,
            timeseriesValue,
          ),
        }));
      } else if (eventName == "FrontHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          frontTargetTemperature: parsed.data.target_temperature,
          frontWiringError: parsed.data.wiring_error,
        }));
      } else if (eventName == "BackHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          backTargetTemperature: parsed.data.target_temperature,
          backWiringError: parsed.data.wiring_error,
        }));
      } else if (eventName == "MiddleHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          middleTargetTemperature: parsed.data.target_temperature,
          middleWiringError: parsed.data.wiring_error,
        }));
      } else if (eventName == "NozzleHeatingStateEvent") {
        const parsed = heatingStateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          nozzleTargetTemperature: parsed.data.target_temperature,
          nozzleWiringError: parsed.data.wiring_error,
        }));
      } else if (eventName == "NozzleHeatingPowerEvent") {
        const parsed = heatingPowerEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.wattage,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          nozzlePower: addNozzlePower(state.nozzlePower, timeseriesValue),
        }));
      } else if (eventName == "FrontHeatingPowerEvent") {
        const parsed = heatingPowerEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.wattage,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          frontPower: addFrontPower(state.frontPower, timeseriesValue),
        }));
      } else if (eventName == "MiddleHeatingPowerEvent") {
        const parsed = heatingPowerEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.wattage,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          middlePower: addMiddlePower(state.middlePower, timeseriesValue),
        }));
      } else if (eventName == "BackHeatingPowerEvent") {
        const parsed = heatingPowerEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.wattage,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          backPower: addBackPower(state.backPower, timeseriesValue),
        }));
      } else if (eventName == "RegulationStateEvent") {
        updateStore((state) => ({
          ...state,
          motorRegulationState: event as MotorRegulationStateEvent,
        }));
      } else if (eventName == "PressureStateEvent") {
        const pressureEvent = event as MotorPressureStateEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: pressureEvent.data.bar,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          motorBarState: pressureEvent,
          bar: addBar(state.bar, timeseriesValue),
        }));
      } else if (eventName == "ScrewStateEvent") {
        const screwEvent = event as MotorScrewStateEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: screwEvent.data.rpm,
          timestamp: event.ts,
        };

        updateStore((state) => ({
          ...state,
          motorRpmState: screwEvent,
          rpm: addRpm(state.rpm, timeseriesValue),
        }));
      }
    } catch (error) {
      console.error(`Unexpected error processing ${eventName} event:`, error);
      throw error;
    }
  };
}

export const createExtruder2NamespaceStore =
  (): StoreApi<Extruder2NamespaceStore> =>
    create<Extruder2NamespaceStore>(() => {
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
        extruderSettingsState: null,

        pressurePidSettings: null,
        temperaturePidSettings: null,

        nozzleTargetTemperature: null,
        frontTargetTemperature: null,
        middleTargetTemperature: null,
        backTargetTemperature: null,

        frontWiringError: null,
        backWiringError: null,
        middleWiringError: null,
        nozzleWiringError: null,

        // Timeseries:
        rpm,
        bar,
        nozzleTemperature,
        frontTemperature,
        middleTemperature,
        backTemperature,

        nozzlePower,
        frontPower,
        backPower,
        middlePower,
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
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };
  // Use the implementation with validated namespace ID
  return useExtruder2NamespaceImplementation(namespaceId);
}

export const modeSchema = z.enum(["Standby", "Heat", "Extrude"]);
export const mode = z.object({
  mode: modeSchema,
});
export const heatingTypeSchema = z.enum(["front", "back", "middle"]);

export const SetRegulationSchema = z.object({
  uses_rpm: z.boolean(),
});

// Data Schemas

export const inverterStatusEventDataSchema = z.object({
  // RUN (Inverter running)
  running: z.boolean(),
  // Forward running motor spins forward
  forward_running: z.boolean(),
  // Reverse running motor spins backwards
  reverse_running: z.boolean(),
  // Up to frequency, SU not completely sure what its for
  up_to_frequency: z.boolean(),
  // overload warning OL
  overload_warning: z.boolean(),
  // No function, its described that way in the datasheet
  no_function: z.boolean(),
  // FU Output Frequency Detection
  output_frequency_detection: z.boolean(),
  // ABC (Fault)
  abc_fault: z.boolean(),
  // is True when a fault occured
  fault_occurence: z.boolean(),
});

export const modeStateEventDataSchema = z.object({
  mode: modeSchema,
});

export const inverterRotationEventDataSchema = z.object({
  forward: z.boolean(),
});

export const heatingTemperatureDataSchema = z.object({
  temperature: z.number(),
});

export const heatingStateDataSchema = z.object({
  target_temperature: z.number(),
  wiring_error: z.boolean(),
});

export const motorScrewStateEventDataSchema = z.object({
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

export const extruderPressureLimitDataSchema = z.object({
  pressure_limit: z.number(),
});

export const extruderPressureLimitEnabledDataSchema = z.object({
  pressure_limit_enabled: z.boolean(),
});

export const extruderSettingsStateEventDataSchema = z.object({
  pressure_limit: z.number(),
  pressure_limit_enabled: z.boolean(),
});

export const pidSettingsEventDataSchema = z.object({
  ki: z.number(),
  kd: z.number(),
  kp: z.number(),
});

// Event Schemas
export const heatingTemperatureEventSchema = eventSchema(
  heatingTemperatureDataSchema,
);

export const heatingStateEventSchema = eventSchema(heatingStateDataSchema);

export const motorScrewStateEventSchema = eventSchema(
  motorScrewStateEventDataSchema,
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
export const modeStateEventSchema = eventSchema(modeStateEventDataSchema);

export const extruderSettingsStateEventSchema = eventSchema(
  extruderSettingsStateEventDataSchema,
);

export const pidSettingsEventSchema = eventSchema(pidSettingsEventDataSchema);

export const heatingPowerEventDataSchema = z.object({
  wattage: z.number(),
});

export const heatingPowerEventSchema = eventSchema(heatingPowerEventDataSchema);

export const inverterStatusEventSchema = eventSchema(
  inverterStatusEventDataSchema,
);

// type defs
export type MotorScrewStateEvent = z.infer<typeof motorScrewStateEventSchema>;
export type MotorPressureStateEvent = z.infer<
  typeof motorPressureStateEventSchema
>;
export type InverterStatusEvent = z.infer<typeof inverterStatusEventSchema>;
export type InverterRotationEvent = z.infer<typeof inverterRotationEventSchema>;

export type MotorRegulationStateEvent = z.infer<
  typeof motorRegulationEventSchema
>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;
export type HeatingPowerEvent = z.infer<typeof heatingPowerEventSchema>;

export type HeatingType = z.infer<typeof heatingTypeSchema>;

export type MotorPressure = z.infer<typeof motorBarStateEventDataSchema>;
export type MotorRpm = z.infer<typeof motorScrewStateEventDataSchema>;
export type Mode = z.infer<typeof modeSchema>;
export type InverterStatus = z.infer<typeof inverterStatusEventDataSchema>;

export type PidSettings = z.infer<typeof pidSettingsEventDataSchema>;
export type PidSettingsEvent = z.infer<typeof pidSettingsEventSchema>;

export type ExtruderSettingsStateEvent = z.infer<
  typeof extruderSettingsStateEventSchema
>;
