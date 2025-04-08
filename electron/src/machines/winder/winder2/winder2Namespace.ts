/**
 * @file Winder1Room.ts
 * @description TypeScript implementation of Winder1 namespace with Zod schema validation.
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
import { useRef } from "react";

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
export type TraverseStateEvent = z.infer<typeof traverseStateEventDataSchema>;
export type PullerRegulation = z.infer<typeof pullerRegulationSchema>;
export type PullerStateEvent = z.infer<typeof pullerStateEventDataSchema>;
export type PullerSpeedEvent = z.infer<typeof pullerSpeedEventDataSchema>;
export type AutostopWoundedLengthEvent = z.infer<
  typeof autostopWoundedLengthEventDataSchema
>;
export type AutostopTransition = z.infer<typeof autostopTransitionSchema>;
export type AutostopStateEvent = z.infer<typeof autostopStateEventDataSchema>;
export type Mode = z.infer<typeof modeSchema>;
export type ModeStateEvent = z.infer<typeof modeStateEventDataSchema>;
export type MeasurementsWindingRpmEvent = z.infer<
  typeof measurementsWindingRpmEventDataSchema
>;
export type MeasurementsTensionArmEvent = z.infer<
  typeof measurementsTensionArmEventDataSchema
>;

// ========== Store Schema Definition ==========

/**
 * Winder1 namespace store schema
 */
export const Winder1NamespaceStoreSchema = z.object({
  // State events (latest only)
  traverseState: traverseStateEventSchema.nullable(),
  pullerState: pullerStateEventSchema.nullable(),
  autostopState: autostopStateEventSchema.nullable(),
  modeState: modeStateEventSchema.nullable(),

  // Metric events (cached for 1 hour)
  traversePositions: z.array(traversePositionEventSchema),
  pullerSpeeds: z.array(pullerSpeedEventSchema),
  autostopWoundedLengths: z.array(autostopWoundedLengthEventSchema),
  measurementsWindingRpms: z.array(measurementsWindingRpmEventSchema),
  measurementsTensionArms: z.array(measurementsTensionArmEventSchema),
});

export type Winder1NamespaceStore = z.infer<typeof Winder1NamespaceStoreSchema>;

/**
 * Factory function to create a new Winder1 namespace store
 * @returns A new Zustand store instance for Winder1 namespace
 */
export const createWinder1NamespaceStore =
  (): StoreApi<Winder1NamespaceStore> =>
    create<Winder1NamespaceStore>(() => ({
      // State events (latest only)
      traverseState: null,
      pullerState: null,
      autostopState: null,
      modeState: null,

      // Metric events (cached for 1 hour)
      traversePositions: [],
      pullerSpeeds: [],
      autostopWoundedLengths: [],
      measurementsWindingRpms: [],
      measurementsTensionArms: [],
    }));

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
  // Constants for time durations
  const ONE_HOUR = 60 * 60 * 1000;

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
      // // Metric events (keep for 1 hour)
      // else if (eventName === "TraversePositionEvent") {
      //   store.setState(
      //     produce((state) => {
      //       state.traversePositions = EventCache.timeWindow(
      //         state.traversePositions,
      //         traversePositionEventSchema.parse(event),
      //         ONE_HOUR,
      //       );
      //     }),
      //   );
      // } else if (eventName === "PullerSpeedEvent") {
      //   store.setState(
      //     produce((state) => {
      //       state.pullerSpeeds = EventCache.timeWindow(
      //         state.pullerSpeeds,
      //         pullerSpeedEventSchema.parse(event),
      //         ONE_HOUR,
      //       );
      //     }),
      //   );
      // } else if (eventName === "AutostopWoundedlengthEvent") {
      //   store.setState(
      //     produce((state) => {
      //       state.autostopWoundedLengths = EventCache.timeWindow(
      //         state.autostopWoundedLengths,
      //         autostopWoundedLengthEventSchema.parse(event),
      //         ONE_HOUR,
      //       );
      //     }),
      //   );
      // } else if (eventName === "MeasurementsWindingRpmEvent") {
      //   store.setState(
      //     produce((state) => {
      //       state.measurementsWindingRpms = EventCache.timeWindow(
      //         state.measurementsWindingRpms,
      //         measurementsWindingRpmEventSchema.parse(event),
      //         ONE_HOUR,
      //       );
      //     }),
      //   );
      // } else if (eventName === "MeasurementsTensionArmEvent") {
      //   store.setState(
      //     produce((state) => {
      //       state.measurementsTensionArms = EventCache.timeWindow(
      //         state.measurementsTensionArms,
      //         measurementsTensionArmEventSchema.parse(event),
      //         ONE_HOUR,
      //       );
      //     }),
      //   );
      // }
      else {
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
const useWinder1NamespaceImplementation =
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
export function useWinder1Namespace(
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
  return useWinder1NamespaceImplementation(namespaceId.current);
}
