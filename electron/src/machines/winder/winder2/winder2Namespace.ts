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
 * Traverse position event schema
 */
export const traversePositionEventDataSchema = z.object({
  position: z.number().nullable(),
});

/**
 * Traverse state event schema
 */
export const traverseStateEventDataSchema = z.object({
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
 * Puller regulation type enum
 */
export const pullerRegulationSchema = z.enum(["Speed", "Diameter"]);

/**
 * Puller state event schema
 */
export const pullerStateEventDataSchema = z.object({
  regulation: pullerRegulationSchema,
  target_speed: z.number(),
  target_diameter: z.number(),
  forward: z.boolean(),
});

/**
 * Puller speed event schema
 */
export const pullerSpeedEventDataSchema = z.object({
  speed: z.number(),
});

/**
 * Autostop wounded length event schema
 */
export const autostopWoundedLengthEventDataSchema = z.object({
  wounded_length: z.number(),
});

/**
 * Autostop transition state enum
 */
export const autostopTransitionSchema = z.enum(["Standby", "Pull"]);

/**
 * Autostop state event schema
 */
export const autostopStateEventDataSchema = z.object({
  enabled: z.boolean(),
  enabled_alarm: z.boolean(),
  limit: z.number(),
  transition: autostopTransitionSchema,
});

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Standby", "Hold", "Pull", "Wind"]);

/**
 * Mode state event schema
 */
export const modeStateEventDataSchema = z.object({
  mode: modeSchema,
  can_wind: z.boolean(),
});

/**
 * Measurements winding RPM event schema
 */
export const spoolRpmEventDataSchema = z.object({
  rpm: z.number(),
});

/**
 * Spool diameter event schema
 */
export const spoolDiameterEventDataSchema = z.object({
  diameter: z.number(),
});

export const spoolStateEventDataSchema = z.object({
  speed_min: z.number(),
  speed_max: z.number(),
});

/**
 * Measurements tension arm event schema
 */
export const tensionArmAngleEventDataSchema = z.object({
  degree: z.number(),
});

export const tensionArmStateEventDataSchema = z.object({
  zeroed: z.boolean(),
});

/**
 * Spool speed controller regulation mode enum
 */
export const spoolRegulationModeSchema = z.enum(["Adaptive", "MinMax"]);

/**
 * Spool speed controller state event schema
 */
export const spoolSpeedControllerStateEventDataSchema = z.object({
  regulation_mode: spoolRegulationModeSchema,
  minmax_min_speed: z.number(),
  minmax_max_speed: z.number(),
  adaptive_tension_target: z.number(),
  adaptive_radius_learning_rate: z.number(),
  adaptive_max_speed_multiplier: z.number(),
  adaptive_acceleration_factor: z.number(),
  adaptive_deacceleration_urgency_multiplier: z.number(),
});

// ========== Event Schemas with Wrappers ==========

export const traversePositionEventSchema = eventSchema(
  traversePositionEventDataSchema,
);
export const traverseStateEventSchema = eventSchema(
  traverseStateEventDataSchema,
);
export const pullerStateEventSchema = eventSchema(pullerStateEventDataSchema);
export const pullerSpeedEventSchema = eventSchema(pullerSpeedEventDataSchema);
export const autostopWoundedLengthEventSchema = eventSchema(
  autostopWoundedLengthEventDataSchema,
);
export const autostopStateEventSchema = eventSchema(
  autostopStateEventDataSchema,
);
export const modeStateEventSchema = eventSchema(modeStateEventDataSchema);
export const spoolRpmEventSchema = eventSchema(spoolRpmEventDataSchema);
export const spoolDiameterEventSchema = eventSchema(
  spoolDiameterEventDataSchema,
);
export const spoolStateEventSchema = eventSchema(spoolStateEventDataSchema);
export const tensionArmAngleEventSchema = eventSchema(
  tensionArmAngleEventDataSchema,
);
export const tensionArmStateEventSchema = eventSchema(
  tensionArmStateEventDataSchema,
);
export const spoolSpeedControllerStateEventSchema = eventSchema(
  spoolSpeedControllerStateEventDataSchema,
);

// ========== Type Inferences ==========

export type TraversePositionEvent = z.infer<typeof traversePositionEventSchema>;
export type TraverseStateEvent = z.infer<typeof traverseStateEventSchema>;
export type PullerStateEvent = z.infer<typeof pullerStateEventSchema>;
export type PullerSpeedEvent = z.infer<typeof pullerSpeedEventSchema>;
export type AutostopWoundedLengthEvent = z.infer<
  typeof autostopWoundedLengthEventSchema
>;
export type AutostopTransition = z.infer<typeof autostopTransitionSchema>;
export type AutostopStateEvent = z.infer<typeof autostopStateEventSchema>;
export type Mode = z.infer<typeof modeSchema>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;
export type SpoolStateEvent = z.infer<typeof spoolStateEventSchema>;
export type MeasurementsWindingRpmEvent = z.infer<typeof spoolRpmEventSchema>;
export type SpoolDiameterEvent = z.infer<typeof spoolDiameterEventSchema>;
export type MeasurementsTensionArmEvent = z.infer<
  typeof tensionArmAngleEventSchema
>;
export type TensionArmStateEvent = z.infer<typeof tensionArmStateEventSchema>;
export type SpoolRegulationMode = z.infer<typeof spoolRegulationModeSchema>;
export type SpoolSpeedControllerStateEvent = z.infer<
  typeof spoolSpeedControllerStateEventSchema
>;

export type Winder2NamespaceStore = {
  // State events (latest only)
  traverseState: TraverseStateEvent | null;
  pullerState: PullerStateEvent | null;
  autostopState: AutostopStateEvent | null;
  modeState: ModeStateEvent | null;
  tensionArmState: TensionArmStateEvent | null;
  spoolSpeedControllerState: SpoolSpeedControllerStateEvent | null;

  // Metric events (cached for 1 hour)
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  autostopWoundedLength: TimeSeries;
  spoolRpm: TimeSeries;
  spoolDiameter: TimeSeries;
  tensionArmAngle: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;
const { initialTimeSeries: traversePosition, insert: addTraversePosition } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const {
  initialTimeSeries: autostopWoundedLength,
  insert: addAutostopWoundedLength,
} = createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
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
        // State events (latest only)
        traverseState: null,
        pullerState: null,
        autostopState: null,
        modeState: null,
        tensionArmState: null,
        spoolSpeedControllerState: null,

        // Metric events (cached for 1 hour)
        traversePosition,
        pullerSpeed,
        autostopWoundedLength,
        spoolRpm,
        spoolDiameter,
        tensionArmAngle,
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
      // Apply appropriate caching strategy based on event type
      // State events (keep only the latest)
      if (eventName === "TraverseStateEvent") {
        console.log("TraverseStateEvent", event);
        updateStore((state) => ({
          ...state,
          traverseState: event as TraverseStateEvent,
        }));
      } else if (eventName === "PullerStateEvent") {
        console.log("PullerStateEvent", event);
        updateStore((state) => ({
          ...state,
          pullerState: event as PullerStateEvent,
        }));
      } else if (eventName === "AutostopStateEvent") {
        console.log("AutostopStateEvent", event);
        updateStore((state) => ({
          ...state,
          autostopState: event as AutostopStateEvent,
        }));
      } else if (eventName === "ModeStateEvent") {
        console.log("ModeStateEvent", event);
        updateStore((state) => ({
          ...state,
          modeState: event as ModeStateEvent,
        }));
      } else if (eventName === "TensionArmStateEvent") {
        console.log("TensionArmStateEvent", event);
        updateStore((state) => ({
          ...state,
          tensionArmState: event as TensionArmStateEvent,
        }));
      } else if (eventName === "SpoolSpeedControllerStateEvent") {
        console.log("SpoolSpeedControllerStateEvent", event);
        updateStore((state) => ({
          ...state,
          spoolSpeedControllerState: event as SpoolSpeedControllerStateEvent,
        }));
      }
      // Metric events (keep for 1 hour)
      else if (eventName === "TraversePositionEvent") {
        const positionEvent = event as TraversePositionEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: positionEvent.data.position ?? 0,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          traversePosition: addTraversePosition(
            state.traversePosition,
            timeseriesValue,
          ),
        }));
      } else if (eventName === "PullerSpeedEvent") {
        const speedEvent = event as PullerSpeedEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: speedEvent.data.speed,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          pullerSpeed: addPullerSpeed(state.pullerSpeed, timeseriesValue),
        }));
      } else if (eventName === "AutostopWoundedLengthEvent") {
        const woundedEvent = event as AutostopWoundedLengthEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: woundedEvent.data.wounded_length,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          autostopWoundedLength: addAutostopWoundedLength(
            state.autostopWoundedLength,
            timeseriesValue,
          ),
        }));
      } else if (eventName === "SpoolRpmEvent") {
        const rpmEvent = event as MeasurementsWindingRpmEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: rpmEvent.data.rpm,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          spoolRpm: addSpoolRpm(state.spoolRpm, timeseriesValue),
        }));
      } else if (eventName === "SpoolDiameterEvent") {
        const diameterEvent = event as SpoolDiameterEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: diameterEvent.data.diameter,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          spoolDiameter: addSpoolDiameter(state.spoolDiameter, timeseriesValue),
        }));
      } else if (eventName === "TensionArmAngleEvent") {
        const angleEvent = event as MeasurementsTensionArmEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: angleEvent.data.degree,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          tensionArmAngle: addTensionArmAngle(
            state.tensionArmAngle,
            timeseriesValue,
          ),
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
