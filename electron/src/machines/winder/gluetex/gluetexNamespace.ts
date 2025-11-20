/**
 * @file gluetexNamespace.ts
 * @description TypeScript implementation of Gluetex namespace with real backend connection
 * Standard winder features connect to backend, addon features use local state
 */

import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
  ThrottledStoreUpdater,
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { useMemo } from "react";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

// ========== Event Schema Definitions (Backend) ==========

/**
 * Consolidated live values event schema (30FPS data)
 */
export const liveValuesEventDataSchema = z.object({
  traverse_position: z.number().nullable(),
  puller_speed: z.number(),
  spool_rpm: z.number(),
  tension_arm_angle: z.number(),
  spool_progress: z.number(),
  temperature_1: z.number(),
  temperature_2: z.number(),
  temperature_3: z.number(),
  temperature_4: z.number(),
  temperature_5: z.number(),
  temperature_6: z.number(),
  heater_1_power: z.number(),
  heater_2_power: z.number(),
  heater_3_power: z.number(),
  heater_4_power: z.number(),
  heater_5_power: z.number(),
  heater_6_power: z.number(),
  slave_puller_speed: z.number(),
  slave_tension_arm_angle: z.number(),
});

/**
 * Puller regulation type enum
 */
export const pullerRegulationSchema = z.enum(["Speed", "Diameter"]);
export type PullerRegulation = z.infer<typeof pullerRegulationSchema>;

/**
 * Gear ratio enum for winding speed
 */
export const gearRatioSchema = z.enum(["OneToOne", "OneToFive", "OneToTen"]);
export type GearRatio = z.infer<typeof gearRatioSchema>;

/**
 * Get the multiplier for a gear ratio
 */
export function getGearRatioMultiplier(
  gearRatio: GearRatio | undefined,
): number {
  switch (gearRatio) {
    case "OneToOne":
      return 1.0;
    case "OneToFive":
      return 5.0;
    case "OneToTen":
      return 10.0;
    default:
      return 1.0;
  }
}

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Standby", "Hold", "Pull", "Wind"]);
export type Mode = z.infer<typeof modeSchema>;

/**
 * Machine operation mode enum
 */
export const spoolAutomaticActionModeSchema = z.enum([
  "NoAction",
  "Pull",
  "Hold",
]);

export type SpoolAutomaticActionMode = z.infer<
  typeof spoolAutomaticActionModeSchema
>;

/**
 * Spool speed controller regulation mode enum
 */
export const spoolRegulationModeSchema = z.enum(["Adaptive", "MinMax"]);
export type SpoolRegulationMode = z.infer<typeof spoolRegulationModeSchema>;

export const spoolAutomaticActionStateSchema = z.object({
  spool_required_meters: z.number(),
  spool_automatic_action_mode: spoolAutomaticActionModeSchema,
});

export type SpoolAutomaticActionState = z.infer<
  typeof spoolAutomaticActionStateSchema
>;

/**
 * Traverse state schema
 */
export const traverseStateSchema = z.object({
  limit_inner: z.number(),
  limit_outer: z.number(),
  position_in: z.number(),
  position_out: z.number(),
  is_going_in: z.boolean(),
  is_going_out: z.boolean(),
  is_homed: z.boolean(),
  is_going_home: z.boolean(),
  is_traversing: z.boolean(),
  laserpointer: z.boolean(),
  step_size: z.number(),
  padding: z.number(),
  can_go_in: z.boolean(),
  can_go_out: z.boolean(),
  can_go_home: z.boolean(),
});

/**
 * Puller state schema
 */
export const pullerStateSchema = z.object({
  regulation: pullerRegulationSchema,
  target_speed: z.number(),
  target_diameter: z.number(),
  forward: z.boolean(),
  gear_ratio: gearRatioSchema,
});

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
  can_wind: z.boolean(),
});

/**
 *  Connected machine state scheme
 */
export const machineIdentificationSchema = z.object({
  vendor: z.number(),
  machine: z.number(),
});

export const machineIdentificationUniqueSchema = z.object({
  machine_identification: machineIdentificationSchema,
  serial: z.number(),
});

export const connectedMachineStateSchema = z.object({
  machine_identification_unique: machineIdentificationUniqueSchema.nullable(),
  is_available: z.boolean(),
});

/**
 * Tension arm state schema
 */
export const tensionArmStateSchema = z.object({
  zeroed: z.boolean(),
});

/**
 * Spool speed controller state schema
 */
export const spoolSpeedControllerStateSchema = z.object({
  regulation_mode: spoolRegulationModeSchema,
  minmax_min_speed: z.number(),
  minmax_max_speed: z.number(),
  adaptive_tension_target: z.number(),
  adaptive_radius_learning_rate: z.number(),
  adaptive_max_speed_multiplier: z.number(),
  adaptive_acceleration_factor: z.number(),
  adaptive_deacceleration_urgency_multiplier: z.number(),
  forward: z.boolean(),
});

/**
 * Heating state schema
 */
export const heatingStateSchema = z.object({
  target_temperature: z.number(),
  wiring_error: z.boolean(),
});

/**
 * Heating states schema
 */
export const heatingStatesSchema = z.object({
  zone_1: heatingStateSchema,
  zone_2: heatingStateSchema,
  zone_3: heatingStateSchema,
  zone_4: heatingStateSchema,
  zone_5: heatingStateSchema,
  zone_6: heatingStateSchema,
});

/**
 * Addon motor state schema (from backend)
 */
export const addonMotorStateSchema = z.object({
  enabled: z.boolean(),
  master_ratio: z.number(),
  slave_ratio: z.number(),
});

/**
 * Consolidated state event schema (state changes only) - from backend
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  traverse_state: traverseStateSchema,
  puller_state: pullerStateSchema,
  mode_state: modeStateSchema,
  tension_arm_state: tensionArmStateSchema,
  spool_speed_controller_state: spoolSpeedControllerStateSchema,
  spool_automatic_action_state: spoolAutomaticActionStateSchema,
  heating_states: heatingStatesSchema,
  connected_machine_state: connectedMachineStateSchema,
  addon_motor_3_state: addonMotorStateSchema,
  addon_motor_4_state: addonMotorStateSchema,
});

// ========== Event Schemas with Wrappers ==========

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventSchema>;
export type StateEventData = z.infer<typeof stateEventDataSchema>;

// ========== Addon Types (Local State Only) ==========

export type StepperMode = "Standby" | "Run";
export type HeatingMode = "Standby" | "Heating";

export type SlavePullerState = {
  enabled: boolean;
  forward: boolean;
  min_angle: number;
  max_angle: number;
  min_speed_factor: number | null;
  max_speed_factor: number | null;
  tension_arm: {
    zeroed: boolean;
  };
};

export type MotorRatiosState = {
  stepper3_master: number;
  stepper3_slave: number;
  stepper4_master: number;
  stepper4_slave: number;
};

export type StepperState = {
  stepper3_mode: StepperMode;
  stepper4_mode: StepperMode;
};

/**
 * Helper to convert backend addon motor state to frontend MotorRatiosState
 */
function getMotorRatiosFromBackend(
  motor3: z.infer<typeof addonMotorStateSchema>,
  motor4: z.infer<typeof addonMotorStateSchema>,
): MotorRatiosState {
  return {
    stepper3_master: motor3.master_ratio,
    stepper3_slave: motor3.slave_ratio,
    stepper4_master: motor4.master_ratio,
    stepper4_slave: motor4.slave_ratio,
  };
}

/**
 * Helper to determine stepper4 mode from backend enabled state
 */
function getStepper4ModeFromBackend(
  motor4: z.infer<typeof addonMotorStateSchema>,
): StepperMode {
  return motor4.enabled ? "Run" : "Standby";
}

export type HeatingState = {
  heating_mode: HeatingMode;
};

export type TemperatureState = {
  current_temperature: number;
  min_temperature: number;
  max_temperature: number;
};

export type QualityControlState = {
  temperature1: TemperatureState;
  temperature2: TemperatureState;
};

/**
 * Extended state event data with addon fields
 */
export type ExtendedStateEventData = StateEventData & {
  slave_puller_state: SlavePullerState;
  motor_ratios_state: MotorRatiosState;
  stepper_state: StepperState;
  heating_state: HeatingState;
  quality_control_state: QualityControlState;
};

/**
 * Extended state event with addon fields
 */
export type ExtendedStateEvent = {
  name: string;
  ts: number;
  data: ExtendedStateEventData;
};

// ========== Store Definition ==========

export type GluetexNamespaceStore = {
  // State event from server (extended with addon fields)
  state: ExtendedStateEvent | null;
  defaultState: ExtendedStateEvent | null;

  // Time series data for live values (from backend)
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  spoolRpm: TimeSeries;
  tensionArmAngle: TimeSeries;
  spoolProgress: TimeSeries;
  temperature1: TimeSeries;
  temperature2: TimeSeries;
  temperature3: TimeSeries;
  temperature4: TimeSeries;
  temperature5: TimeSeries;
  temperature6: TimeSeries;
  heater1Power: TimeSeries;
  heater2Power: TimeSeries;
  heater3Power: TimeSeries;
  heater4Power: TimeSeries;
  heater5Power: TimeSeries;
  heater6Power: TimeSeries;

  // Time series data for addons (local)
  slavePullerSpeed: TimeSeries;
  slaveTensionArmAngle: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

// Create time series for backend values
const { initialTimeSeries: spoolProgress, insert: addSpoolProgress } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: traversePosition, insert: addTraversePosition } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: pullerSpeed, insert: addPullerSpeed } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: spoolRpm, insert: addSpoolRpm } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);
const { initialTimeSeries: tensionArmAngle, insert: addTensionArmAngle } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

// Create time series for temperature values (from backend)
const { initialTimeSeries: temperature1, insert: addTemperature1 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: temperature2, insert: addTemperature2 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: temperature3, insert: addTemperature3 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: temperature4, insert: addTemperature4 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: temperature5, insert: addTemperature5 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: temperature6, insert: addTemperature6 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: heater1Power, insert: addHeater1Power } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: heater2Power, insert: addHeater2Power } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: heater3Power, insert: addHeater3Power } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: heater4Power, insert: addHeater4Power } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: heater5Power, insert: addHeater5Power } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: heater6Power, insert: addHeater6Power } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

// Create time series for addon values (local)
const { initialTimeSeries: slavePullerSpeed, insert: addSlavePullerSpeed } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const {
  initialTimeSeries: slaveTensionArmAngle,
  insert: addSlaveTensionArmAngle,
} = createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

// Default addon state
const DEFAULT_ADDON_STATE = {
  slave_puller_state: {
    enabled: false,
    forward: true,
    min_angle: 20.0,
    max_angle: 90.0,
    min_speed_factor: null,
    max_speed_factor: null,
    tension_arm: {
      zeroed: false,
    },
  },
  motor_ratios_state: {
    stepper3_master: 1.0,
    stepper3_slave: 1.0,
    stepper4_master: 1.0,
    stepper4_slave: 1.0,
  },
  stepper_state: {
    stepper3_mode: "Standby" as StepperMode,
    stepper4_mode: "Standby" as StepperMode,
  },
  heating_state: {
    heating_mode: "Standby" as HeatingMode,
  },
  quality_control_state: {
    temperature1: {
      current_temperature: 85.0,
      min_temperature: 75.0,
      max_temperature: 95.0,
    },
    temperature2: {
      current_temperature: 125.0,
      min_temperature: 115.0,
      max_temperature: 135.0,
    },
  },
};

/**
 * Factory function to create a new Gluetex namespace store
 * @returns A new Zustand store instance for Gluetex namespace
 */
export const createGluetexNamespaceStore =
  (): StoreApi<GluetexNamespaceStore> =>
    create<GluetexNamespaceStore>(() => {
      return {
        // State event from server (will be extended with addon state)
        state: null,
        defaultState: null,

        // Time series data for live values
        traversePosition,
        pullerSpeed,
        spoolRpm,
        tensionArmAngle,
        spoolProgress,
        temperature1,
        temperature2,
        temperature3,
        temperature4,
        temperature5,
        temperature6,
        heater1Power,
        heater2Power,
        heater3Power,
        heater4Power,
        heater5Power,
        heater6Power,

        // Time series data for addons
        slavePullerSpeed,
        slaveTensionArmAngle,
      };
    });

/**
 * Creates a message handler for Gluetex namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function gluetexMessageHandler(
  store: StoreApi<GluetexNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<GluetexNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: GluetexNamespaceStore) => GluetexNamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      if (eventName === "StateEvent") {
        console.log(event);
        // Parse and validate the state event
        const stateEvent = stateEventSchema.parse(event);

        updateStore((state) => {
          // Derive motor ratios from backend addon motor state
          const motorRatiosState = getMotorRatiosFromBackend(
            stateEvent.data.addon_motor_3_state,
            stateEvent.data.addon_motor_4_state,
          );

          // Derive stepper4 mode from backend enabled state
          const stepper4Mode = getStepper4ModeFromBackend(
            stateEvent.data.addon_motor_4_state,
          );

          // Extend backend state with addon state (some local, some derived from backend)
          const extendedData: ExtendedStateEventData = {
            ...stateEvent.data,
            // Preserve existing local-only addon state
            slave_puller_state:
              state.state?.data.slave_puller_state ||
              DEFAULT_ADDON_STATE.slave_puller_state,
            // Derive from backend addon motor state
            motor_ratios_state: motorRatiosState,
            stepper_state: {
              stepper3_mode:
                state.state?.data.stepper_state.stepper3_mode ||
                DEFAULT_ADDON_STATE.stepper_state.stepper3_mode,
              stepper4_mode: stepper4Mode,
            },
            heating_state:
              state.state?.data.heating_state ||
              DEFAULT_ADDON_STATE.heating_state,
            quality_control_state:
              state.state?.data.quality_control_state ||
              DEFAULT_ADDON_STATE.quality_control_state,
          };

          const extendedState: ExtendedStateEvent = {
            name: stateEvent.name,
            ts: stateEvent.ts,
            data: extendedData,
          };

          return {
            ...state,
            state: extendedState,
            // only set default state if is_default_state is true
            defaultState: stateEvent.data.is_default_state
              ? extendedState
              : state.defaultState,
          };
        });
      } else if (eventName === "LiveValuesEvent") {
        // Parse and validate the live values event
        const liveValuesEvent = liveValuesEventSchema.parse(event);

        // Extract values and add to time series
        const {
          traverse_position,
          puller_speed,
          spool_rpm,
          tension_arm_angle,
          spool_progress,
          temperature_1,
          temperature_2,
          temperature_3,
          temperature_4,
          temperature_5,
          temperature_6,
          heater_1_power,
          heater_2_power,
          heater_3_power,
          heater_4_power,
          heater_5_power,
          heater_6_power,
          slave_puller_speed,
          slave_tension_arm_angle,
        } = liveValuesEvent.data;
        const timestamp = liveValuesEvent.ts;

        updateStore((state) => {
          const newState = { ...state };

          // Add traverse position if not null
          if (traverse_position !== null) {
            const timeseriesValue: TimeSeriesValue = {
              value: traverse_position,
              timestamp,
            };
            newState.traversePosition = addTraversePosition(
              state.traversePosition,
              timeseriesValue,
            );
          }

          if (spool_progress !== null) {
            const timeseriesValue: TimeSeriesValue = {
              value: spool_progress,
              timestamp,
            };
            newState.spoolProgress = addSpoolProgress(
              state.spoolProgress,
              timeseriesValue,
            );
          }

          // Add puller speed
          const pullerSpeedValue: TimeSeriesValue = {
            value: puller_speed,
            timestamp,
          };
          newState.pullerSpeed = addPullerSpeed(
            state.pullerSpeed,
            pullerSpeedValue,
          );

          // Add spool RPM
          const spoolRpmValue: TimeSeriesValue = {
            value: spool_rpm,
            timestamp,
          };
          newState.spoolRpm = addSpoolRpm(state.spoolRpm, spoolRpmValue);

          // Add tension arm angle
          const tensionArmAngleValue: TimeSeriesValue = {
            value: tension_arm_angle,
            timestamp,
          };
          newState.tensionArmAngle = addTensionArmAngle(
            state.tensionArmAngle,
            tensionArmAngleValue,
          );

          // Add temperature readings from backend
          const temp1Value: TimeSeriesValue = {
            value: temperature_1,
            timestamp,
          };
          newState.temperature1 = addTemperature1(
            state.temperature1,
            temp1Value,
          );

          const temp2Value: TimeSeriesValue = {
            value: temperature_2,
            timestamp,
          };
          newState.temperature2 = addTemperature2(
            state.temperature2,
            temp2Value,
          );

          const temp3Value: TimeSeriesValue = {
            value: temperature_3,
            timestamp,
          };
          newState.temperature3 = addTemperature3(
            state.temperature3,
            temp3Value,
          );

          const temp4Value: TimeSeriesValue = {
            value: temperature_4,
            timestamp,
          };
          newState.temperature4 = addTemperature4(
            state.temperature4,
            temp4Value,
          );

          const temp5Value: TimeSeriesValue = {
            value: temperature_5,
            timestamp,
          };
          newState.temperature5 = addTemperature5(
            state.temperature5,
            temp5Value,
          );

          const temp6Value: TimeSeriesValue = {
            value: temperature_6,
            timestamp,
          };
          newState.temperature6 = addTemperature6(
            state.temperature6,
            temp6Value,
          );

          // Add heater power values
          const heater1Value: TimeSeriesValue = {
            value: heater_1_power,
            timestamp,
          };
          newState.heater1Power = addHeater1Power(
            state.heater1Power,
            heater1Value,
          );

          const heater2Value: TimeSeriesValue = {
            value: heater_2_power,
            timestamp,
          };
          newState.heater2Power = addHeater2Power(
            state.heater2Power,
            heater2Value,
          );

          const heater3Value: TimeSeriesValue = {
            value: heater_3_power,
            timestamp,
          };
          newState.heater3Power = addHeater3Power(
            state.heater3Power,
            heater3Value,
          );

          const heater4Value: TimeSeriesValue = {
            value: heater_4_power,
            timestamp,
          };
          newState.heater4Power = addHeater4Power(
            state.heater4Power,
            heater4Value,
          );

          const heater5Value: TimeSeriesValue = {
            value: heater_5_power,
            timestamp,
          };
          newState.heater5Power = addHeater5Power(
            state.heater5Power,
            heater5Value,
          );

          const heater6Value: TimeSeriesValue = {
            value: heater_6_power,
            timestamp,
          };
          newState.heater6Power = addHeater6Power(
            state.heater6Power,
            heater6Value,
          );

          // Update slave puller data from backend
          const slavePullerValue: TimeSeriesValue = {
            value: slave_puller_speed,
            timestamp,
          };
          newState.slavePullerSpeed = addSlavePullerSpeed(
            state.slavePullerSpeed,
            slavePullerValue,
          );

          const slaveTensionArmValue: TimeSeriesValue = {
            value: slave_tension_arm_angle,
            timestamp,
          };
          newState.slaveTensionArmAngle = addSlaveTensionArmAngle(
            state.slaveTensionArmAngle,
            slaveTensionArmValue,
          );

          return newState;
        });
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (error) {
      console.error(`Unexpected error processing ${eventName} event:`, error);
      throw error;
    }
  };
}

/**
 * Create the Gluetex namespace implementation
 */
const useGluetexNamespaceImplementation =
  createNamespaceHookImplementation<GluetexNamespaceStore>({
    createStore: createGluetexNamespaceStore,
    createEventHandler: gluetexMessageHandler,
  });

/**
 * Hook for a machine-specific Gluetex namespace
 *
 * @example
 * ```tsx
 * function GluetexStatus({ machine }) {
 *   const { state, traversePosition, pullerSpeed } = useGluetexNamespace(machine);
 *
 *   return (
 *     <div>
 *       {state && (
 *         <div>Mode: {state.data.mode_state.mode}</div>
 *       )}
 *     </div>
 *   );
 * }
 * ```
 */
export function useGluetexNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): GluetexNamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useGluetexNamespaceImplementation(namespaceId);
}
