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
  handleUnhandledEventError,
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import { useMemo } from "react";

// ========== Event Schema Definitions ==========

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Standby", "Heat", "Extrude"]);
export type Mode = z.infer<typeof modeSchema>;

export const liveMotorStatusDataSchema = z.object({
  screw_rpm: z.number(),
  frequency: z.number(),
  current: z.number(),
  power: z.number(),
});
export type MotorStatus = z.infer<typeof liveMotorStatusDataSchema>;
/**
 * Consolidated live values event schema (30FPS data)
 */
export const liveValuesEventDataSchema = z.object({
  motor_status: liveMotorStatusDataSchema,
  pressure: z.number(),
  nozzle_temperature: z.number(),
  front_temperature: z.number(),
  back_temperature: z.number(),
  middle_temperature: z.number(),
  nozzle_power: z.number(),
  front_power: z.number(),
  back_power: z.number(),
  middle_power: z.number(),
  combined_power: z.number(),
  total_energy_kwh: z.number(),
});

/**
 * Rotation state schema
 */
export const rotationStateSchema = z.object({
  forward: z.boolean(),
});

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});

/**
 * Regulation state schema
 */
export const regulationStateSchema = z.object({
  uses_rpm: z.boolean(),
});

/**
 * Pressure state schema
 */
export const pressureStateSchema = z.object({
  target_bar: z.number(),
  wiring_error: z.boolean(),
});

/**
 * Screw state schema
 */
export const screwStateSchema = z.object({
  target_rpm: z.number(),
});

/**
 * Heating state schema
 */
export const heatingStateSchema = z.object({
  target_temperature: z.number(),
  wiring_error: z.boolean(),
});

export type HeatingState = z.infer<typeof heatingStateSchema>;

/**
 * Heating states schema
 */
export const heatingStatesSchema = z.object({
  nozzle: heatingStateSchema,
  front: heatingStateSchema,
  back: heatingStateSchema,
  middle: heatingStateSchema,
});

/**
 * Extruder settings state schema
 */
export const extruderSettingsStateSchema = z.object({
  pressure_limit: z.number(),
  pressure_limit_enabled: z.boolean(),
});

/**
 * Inverter status state schema
 */
export const inverterStatusStateSchema = z.object({
  running: z.boolean(),
  forward_running: z.boolean(),
  reverse_running: z.boolean(),
  up_to_frequency: z.boolean(),
  overload_warning: z.boolean(),
  no_function: z.boolean(),
  output_frequency_detection: z.boolean(),
  abc_fault: z.boolean(),
  fault_occurence: z.boolean(),
});

/**
 * PID settings schema
 */
export const pidSettingsSchema = z.object({
  temperature: z.object({
    front: z.object({
      ki: z.number(),
      kp: z.number(),
      kd: z.number(),
      zone: z.string(),
    }),
    middle: z.object({
      ki: z.number(),
      kp: z.number(),
      kd: z.number(),
      zone: z.string(),
    }),
    back: z.object({
      ki: z.number(),
      kp: z.number(),
      kd: z.number(),
      zone: z.string(),
    }),
    nozzle: z.object({
      ki: z.number(),
      kp: z.number(),
      kd: z.number(),
      zone: z.string(),
    }),
  }),
  pressure: z.object({
    ki: z.number(),
    kp: z.number(),
    kd: z.number(),
  }),
});

/**
 * Consolidated state event schema (state changes only)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  rotation_state: rotationStateSchema,
  mode_state: modeStateSchema,
  regulation_state: regulationStateSchema,
  pressure_state: pressureStateSchema,
  screw_state: screwStateSchema,
  heating_states: heatingStatesSchema,
  extruder_settings_state: extruderSettingsStateSchema,
  inverter_status_state: inverterStatusStateSchema,
  pid_settings: pidSettingsSchema,
});

// ========== Event Schemas with Wrappers ==========

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========

export type StateEvent = z.infer<typeof stateEventSchema>;

// Additional exports for backward compatibility
export const SetRegulationSchema = z.object({
  uses_rpm: z.boolean(),
});

export const mode = z.object({
  mode: modeSchema,
});

export type Extruder3NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  motorFrequency: TimeSeries;
  motorCurrent: TimeSeries;
  motorScrewRpm: TimeSeries;
  motorPower: TimeSeries;

  pressure: TimeSeries;
  nozzleTemperature: TimeSeries;
  frontTemperature: TimeSeries;
  backTemperature: TimeSeries;
  middleTemperature: TimeSeries;
  nozzlePower: TimeSeries;
  frontPower: TimeSeries;
  middlePower: TimeSeries;
  backPower: TimeSeries;

  // Combined power consumption and energy
  combinedPower: TimeSeries;
  totalEnergyKWh: TimeSeries;
};

const { initialTimeSeries: pressure, insert: addPressure } = createTimeSeries();
const { initialTimeSeries: backTemperature, insert: addBackTemperature } =
  createTimeSeries();
const { initialTimeSeries: frontTemperature, insert: addFrontTemperature } =
  createTimeSeries();
const { initialTimeSeries: middleTemperature, insert: addMiddleTemperature } =
  createTimeSeries();
const { initialTimeSeries: nozzleTemperature, insert: addNozzleTemperature } =
  createTimeSeries();
const { initialTimeSeries: nozzlePower, insert: addNozzlePower } =
  createTimeSeries();
const { initialTimeSeries: frontPower, insert: addFrontPower } =
  createTimeSeries();
const { initialTimeSeries: middlePower, insert: addMiddlePower } =
  createTimeSeries();
const { initialTimeSeries: backPower, insert: addBackPower } =
  createTimeSeries();
const { initialTimeSeries: combinedPower, insert: addCombinedPower } =
  createTimeSeries();
const { initialTimeSeries: totalEnergyKWh, insert: addTotalEnergyKWh } =
  createTimeSeries();
const { initialTimeSeries: motorCurrent, insert: addMotorCurrent } =
  createTimeSeries();
const { initialTimeSeries: motorFrequency, insert: addMotorFrequency } =
  createTimeSeries();
const { initialTimeSeries: motorScrewRpm, insert: addMotorScrewRpm } =
  createTimeSeries();
const { initialTimeSeries: motorPower, insert: addMotorPower } =
  createTimeSeries();

export function extruder3MessageHandler(
  store: StoreApi<Extruder3NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Extruder3NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Extruder3NamespaceStore) => Extruder3NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      if (eventName === "StateEvent") {
        console.log(event);
        const stateEvent = stateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          state: stateEvent,
          // only set default state if is_default_state is true
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
        }));
      } else if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const timestamp = event.ts;
        updateStore((state) => ({
          ...state,
          motorScrewRpm: addMotorScrewRpm(state.motorScrewRpm, {
            value: liveValuesEvent.data.motor_status.screw_rpm,
            timestamp,
          }),
          motorCurrent: addMotorCurrent(state.motorCurrent, {
            value: liveValuesEvent.data.motor_status.current,
            timestamp,
          }),
          motorFrequency: addMotorFrequency(state.motorFrequency, {
            value: liveValuesEvent.data.motor_status.frequency,
            timestamp,
          }),
          motorPower: addMotorPower(state.motorPower, {
            value: liveValuesEvent.data.motor_status.power,
            timestamp,
          }),
          pressure: addPressure(state.pressure, {
            value: liveValuesEvent.data.pressure,
            timestamp,
          }),
          nozzleTemperature: addNozzleTemperature(state.nozzleTemperature, {
            value: liveValuesEvent.data.nozzle_temperature,
            timestamp,
          }),
          frontTemperature: addFrontTemperature(state.frontTemperature, {
            value: liveValuesEvent.data.front_temperature,
            timestamp,
          }),
          backTemperature: addBackTemperature(state.backTemperature, {
            value: liveValuesEvent.data.back_temperature,
            timestamp,
          }),
          middleTemperature: addMiddleTemperature(state.middleTemperature, {
            value: liveValuesEvent.data.middle_temperature,
            timestamp,
          }),
          nozzlePower: addNozzlePower(state.nozzlePower, {
            value: liveValuesEvent.data.nozzle_power,
            timestamp,
          }),
          frontPower: addFrontPower(state.frontPower, {
            value: liveValuesEvent.data.front_power,
            timestamp,
          }),
          middlePower: addMiddlePower(state.middlePower, {
            value: liveValuesEvent.data.middle_power,
            timestamp,
          }),
          backPower: addBackPower(state.backPower, {
            value: liveValuesEvent.data.back_power,
            timestamp,
          }),
          combinedPower: addCombinedPower(state.combinedPower, {
            value: liveValuesEvent.data.combined_power,
            timestamp,
          }),
          totalEnergyKWh: addTotalEnergyKWh(state.totalEnergyKWh, {
            value: liveValuesEvent.data.total_energy_kwh,
            timestamp,
          }),
        }));
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (error) {
      console.error(`Unexpected error processing ${eventName} event:`, error);
      throw error;
    }
  };
}

export const createExtruder3NamespaceStore =
  (): StoreApi<Extruder3NamespaceStore> =>
    create<Extruder3NamespaceStore>(() => {
      return {
        state: null,
        defaultState: null,

        motorCurrent,
        motorFrequency,
        motorScrewRpm,
        motorPower,

        pressure,
        nozzleTemperature,
        frontTemperature,
        backTemperature,
        middleTemperature,
        nozzlePower,
        frontPower,
        backPower,
        middlePower,
        combinedPower,
        totalEnergyKWh,
      };
    });

const useExtruder3NamespaceImplementation =
  createNamespaceHookImplementation<Extruder3NamespaceStore>({
    createStore: createExtruder3NamespaceStore,
    createEventHandler: extruder3MessageHandler,
  });

export function useExtruder3Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Extruder3NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useExtruder3NamespaceImplementation(namespaceId);
}
