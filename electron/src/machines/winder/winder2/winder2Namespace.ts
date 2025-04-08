/**
 * @file Winder1Room.ts
 * @description TypeScript implementation of Winder1 namespace with Zod schema validation.
 */

import { StoreApi } from "zustand";
import { create } from "zustand";
import { produce } from "immer";
import { number, z } from "zod";
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

// ========== Event Schema Definitions ==========

/**
 * Traverse position event schema
 */
export const traversePositionEventDataSchema = z.object({
  position: z.number(),
});

/**
 * Traverse state event schema
 */
export const traverseStateEventDataSchema = z.object({
  limit_inner: z.number(),
  limit_outer: z.number(),
  position_in: z.number(),
  position_out: z.number(),
  is_in: z.boolean(),
  is_out: z.boolean(),
  is_going_in: z.boolean(),
  is_going_out: z.boolean(),
  is_homed: z.boolean(),
  is_going_home: z.boolean(),
  laserpointer: z.boolean(),
});

/**
 * Puller regulation type enum
 */
export const pullerRegulationSchema = z.enum(["Speed", "Diameter"]);

/**
 * Puller state event schema
 */
export const pullerStateEventDataSchema = z.object({
  speed: z.number(),
  regulation: pullerRegulationSchema,
  target_speed: z.number(),
  target_diameter: z.number(),
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
export const autostopTransitionSchema = z.enum(["None", "Pending", "Active"]);

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
});

/**
 * Measurements winding RPM event schema
 */
export const measurementsWindingRpmEventDataSchema = z.object({
  rpm: z.number(),
});

/**
 * Measurements tension arm event schema
 */
export const measurementsTensionArmEventDataSchema = z.object({
  degree: z.number(),
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
export const measurementsWindingRpmEventSchema = eventSchema(
  measurementsWindingRpmEventDataSchema,
);
export const measurementsTensionArmEventSchema = eventSchema(
  measurementsTensionArmEventDataSchema,
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
export type MeasurementsWindingRpmEvent = z.infer<
  typeof measurementsWindingRpmEventDataSchema
>;
export type MeasurementsTensionArmEvent = z.infer<
  typeof measurementsTensionArmEventDataSchema
>;

export type Winder1NamespaceStore = {
  // State events (latest only)
  traverseState: TraverseStateEvent | null;
  pullerState: PullerStateEvent | null;
  autostopState: AutostopStateEvent | null;
  modeState: ModeStateEvent | null;

  // Metric events (cached for 1 hour)
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  autostopWoundedLength: TimeSeries;
  measurementsWindingRpm: TimeSeries;
  measurementsTensionArm: TimeSeries;
};

// Constants for time durations
const ONE_SECOND = 1000;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: traversePosition, insert: addTraversePosition } =
  createTimeSeries(ONE_SECOND, ONE_HOUR);
const {
  initialTimeSeries: autostopWoundedLength,
  insert: addAutostopWoundedLength,
} = createTimeSeries(ONE_SECOND, ONE_HOUR);
const { initialTimeSeries: pullerSpeed, insert: addPullerSpeed } =
  createTimeSeries(ONE_SECOND, ONE_HOUR);
const {
  initialTimeSeries: measurementsWindingRpm,
  insert: addMeasurementsWindingRpm,
} = createTimeSeries(ONE_SECOND, ONE_HOUR);
const {
  initialTimeSeries: measurementsTensionArm,
  insert: addMeasurementsTensionArm,
} = createTimeSeries(ONE_SECOND, ONE_HOUR);

/**
 * Factory function to create a new Winder1 namespace store
 * @returns A new Zustand store instance for Winder1 namespace
 */
export const createWinder1NamespaceStore =
  (): StoreApi<Winder1NamespaceStore> =>
    create<Winder1NamespaceStore>((set) => {
      return {
        // State events (latest only)
        traverseState: null,
        pullerState: null,
        autostopState: null,
        modeState: null,

        // Metric events (cached for 1 hour)
        traversePosition,
        pullerSpeed,
        autostopWoundedLength,
        measurementsWindingRpm,
        measurementsTensionArm,
      };
    });
/**
 * @file Winder1Room.ts (continued)
 */

/**
 * Creates a message handler for Winder1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @returns A message handler function
 */
export function winder2MessageHandler(
  store: StoreApi<Winder1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    try {
      // Apply appropriate caching strategy based on event type
      // State events (keep only the latest)
      if (eventName === "TraverseStateEvent") {
        store.setState(
          produce((state) => {
            state.traverseState = traverseStateEventSchema.parse(event);
          }),
        );
      } else if (eventName === "PullerStateEvent") {
        store.setState(
          produce((state) => {
            state.pullerState = pullerStateEventSchema.parse(event);
          }),
        );
      } else if (eventName === "AutostopStateEvent") {
        store.setState(
          produce((state) => {
            state.autostopState = autostopStateEventSchema.parse(event);
          }),
        );
      } else if (eventName === "ModeStateEvent") {
        store.setState(
          produce((state) => {
            state.modeState = modeStateEventSchema.parse(event);
          }),
        );
      }
      // Metric events (keep for 1 hour)
      else if (eventName === "TraversePositionEvent") {
        let parsed = traversePositionEventSchema.parse(event);
        let timeseriesValue: TimeSeriesValue = {
          value: parsed.data.position,
          timestamp: event.ts,
        };
        store.setState(
          produce((state) => {
            state.traversePosition = addTraversePosition(
              state.traversePosition,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName === "PullerSpeedEvent") {
        let parsed = pullerSpeedEventSchema.parse(event);
        let timeseriesValue: TimeSeriesValue = {
          value: parsed.data.speed,
          timestamp: event.ts,
        };
        store.setState(
          produce((state) => {
            state.pullerSpeed = addPullerSpeed(
              state.pullerSpeed,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName === "AutostopWoundedlengthEvent") {
        let parsed = autostopWoundedLengthEventSchema.parse(event);
        let timeseriesValue: TimeSeriesValue = {
          value: parsed.data.wounded_length,
          timestamp: event.ts,
        };
        store.setState(
          produce((state) => {
            state.autostopWoundedLength = addAutostopWoundedLength(
              state.autostopWoundedLength,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName === "MeasurementsWindingRpmEvent") {
        let parsed = measurementsWindingRpmEventSchema.parse(event);
        let timeseriesValue: TimeSeriesValue = {
          value: parsed.data.rpm,
          timestamp: event.ts,
        };
        store.setState(
          produce((state) => {
            state.measurementsWindingRpm = addMeasurementsWindingRpm(
              state.measurementsWindingRpm,
              timeseriesValue,
            );
          }),
        );
      } else if (eventName === "MeasurementsTensionArmEvent") {
        let parsed = measurementsTensionArmEventSchema.parse(event);
        let timeseriesValue: TimeSeriesValue = {
          value: parsed.data.degree,
          timestamp: event.ts,
        };
        store.setState(
          produce((state) => {
            state.measurementsTensionArm = addMeasurementsTensionArm(
              state.measurementsTensionArm,
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
 * Create the Winder1 namespace implementation
 */
const useWinder2NamespaceImplementation =
  createNamespaceHookImplementation<Winder1NamespaceStore>({
    createStore: createWinder1NamespaceStore,
    createEventHandler: winder2MessageHandler,
  });

/**
 * Hook for a machine-specific Winder1 namespace
 *
 * @example
 * ```tsx
 * function WinderStatus({ machine }) {
 *   const { traverseState, pullerSpeeds } = useWinder1Namespace(machine);
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
): Winder1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useRef<NamespaceId>({
    type: "machine",
    vendor: machine_identification_unique.vendor,
    serial: machine_identification_unique.serial,
    machine: machine_identification_unique.machine,
  });

  // Use the implementation with validated namespace ID
  return useWinder2NamespaceImplementation(namespaceId.current);
}
