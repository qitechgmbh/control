import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  Event,
  EventHandler,
  NamespaceId,
  ThrottledStoreUpdater,
  createNamespaceHookImplementation,
  eventSchema,
  handleUnhandledEventError,
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  TimeSeries,
  TimeSeriesValue,
  createTimeSeries,
} from "@/lib/timeseries";
import { useMemo } from "react";

export const modeSchema = z.enum(["Standby", "Hold", "Pull", "Rewind"]);
export type Mode = z.infer<typeof modeSchema>;

export const gearRatioSchema = z.enum(["OneToOne", "OneToFive", "OneToTen"]);
export type GearRatio = z.infer<typeof gearRatioSchema>;

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

export const spoolRegulationModeSchema = z.enum(["Adaptive", "MinMax"]);
export type SpoolRegulationMode = z.infer<typeof spoolRegulationModeSchema>;

export const liveValuesEventDataSchema = z.object({
  traverse_position: z.number().nullable(),
  puller_speed: z.number(),
  takeup_spool_rpm: z.number(),
  source_spool_rpm: z.number(),
  takeup_tension_arm_angle: z.number(),
  source_tension_arm_angle: z.number(),
});

export const modeStateSchema = z.object({
  mode: modeSchema,
  can_rewind: z.boolean(),
});

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
  step_size: z.number(),
  padding: z.number(),
});

export const pullerStateSchema = z.object({
  target_speed: z.number(),
  gear_ratio: gearRatioSchema,
});

export const takeupSpoolStateSchema = z.object({
  regulation_mode: spoolRegulationModeSchema,
  minmax_min_speed: z.number(),
  minmax_max_speed: z.number(),
  adaptive_tension_target: z.number(),
  adaptive_radius_learning_rate: z.number(),
  adaptive_max_speed_multiplier: z.number(),
  adaptive_acceleration_factor: z.number(),
  adaptive_deacceleration_urgency_multiplier: z.number(),
});

export const sourceSpoolStateSchema = z.object({
  adaptive_tension_target: z.number(),
});

export const tensionArmStateSchema = z.object({
  zeroed: z.boolean(),
});

export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  mode_state: modeStateSchema,
  traverse_state: traverseStateSchema,
  puller_state: pullerStateSchema,
  takeup_spool_state: takeupSpoolStateSchema,
  source_spool_state: sourceSpoolStateSchema,
  takeup_tension_arm_state: tensionArmStateSchema,
  source_tension_arm_state: tensionArmStateSchema,
});

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventSchema>;

export type RewinderNamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  takeupSpoolRpm: TimeSeries;
  sourceSpoolRpm: TimeSeries;
  takeupTensionArmAngle: TimeSeries;
  sourceTensionArmAngle: TimeSeries;
};

const { initialTimeSeries: traversePosition, insert: addTraversePosition } =
  createTimeSeries();
const { initialTimeSeries: pullerSpeed, insert: addPullerSpeed } =
  createTimeSeries();
const { initialTimeSeries: takeupSpoolRpm, insert: addTakeupSpoolRpm } =
  createTimeSeries();
const { initialTimeSeries: sourceSpoolRpm, insert: addSourceSpoolRpm } =
  createTimeSeries();
const {
  initialTimeSeries: takeupTensionArmAngle,
  insert: addTakeupTensionArmAngle,
} = createTimeSeries();
const {
  initialTimeSeries: sourceTensionArmAngle,
  insert: addSourceTensionArmAngle,
} = createTimeSeries();

export const createRewinderNamespaceStore =
  (): StoreApi<RewinderNamespaceStore> =>
    create<RewinderNamespaceStore>(() => ({
      state: null,
      defaultState: null,
      traversePosition,
      pullerSpeed,
      takeupSpoolRpm,
      sourceSpoolRpm,
      takeupTensionArmAngle,
      sourceTensionArmAngle,
    }));

export function rewinderMessageHandler(
  store: StoreApi<RewinderNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<RewinderNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: RewinderNamespaceStore) => RewinderNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          state: stateEvent,
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
        }));
      } else if (event.name === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const timestamp = liveValuesEvent.ts;
        const {
          traverse_position,
          puller_speed,
          takeup_spool_rpm,
          source_spool_rpm,
          takeup_tension_arm_angle,
          source_tension_arm_angle,
        } = liveValuesEvent.data;

        updateStore((state) => {
          const next = { ...state };

          if (traverse_position !== null) {
            next.traversePosition = addTraversePosition(
              state.traversePosition,
              { value: traverse_position, timestamp },
            );
          }

          const values: Array<[keyof RewinderNamespaceStore, number]> = [
            ["pullerSpeed", puller_speed],
            ["takeupSpoolRpm", takeup_spool_rpm],
            ["sourceSpoolRpm", source_spool_rpm],
            ["takeupTensionArmAngle", takeup_tension_arm_angle],
            ["sourceTensionArmAngle", source_tension_arm_angle],
          ];

          for (const [key, value] of values) {
            const timeseriesValue: TimeSeriesValue = { value, timestamp };
            if (key === "pullerSpeed") {
              next.pullerSpeed = addPullerSpeed(
                state.pullerSpeed,
                timeseriesValue,
              );
            } else if (key === "takeupSpoolRpm") {
              next.takeupSpoolRpm = addTakeupSpoolRpm(
                state.takeupSpoolRpm,
                timeseriesValue,
              );
            } else if (key === "sourceSpoolRpm") {
              next.sourceSpoolRpm = addSourceSpoolRpm(
                state.sourceSpoolRpm,
                timeseriesValue,
              );
            } else if (key === "takeupTensionArmAngle") {
              next.takeupTensionArmAngle = addTakeupTensionArmAngle(
                state.takeupTensionArmAngle,
                timeseriesValue,
              );
            } else if (key === "sourceTensionArmAngle") {
              next.sourceTensionArmAngle = addSourceTensionArmAngle(
                state.sourceTensionArmAngle,
                timeseriesValue,
              );
            }
          }

          return next;
        });
      } else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Unexpected error processing ${event.name} event:`, error);
      throw error;
    }
  };
}

const useRewinderNamespaceImplementation =
  createNamespaceHookImplementation<RewinderNamespaceStore>({
    createStore: createRewinderNamespaceStore,
    createEventHandler: rewinderMessageHandler,
  });

export function useRewinderNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): RewinderNamespaceStore {
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  return useRewinderNamespaceImplementation(namespaceId);
}
