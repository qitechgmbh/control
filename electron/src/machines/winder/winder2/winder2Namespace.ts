/**
 * @file winder2Namespace.ts
 * @description TypeScript implementation of Winder2 namespace with Zod schema validation.
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

// ========== Event Schema Definitions ==========

/**
 * Consolidated live values event schema (60FPS data)
 */
export const liveValuesEventDataSchema = z.object({
  traverse_position: z.number().nullable(),
  puller_speed: z.number(),
  spool_rpm: z.number(),
  spool_diameter: z.number(),
  tension_arm_angle: z.number(),
  puller_progress: z.number(),
});

/**
 * Puller regulation type enum
 */
export const pullerRegulationSchema = z.enum(["Speed", "Diameter"]);
export type PullerRegulation = z.infer<typeof pullerRegulationSchema>;

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Standby", "Hold", "Pull", "Wind"]);
export type Mode = z.infer<typeof modeSchema>;

/**
 * Machine operation mode enum
 */
export const spoolAutomaticActionModeSchema = z.enum([
  "Disabled",
  "Pull",
  "Stop",
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
});

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
  can_wind: z.boolean(),
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
});

/**
 * Consolidated state event schema (state changes only)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  traverse_state: traverseStateSchema,
  puller_state: pullerStateSchema,
  mode_state: modeStateSchema,
  tension_arm_state: tensionArmStateSchema,
  spool_speed_controller_state: spoolSpeedControllerStateSchema,
  spool_automatic_action_state: spoolAutomaticActionStateSchema,
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
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  spoolRpm: TimeSeries;
  spoolDiameter: TimeSeries;
  tensionArmAngle: TimeSeries;
  pullerProgress: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: pullerProgress, insert: addPullerProgress } =
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
const { initialTimeSeries: spoolDiameter, insert: addSpoolDiameter } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: tensionArmAngle, insert: addTensionArmAngle } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

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
        spoolDiameter,
        tensionArmAngle,
        pullerProgress,
      };
    });
/**
 * @file winder2Namespace.ts (continued)
 */

/**
 * Creates a message handler for Winder2 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
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
      if (eventName === "StateEvent") {
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
      } else if (eventName === "LiveValuesEvent") {
        // Parse and validate the live values event
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        console.log("LiveValuesEvent", liveValuesEvent);

        // Extract values and add to time series
        const {
          traverse_position,
          puller_speed,
          spool_rpm,
          spool_diameter,
          tension_arm_angle,
          puller_progress,
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

          if (pullerProgress !== null) {
            const timeseriesValue: TimeSeriesValue = {
              value: puller_progress,
              timestamp,
            };
            newState.pullerProgress = addPullerProgress(
              state.pullerProgress,
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

          // Add spool diameter
          const spoolDiameterValue: TimeSeriesValue = {
            value: spool_diameter,
            timestamp,
          };
          newState.spoolDiameter = addSpoolDiameter(
            state.spoolDiameter,
            spoolDiameterValue,
          );

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
