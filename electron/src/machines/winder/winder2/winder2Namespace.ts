/**
 * @file winder2Namespace.ts
 * @description TypeScript implementation of Winder2 namespace with Zod schema validation.
 */

import { StoreApi } from "zustand";
import { create } from "zustand";
import { produce } from "immer";
import { z } from "zod";
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
export const spoolStateEventSchema = eventSchema(spoolStateEventDataSchema);
export const tensionArmAngleEventSchema = eventSchema(
  tensionArmAngleEventDataSchema,
);
export const tensionArmStateEventSchema = eventSchema(
  tensionArmStateEventDataSchema,
);

// ========== Type Inferences ==========

export type TraversePositionEvent = z.infer<
  typeof traversePositionEventDataSchema
>;
export type TraverseStateEvent = z.infer<typeof traverseStateEventSchema>;
export type PullerStateEvent = z.infer<typeof pullerStateEventSchema>;
export type PullerSpeedEvent = z.infer<typeof pullerSpeedEventSchema>;
export type AutostopWoundedLengthEvent = z.infer<
  typeof autostopWoundedLengthEventDataSchema
>;
export type AutostopTransition = z.infer<typeof autostopTransitionSchema>;
export type AutostopStateEvent = z.infer<typeof autostopStateEventSchema>;
export type Mode = z.infer<typeof modeSchema>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;
export type SpoolStateEvent = z.infer<typeof spoolStateEventSchema>;
export type MeasurementsWindingRpmEvent = z.infer<
  typeof spoolRpmEventDataSchema
>;
export type MeasurementsTensionArmEvent = z.infer<
  typeof tensionArmAngleEventDataSchema
>;
export type TensionArmStateEvent = z.infer<typeof tensionArmStateEventSchema>;

export type Winder2NamespaceStore = {
  // State events (latest only)
  traverseState: TraverseStateEvent | null;
  pullerState: PullerStateEvent | null;
  autostopState: AutostopStateEvent | null;
  modeState: ModeStateEvent | null;
  tensionArmState: TensionArmStateEvent | null;

  // Metric events (cached for 1 hour)
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  autostopWoundedLength: TimeSeries;
  spoolRpm: TimeSeries;
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

        // Metric events (cached for 1 hour)
        traversePosition,
        pullerSpeed,
        autostopWoundedLength,
        spoolRpm,
        tensionArmAngle,
      };
    });
/**
 * @file winder2Namespace.ts (continued)
 */

/**
 * Creates a message handler for Winder2 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @returns A message handler function
 */
export function winder2MessageHandler(
  store: StoreApi<Winder2NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    try {
      // Apply appropriate caching strategy based on event type
      // State events (keep only the latest)
      if (eventName === "TraverseStateEvent") {
        const parsed = traverseStateEventSchema.parse(event);
        console.log("TraverseStateEvent", parsed);
        store.setState(
          produce(store.getState(), (state) => {
            state.traverseState = parsed;
          }),
        );
      } else if (eventName === "PullerStateEvent") {
        const parsed = pullerStateEventSchema.parse(event);
        console.log("PullerStateEvent", parsed);
        store.setState(
          produce(store.getState(), (state) => {
            state.pullerState = parsed;
          }),
        );
      } else if (eventName === "AutostopStateEvent") {
        const parsed = autostopStateEventSchema.parse(event);
        console.log("AutostopStateEvent", parsed);
        store.setState(
          produce(store.getState(), (state) => {
            state.autostopState = parsed;
          }),
        );
      } else if (eventName === "ModeStateEvent") {
        const parsed = modeStateEventSchema.parse(event);
        console.log("ModeStateEvent", parsed);
        store.setState(
          produce(store.getState(), (state) => {
            state.modeState = parsed;
          }),
        );
      } else if (eventName === "TensionArmStateEvent") {
        const parsed = tensionArmStateEventSchema.parse(event);
        console.log("TensionArmStateEvent", parsed);
        store.setState(
          produce(store.getState(), (state) => {
            state.tensionArmState = parsed;
          }),
        );
      }
      // Metric events (keep for 1 hour)
      else if (eventName === "TraversePositionEvent") {
        const parsed = traversePositionEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.position ?? 0,
          timestamp: event.ts,
        };
        store.setState(
          produce(store.getState(), (state) => {
            state.traversePosition = addTraversePosition(
              state.traversePosition,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName === "PullerSpeedEvent") {
        const parsed = pullerSpeedEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.speed,
          timestamp: event.ts,
        };
        store.setState(
          produce(store.getState(), (state) => {
            state.pullerSpeed = addPullerSpeed(
              state.pullerSpeed,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName === "AutostopWoundedLengthEvent") {
        const parsed = autostopWoundedLengthEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.wounded_length,
          timestamp: event.ts,
        };
        store.setState(
          produce(store.getState(), (state) => {
            state.autostopWoundedLength = addAutostopWoundedLength(
              state.autostopWoundedLength,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName === "SpoolRpmEvent") {
        const parsed = spoolRpmEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.rpm,
          timestamp: event.ts,
        };
        store.setState(
          produce(store.getState(), (state) => {
            state.spoolRpm = addSpoolRpm(state.spoolRpm, timeseriesValue);
          }),
        );
      } else if (eventName === "TensionArmAngleEvent") {
        const parsed = tensionArmAngleEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.degree,
          timestamp: event.ts,
        };
        store.setState(
          produce(store.getState(), (state) => {
            state.tensionArmAngle = addTensionArmAngle(
              state.tensionArmAngle,
              timeseriesValue,
            );
          }),
        );
      } else {
        handleUnhandledEventError(eventName);
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
