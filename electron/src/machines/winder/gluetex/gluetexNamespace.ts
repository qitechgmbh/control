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
  connected_machine_state: connectedMachineStateSchema,
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
  forward: boolean;
};

export type MotorRatiosState = {
  stepper3_master: number;
  stepper3_slave: number;
  stepper4_master: number;
  stepper4_slave: number;
};

export type StepperState = {
  stepper2_mode: StepperMode;
  stepper34_mode: StepperMode;
  cutting_unit_mode: StepperMode;
};

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

  // Time series data for addons (local)
  slavePullerSpeed: TimeSeries;
  temperature1: TimeSeries;
  temperature2: TimeSeries;
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

// Create time series for addon values (local)
const { initialTimeSeries: slavePullerSpeed, insert: addSlavePullerSpeed } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: temperature1, insert: addTemperature1 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: temperature2, insert: addTemperature2 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

// Default addon state
const DEFAULT_ADDON_STATE = {
  slave_puller_state: {
    forward: true,
  },
  motor_ratios_state: {
    stepper3_master: 1.0,
    stepper3_slave: 1.0,
    stepper4_master: 1.0,
    stepper4_slave: 1.0,
  },
  stepper_state: {
    stepper2_mode: "Standby" as StepperMode,
    stepper34_mode: "Standby" as StepperMode,
    cutting_unit_mode: "Standby" as StepperMode,
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

        // Time series data for addons
        slavePullerSpeed,
        temperature1,
        temperature2,
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
          // Extend backend state with addon state
          const extendedData: ExtendedStateEventData = {
            ...stateEvent.data,
            // Preserve existing addon state or use defaults
            slave_puller_state:
              state.state?.data.slave_puller_state ||
              DEFAULT_ADDON_STATE.slave_puller_state,
            motor_ratios_state:
              state.state?.data.motor_ratios_state ||
              DEFAULT_ADDON_STATE.motor_ratios_state,
            stepper_state:
              state.state?.data.stepper_state ||
              DEFAULT_ADDON_STATE.stepper_state,
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

          // Simulate addon live values (these would come from backend in the future)
          // For now, generate synthetic data based on puller speed
          const slavePullerValue: TimeSeriesValue = {
            value: puller_speed * 0.95, // Slave runs slightly slower
            timestamp,
          };
          newState.slavePullerSpeed = addSlavePullerSpeed(
            state.slavePullerSpeed,
            slavePullerValue,
          );

          // Simulate temperature readings
          const temp1Value: TimeSeriesValue = {
            value:
              state.state?.data.quality_control_state?.temperature1
                .current_temperature || 85.0,
            timestamp,
          };
          newState.temperature1 = addTemperature1(
            state.temperature1,
            temp1Value,
          );

          const temp2Value: TimeSeriesValue = {
            value:
              state.state?.data.quality_control_state?.temperature2
                .current_temperature || 125.0,
            timestamp,
          };
          newState.temperature2 = addTemperature2(
            state.temperature2,
            temp2Value,
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
