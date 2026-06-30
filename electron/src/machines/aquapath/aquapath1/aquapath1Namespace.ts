/**
 * @file Aquapath1Namespace.ts
 * @description TypeScript implementation of Aquapath1 namespace with Zod schema validation.
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
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import { Toast } from "@/components/Toast";
import { toast } from "sonner";
import React from "react";

// ========== Event Schema Definitions ==========
/**
 * Mode enum for Aquapath Machine
 */
export const modeSchema = z.enum(["Standby", "Auto"]);

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});
export const tempStateSchema = z.object({
  temperature: z.number(),
  target_temperature: z.number(),
});
/**
 * Cooling states schema
 */
export const tempStatesSchema = z.object({
  left: tempStateSchema,
  right: tempStateSchema,
});

export const flowStateSchema = z.object({
  flow: z.number(),
  should_flow: z.boolean(),
});
export const flowStatesSchema = z.object({
  left: flowStateSchema,
  right: flowStateSchema,
});

export const fanStateSchema = z.object({
  revolutions: z.number(),
  max_revolutions: z.number(),
});
export const fanStatesSchema = z.object({
  right: fanStateSchema,
  left: fanStateSchema,
});

export const coolingModeSchema = z.enum(["Low", "Ramp", "Max"]);
export const coolingModeStateSchema = z.object({
  mode: coolingModeSchema.nullable(),
});
export const coolingModeStatesSchema = z.object({
  right: coolingModeStateSchema,
  left: coolingModeStateSchema,
});

export const toleranceStateSchema = z.object({
  heating: z.number(),
  cooling: z.number(),
});
export const toleranceStatesSchema = z.object({
  right: toleranceStateSchema,
  left: toleranceStateSchema,
});

export const pidStateSchema = z.object({
  kp: z.number(),
  ki: z.number(),
  kd: z.number(),
});

export const pidStatesSchema = z.object({
  right: pidStateSchema,
  left: pidStateSchema,
});

export const thermalSafetyStateSchema = z.object({
  thermal_delay: z.number(),
  cooldown_min_temperature: z.number(),
});

export const thermalSafetyStatesSchema = z.object({
  right: thermalSafetyStateSchema,
  left: thermalSafetyStateSchema,
});
/**
 * Live values event schema (time-series data)
 */
export const liveValuesEventDataSchema = z.object({
  left_flow: z.number(),
  right_flow: z.number(),
  left_temperature: z.number(),
  right_temperature: z.number(),
  left_revolutions: z.number(),
  right_revolutions: z.number(),
  right_power: z.number(),
  left_power: z.number(),
  left_heating: z.boolean().optional().default(false),
  right_heating: z.boolean().optional().default(false),
  left_cooling_mode: coolingModeSchema.nullable().optional().default(null),
  right_cooling_mode: coolingModeSchema.nullable().optional().default(null),
  left_pump_cooldown_active: z.boolean().optional().default(false),
  right_pump_cooldown_active: z.boolean().optional().default(false),
  left_pump_cooldown_remaining: z.number().optional().default(0),
  right_pump_cooldown_remaining: z.number().optional().default(0),
  left_heating_startup_wait_active: z.boolean().optional().default(false),
  right_heating_startup_wait_active: z.boolean().optional().default(false),
  left_heating_startup_wait_remaining: z.number().optional().default(0),
  right_heating_startup_wait_remaining: z.number().optional().default(0),
  left_total_energy: z.number(),
  right_total_energy: z.number(),
});

/**
 * State event schema (consolidated state)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  mode_state: modeStateSchema,
  ambient_temperature_calibration: z.number().optional().default(22),
  flow_states: flowStatesSchema,
  temperature_states: tempStatesSchema,
  fan_states: fanStatesSchema,
  cooling_mode_states: coolingModeStatesSchema,
  tolerance_states: toleranceStatesSchema,
  pid_states: pidStatesSchema,
  thermal_safety_states: thermalSafetyStatesSchema,
});

export const noticeEventDataSchema = z.object({
  title: z.string(),
  message: z.string(),
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);
export const noticeEventSchema = eventSchema(noticeEventDataSchema);

// ========== Type Inferences ==========
export type Mode = z.infer<typeof modeSchema>;
export type ModeState = z.infer<typeof modeStateSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type Aquapath1NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  left_flow: TimeSeries;
  right_flow: TimeSeries;

  left_temperature: TimeSeries;
  right_temperature: TimeSeries;

  left_revolutions: TimeSeries;
  right_revolutions: TimeSeries;

  left_power: TimeSeries;
  right_power: TimeSeries;
  combinedPower: TimeSeries;

  left_total_energy: TimeSeries;
  right_total_energy: TimeSeries;
  totalEnergyKWh: TimeSeries;
  left_heating: boolean;
  right_heating: boolean;
  left_cooling_mode: "Low" | "Ramp" | "Max" | null;
  right_cooling_mode: "Low" | "Ramp" | "Max" | null;
  left_pump_cooldown_active: boolean;
  right_pump_cooldown_active: boolean;
  left_pump_cooldown_remaining: number;
  right_pump_cooldown_remaining: number;
  left_heating_startup_wait_active: boolean;
  right_heating_startup_wait_active: boolean;
  left_heating_startup_wait_remaining: number;
  right_heating_startup_wait_remaining: number;
  targetLeftTemperature: TimeSeries;
  targetRightTemperature: TimeSeries;
};

const { initialTimeSeries: left_temperature, insert: addTemperature1 } =
  createTimeSeries();
const { initialTimeSeries: right_temperature, insert: addTemperature2 } =
  createTimeSeries();
const { initialTimeSeries: left_flow, insert: addFlow1 } = createTimeSeries();
const { initialTimeSeries: right_flow, insert: addFlow2 } = createTimeSeries();
const { initialTimeSeries: left_revolutions, insert: addFan1 } =
  createTimeSeries();
const { initialTimeSeries: right_revolutions, insert: addFan2 } =
  createTimeSeries();
const { initialTimeSeries: left_power, insert: addLeftPower } =
  createTimeSeries();
const { initialTimeSeries: right_power, insert: addRightPower } =
  createTimeSeries();
const { initialTimeSeries: combinedPower, insert: addCombinedPower } =
  createTimeSeries();
const { initialTimeSeries: left_total_energy, insert: addLeftEnergy } =
  createTimeSeries();
const { initialTimeSeries: right_total_energy, insert: addRightEnergy } =
  createTimeSeries();
const { initialTimeSeries: totalEnergyKWh, insert: addTotalEnergyKWh } =
  createTimeSeries();
const {
  initialTimeSeries: targetLeftTemperature,
  insert: addTargetLeftTemperature,
} = createTimeSeries();
const {
  initialTimeSeries: targetRightTemperature,
  insert: addTargetRightTemperature,
} = createTimeSeries();

/**
 * Factory function to create a new Aquapath namespace store
 * @returns A new Zustand store instance for Aquapath namespace
 */
export const createAquapath1NamespaceStore =
  (): StoreApi<Aquapath1NamespaceStore> => {
    return create<Aquapath1NamespaceStore>(() => {
      return {
        state: null,
        defaultState: null,
        left_temperature: left_temperature,
        right_temperature: right_temperature,
        left_flow: left_flow,
        right_flow: right_flow,
        left_revolutions: left_revolutions,
        right_revolutions: right_revolutions,
        right_power: right_power,
        left_power: left_power,
        combinedPower: combinedPower,
        left_total_energy: left_total_energy,
        right_total_energy: right_total_energy,
        totalEnergyKWh: totalEnergyKWh,
        left_heating: false,
        right_heating: false,
        left_cooling_mode: null,
        right_cooling_mode: null,
        left_pump_cooldown_active: false,
        right_pump_cooldown_active: false,
        left_pump_cooldown_remaining: 0,
        right_pump_cooldown_remaining: 0,
        left_heating_startup_wait_active: false,
        right_heating_startup_wait_active: false,
        left_heating_startup_wait_remaining: 0,
        right_heating_startup_wait_remaining: 0,
        targetLeftTemperature: targetLeftTemperature,
        targetRightTemperature: targetRightTemperature,
      };
    });
  };

/**
 * Creates a message handler for Mock1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
 * @returns A message handler function
 */
export function aquapath1MessageHandler(
  store: StoreApi<Aquapath1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Aquapath1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Aquapath1NamespaceStore) => Aquapath1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      // State events (latest only)
      if (eventName === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);
        const timestamp = event.ts;
        const nextTargetLeftTemperature =
          stateEvent.data.temperature_states.left.target_temperature;
        const nextTargetRightTemperature =
          stateEvent.data.temperature_states.right.target_temperature;
        updateStore((state) => ({
          ...state,
          state: stateEvent,
          // only set default state if is_default_state is true
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
          targetLeftTemperature:
            state.targetLeftTemperature.current?.value ===
            nextTargetLeftTemperature
              ? state.targetLeftTemperature
              : addTargetLeftTemperature(state.targetLeftTemperature, {
                  value: nextTargetLeftTemperature,
                  timestamp,
                }),
          targetRightTemperature:
            state.targetRightTemperature.current?.value ===
            nextTargetRightTemperature
              ? state.targetRightTemperature
              : addTargetRightTemperature(state.targetRightTemperature, {
                  value: nextTargetRightTemperature,
                  timestamp,
                }),
        }));
      } else if (eventName === "NoticeEvent") {
        const noticeEvent = noticeEventSchema.parse(event);
        toast.custom(
          () =>
            React.createElement(
              Toast,
              {
                title: noticeEvent.data.title,
                icon: "lu:CircleAlert",
              },
              React.createElement(
                "div",
                { className: "text-zinc-500" },
                noticeEvent.data.message,
              ),
            ),
          {
            duration: 7000,
          },
        );
      }
      // Live values events (time-series data)
      else if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);

        updateStore((state) => ({
          ...state,
          left_temperature: addTemperature1(state.left_temperature, {
            value: liveValuesEvent.data.left_temperature,
            timestamp: event.ts,
          }),
          right_temperature: addTemperature2(state.right_temperature, {
            value: liveValuesEvent.data.right_temperature,
            timestamp: event.ts,
          }),
          left_flow: addFlow1(state.left_flow, {
            value: liveValuesEvent.data.left_flow,
            timestamp: event.ts,
          }),
          right_flow: addFlow2(state.right_flow, {
            value: liveValuesEvent.data.right_flow,
            timestamp: event.ts,
          }),
          left_revolutions: addFan1(state.left_revolutions, {
            value: liveValuesEvent.data.left_revolutions,
            timestamp: event.ts,
          }),
          right_revolutions: addFan2(state.right_revolutions, {
            value: liveValuesEvent.data.right_revolutions,
            timestamp: event.ts,
          }),
          left_power: addLeftPower(state.left_power, {
            value: liveValuesEvent.data.left_power,
            timestamp: event.ts,
          }),
          right_power: addRightPower(state.right_power, {
            value: liveValuesEvent.data.right_power,
            timestamp: event.ts,
          }),
          combinedPower: addCombinedPower(state.combinedPower, {
            value:
              liveValuesEvent.data.left_power +
              liveValuesEvent.data.right_power,
            timestamp: event.ts,
          }),
          left_total_energy: addLeftEnergy(state.left_total_energy, {
            value: liveValuesEvent.data.left_total_energy,
            timestamp: event.ts,
          }),
          right_total_energy: addRightEnergy(state.right_total_energy, {
            value: liveValuesEvent.data.right_total_energy,
            timestamp: event.ts,
          }),
          totalEnergyKWh: addTotalEnergyKWh(state.totalEnergyKWh, {
            value:
              (liveValuesEvent.data.left_total_energy +
                liveValuesEvent.data.right_total_energy) /
              1000,
            timestamp: event.ts,
          }),
          left_heating: liveValuesEvent.data.left_heating,
          right_heating: liveValuesEvent.data.right_heating,
          left_cooling_mode: liveValuesEvent.data.left_cooling_mode,
          right_cooling_mode: liveValuesEvent.data.right_cooling_mode,
          left_pump_cooldown_active:
            liveValuesEvent.data.left_pump_cooldown_active,
          right_pump_cooldown_active:
            liveValuesEvent.data.right_pump_cooldown_active,
          left_pump_cooldown_remaining:
            liveValuesEvent.data.left_pump_cooldown_remaining,
          right_pump_cooldown_remaining:
            liveValuesEvent.data.right_pump_cooldown_remaining,
          left_heating_startup_wait_active:
            liveValuesEvent.data.left_heating_startup_wait_active,
          right_heating_startup_wait_active:
            liveValuesEvent.data.right_heating_startup_wait_active,
          left_heating_startup_wait_remaining:
            liveValuesEvent.data.left_heating_startup_wait_remaining,
          right_heating_startup_wait_remaining:
            liveValuesEvent.data.right_heating_startup_wait_remaining,
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
 * Create the Aquapath1 namespace implementation
 */
const useAquapath1NamespaceImplementation =
  createNamespaceHookImplementation<Aquapath1NamespaceStore>({
    createStore: createAquapath1NamespaceStore,
    createEventHandler: aquapath1MessageHandler,
  });

export function useAquapath1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Aquapath1NamespaceStore {
  // Generate namespace ID from validated machine ID (memoized to keep reference stable)
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useAquapath1NamespaceImplementation(namespaceId);
}
