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

// ========== Event Schema Definitions ==========

/**
 * Consolidated live values event schema (30FPS data)
 */
export const liveValuesEventDataSchema = z.object({
  traverse_position: z.number().nullable(),
  puller_speed: z.number(),
  spool_rpm: z.number(),
  tension_arm_angle: z.number(),
  spool_length_task_progress: z.number(),
});

/**
 * Direction enum
 */
export const directionSchema = z.enum(["Forward", "Reverse"]);
export type Direction = z.infer<typeof directionSchema>;

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Standby", "Hold", "Pull", "Wind"]);
export type Mode = z.infer<typeof modeSchema>;

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

export const machineCrossConnectionStateSchema = z.object({
  machine_identification_unique: machineIdentificationUniqueSchema.nullable(),
  is_available: z.boolean(),
});

/**
 * Gear ratio enum for winding speed
 */
export const gearRatioSchema = z.enum(["OneToOne", "FiveToOne", "TenToOne"]);
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
    case "FiveToOne":
      return 5.0;
    case "TenToOne":
      return 10.0;
    default:
      return 1.0;
  }
}

/**
 * Machine operation mode enum
 */
export const spoolLengthTaskCompletedAction = z.enum(["NoAction", "Pull", "Hold"]);
export type OnSpoolLengthTaskCompletedAction = z.infer<typeof spoolLengthTaskCompletedAction>;

/**
 * Spool speed controller regulation mode enum
 */
export const spoolSpeedControlModeSchema = z.enum(["Adaptive", "MinMax"]);
export type SpoolSpeedControlMode = z.infer<typeof spoolSpeedControlModeSchema>;

/**
 * Spool speed controller regulation mode enum
 */
export const pullerSpeedControlModeSchema = z.enum(["Fixed", "Adaptive"]);
export type PullerSpeedControlMode = z.infer<typeof pullerSpeedControlModeSchema>;

/**
 * Spool speed controller state schema
 */
export const spoolStateSchema = z.object({
  direction: directionSchema,
  speed_control_mode: spoolSpeedControlModeSchema,

  // min max mode
  minmax_min_speed: z.number(),
  minmax_max_speed: z.number(),

  // adaptive mode
  adaptive_tension_target: z.number(),
  adaptive_radius_learning_rate: z.number(),
  adaptive_max_speed_multiplier: z.number(),
  adaptive_acceleration_factor: z.number(),
  adaptive_deacceleration_urgency_multiplier: z.number(),
});

/**
 * Puller state schema
 */
export const pullerStateSchema = z.object({
  direction: directionSchema,
  gear_ratio: gearRatioSchema,
  speed_control_mode: pullerSpeedControlModeSchema,
  fixed_target_speed: z.number(),
  adaptive_base_speed: z.number(),
  adaptive_deviation_max: z.number(),
  adaptive_reference_machine: machineCrossConnectionStateSchema,
});

/**
 * Traverse state schema
 */
export const traverseStateSchema = z.object({
  // config
  limit_inner: z.number(),
  limit_outer: z.number(),
  position_in: z.number(),
  position_out: z.number(),
  step_size: z.number(),
  padding: z.number(),

  // states
  is_going_in: z.boolean(),
  is_going_out: z.boolean(),
  is_homed: z.boolean(),
  is_going_home: z.boolean(),
  is_traversing: z.boolean(),

  // state transitions
  can_go_in: z.boolean(),
  can_go_out: z.boolean(),
  can_go_home: z.boolean(),

  // lazeeeeeeeer
  laserpointer_enabled: z.boolean(),
});

/**
 * Tension arm state schema
 */
export const tensionArmStateSchema = z.object({
  is_calibrated: z.boolean(),
});

/**
 * spool length task state schema
 */
export const spoolLengthTaskStateSchema = z.object({
  target_length: z.number(),
  on_completed_action: spoolLengthTaskCompletedAction,
});

export type SpoolLengthTaskState = z.infer<typeof spoolLengthTaskStateSchema>;

/**
 * Consolidated state event schema (state changes only)
 */
export const stateEventDataSchema = z.object({
  // common
  is_default_state: z.boolean(),

  // machine
  mode: modeSchema,
  can_wind: z.boolean(),

  // devices
  spool_state: spoolStateSchema,
  puller_state: pullerStateSchema,
  traverse_state: traverseStateSchema,
  tension_arm_state: tensionArmStateSchema,

  // tasks
  spool_length_task_state: spoolLengthTaskStateSchema,
});

// ========== Event Schemas with Wrappers ==========

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========

export type StateEvent = z.infer<typeof stateEventSchema>;

// Individual type exports for backward compatibility

export type Winder2NamespaceStore = {
  // State event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  spoolRpm:         TimeSeries;
  pullerSpeed:      TimeSeries;
  traversePosition: TimeSeries;
  tensionArmAngle:  TimeSeries;
  spoolLengthTaskProgress: TimeSeries;
};

//Store Factory and Message Handler -> no param, so default values
const { initialTimeSeries: spoolProgress, insert: addSpoolProgress } =
  createTimeSeries();
const { initialTimeSeries: traversePosition, insert: addTraversePosition } =
  createTimeSeries();
const { initialTimeSeries: pullerSpeed, insert: addPullerSpeed } =
  createTimeSeries();
const { initialTimeSeries: targetPullerSpeed, insert: addTargetPullerSpeed } =
  createTimeSeries();
const { initialTimeSeries: spoolRpm, insert: addSpoolRpm } = createTimeSeries();
const { initialTimeSeries: tensionArmAngle, insert: addTensionArmAngle } =
  createTimeSeries();

/**
 * Factory function to create a new Winder2 namespace store
 * @returns A new Zustand store instance for Winder2 namespace
 */
export const createWinder2NamespaceStore =
  (): StoreApi<Winder2NamespaceStore> =>
    create<Winder2NamespaceStore>(() => {
      return {
        // State event from server
        state: null,
        defaultState: null,

        // Time series data for live values
        traversePosition,
        pullerSpeed,
        spoolRpm,
        tensionArmAngle,
        spoolLengthTaskProgress: spoolProgress,
      };
    });
/**
 * @file winder2Namespace.ts (continued)
 */

/**
 * Creates a message handler for Winder2 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function winder2MessageHandler(
  store: StoreApi<Winder2NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Winder2NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Winder2NamespaceStore) => Winder2NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      if (eventName === "State") 
      {
        console.log(event);
        // Parse and validate the state event
        const stateEvent = stateEventSchema.parse(event);

        updateStore((state) => ({
          ...state,
          state: stateEvent,
          // only set default state if is_default_state is true
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
        }));
      } else if (eventName === "LiveValues") {
        // Parse and validate the live values event
        const liveValuesEvent = liveValuesEventSchema.parse(event);

        // Extract values and add to time series
        const {
          traverse_position,
          puller_speed,
          spool_rpm,
          tension_arm_angle,
          spool_length_task_progress: spool_progress,
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

          if (spoolProgress !== null) {
            const timeseriesValue: TimeSeriesValue = {
              value: spool_progress,
              timestamp,
            };
            newState.spoolLengthTaskProgress = addSpoolProgress(
              state.spoolLengthTaskProgress,
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
 * Create the Winder2 namespace implementation
 */
const useWinder2NamespaceImplementation =
  createNamespaceHookImplementation<Winder2NamespaceStore>({
    createStore: createWinder2NamespaceStore,
    createEventHandler: winder2MessageHandler,
  });

/**
 * Hook for a machine-specific Winder2 namespace
 *
 * @example
 * ```tsx
 * function WinderStatus({ machine }) {
 *   const { traverseState, pullerSpeeds } = useWinder2Namespace(machine);
 *
 *   return (
 *     <div>
 *       {traverseState && (
 *         <div>Traverse position: {traverseState.data.position_in}mm</div>
 *       )}
 *       <h3>Recent Speeds</h3>
 *       {pullerSpeeds.map((event, i) => (
 *         <div key={i}>
 *           {new Date(event.ts).toLocaleTimeString()}: {event.data.speed}mm/s
 *         </div>
 *       ))}
 *     </div>
 *   );
 * }
 * ```
 */
export function useWinder2Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Winder2NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useWinder2NamespaceImplementation(namespaceId);
}
