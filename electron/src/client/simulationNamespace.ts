import { StoreApi, useStore } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  createNamespaceHookImplementation,
  eventSchema,
  Event,
  handleEventValidationError,
  handleUnhandledEventError,
  NamespaceId,
  ThrottledStoreUpdater,
} from "./socketioStore";
import { useRef } from "react";
import { rustEnum } from "@/lib/types";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";

/** One value per heating zone — mirrors the backend's `PerZone<T>`. */
function perZone<T extends z.ZodTypeAny>(value: T) {
  return z.object({
    front: value,
    middle: value,
    back: value,
    nozzle: value,
  });
}

export const zoneTuningSchema = z.object({
  kp: z.number(),
  ki: z.number(),
  kd: z.number(),
});
export type ZoneTuning = z.infer<typeof zoneTuningSchema>;

export const observerPiParamsSchema = z.object({
  kp: z.number(),
  ki: z.number(),
  tau_sensor_s: z.number(),
  tau_filter_s: z.number(),
  lead_max_k: z.number(),
  ff_duty_per_k: z.number(),
  ambient_c: z.number(),
  max_clamp: z.number(),
});
export type ObserverPiParams = z.infer<typeof observerPiParamsSchema>;

/**
 * Mirrors the backend's `StrategyConfig`: `Pid([ZoneTuning; 4])` or
 * `ObserverPi([ObserverPiParams; 4])`, each array in `Zone::port()` order
 * (front, middle, back, nozzle). Used both ways: read from `StateEvent` and
 * sent back in a `SetStrategy` mutation.
 */
export const strategyConfigSchema = z
  .object({
    Pid: z
      .tuple([
        zoneTuningSchema,
        zoneTuningSchema,
        zoneTuningSchema,
        zoneTuningSchema,
      ])
      .optional(),
    ObserverPi: z
      .tuple([
        observerPiParamsSchema,
        observerPiParamsSchema,
        observerPiParamsSchema,
        observerPiParamsSchema,
      ])
      .optional(),
  })
  .check(rustEnum);
export type StrategyConfig = z.infer<typeof strategyConfigSchema>;

export const simulationLiveValuesEventDataSchema = z.object({
  sim_time_s: z.number(),
  sensor_c: perZone(z.number()),
  steel_c: perZone(z.number()),
  band_c: perZone(z.number()),
  duty: perZone(z.number()),
  power_w: perZone(z.number()),
  melt_c: perZone(z.number().nullable()),
  screw_rpm: z.number(),
  throughput_kg_h: z.number(),
});
export type SimulationLiveValuesEventData = z.infer<
  typeof simulationLiveValuesEventDataSchema
>;

export const simulationLiveValuesEventSchema = eventSchema(
  simulationLiveValuesEventDataSchema,
);
export type SimulationLiveValuesEvent = z.infer<
  typeof simulationLiveValuesEventSchema
>;

export const simulationStateEventDataSchema = z.object({
  setpoints_c: perZone(z.number()),
  strategy: strategyConfigSchema,
  screw_rpm: z.number(),
  running: z.boolean(),
  speed: z.number(),
});
export type SimulationStateEventData = z.infer<
  typeof simulationStateEventDataSchema
>;

export const simulationStateEventSchema = eventSchema(
  simulationStateEventDataSchema,
);
export type SimulationStateEvent = z.infer<typeof simulationStateEventSchema>;

export const zoneNameSchema = z.enum(["front", "middle", "back", "nozzle"]);
export type ZoneName = z.infer<typeof zoneNameSchema>;

/**
 * Mirrors the backend's `SimulationMutation` enum. `Play`/`Pause` carry no
 * data but still serialize as `{"Play": {}}` (an empty-struct variant, the
 * same convention `StopPressurePidAutoTune` uses elsewhere) rather than a
 * bare unit-variant string, so every variant shares this one shape.
 */
export const simulationMutationSchema = z
  .object({
    SetSetpoint: z
      .object({ zone: zoneNameSchema, celsius: z.number() })
      .optional(),
    SetAllSetpoints: z
      .tuple([z.number(), z.number(), z.number(), z.number()])
      .optional(),
    SetStrategy: strategyConfigSchema.optional(),
    SetScrewRpm: z.number().optional(),
    SetSpeed: z.number().optional(),
    Play: z.object({}).optional(),
    Pause: z.object({}).optional(),
    Reset: z.object({ initial_c: z.number() }).optional(),
  })
  .check(rustEnum);
export type SimulationMutation = z.infer<typeof simulationMutationSchema>;

export type SimulationNamespaceStore = {
  state: SimulationStateEvent | null;
  liveValues: SimulationLiveValuesEvent | null;

  // Live sensor temperature per zone, for the graph.
  frontTemperature: TimeSeries;
  middleTemperature: TimeSeries;
  backTemperature: TimeSeries;
  nozzleTemperature: TimeSeries;

  // Setpoint history per zone, for the graph's target-line overlay.
  targetFrontTemperature: TimeSeries;
  targetMiddleTemperature: TimeSeries;
  targetBackTemperature: TimeSeries;
  targetNozzleTemperature: TimeSeries;

  frontPower: TimeSeries;
  middlePower: TimeSeries;
  backPower: TimeSeries;
  nozzlePower: TimeSeries;
};

const { initialTimeSeries: frontTemperature, insert: addFrontTemperature } =
  createTimeSeries();
const { initialTimeSeries: middleTemperature, insert: addMiddleTemperature } =
  createTimeSeries();
const { initialTimeSeries: backTemperature, insert: addBackTemperature } =
  createTimeSeries();
const { initialTimeSeries: nozzleTemperature, insert: addNozzleTemperature } =
  createTimeSeries();

const {
  initialTimeSeries: targetFrontTemperature,
  insert: addTargetFrontTemperature,
} = createTimeSeries();
const {
  initialTimeSeries: targetMiddleTemperature,
  insert: addTargetMiddleTemperature,
} = createTimeSeries();
const {
  initialTimeSeries: targetBackTemperature,
  insert: addTargetBackTemperature,
} = createTimeSeries();
const {
  initialTimeSeries: targetNozzleTemperature,
  insert: addTargetNozzleTemperature,
} = createTimeSeries();

const { initialTimeSeries: frontPower, insert: addFrontPower } =
  createTimeSeries();
const { initialTimeSeries: middlePower, insert: addMiddlePower } =
  createTimeSeries();
const { initialTimeSeries: backPower, insert: addBackPower } =
  createTimeSeries();
const { initialTimeSeries: nozzlePower, insert: addNozzlePower } =
  createTimeSeries();

export const createSimulationNamespaceStore =
  (): StoreApi<SimulationNamespaceStore> =>
    create<SimulationNamespaceStore>()(() => ({
      state: null,
      liveValues: null,
      frontTemperature,
      middleTemperature,
      backTemperature,
      nozzleTemperature,
      targetFrontTemperature,
      targetMiddleTemperature,
      targetBackTemperature,
      targetNozzleTemperature,
      frontPower,
      middlePower,
      backPower,
      nozzlePower,
    }));

export function simulationMessageHandler(
  store: StoreApi<SimulationNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<SimulationNamespaceStore>,
): EventHandler {
  const updateStore = (
    updater: (state: SimulationNamespaceStore) => SimulationNamespaceStore,
  ) => throttledUpdater.updateWith(updater);

  return (event: Event<any>) => {
    const eventName = event.name;
    try {
      if (eventName === "StateEvent") {
        const validated = simulationStateEventSchema.parse(event);
        const timestamp = event.ts;
        const targets = validated.data.setpoints_c;
        updateStore((state) => ({
          ...state,
          state: validated,
          targetFrontTemperature:
            state.targetFrontTemperature.current?.value === targets.front
              ? state.targetFrontTemperature
              : addTargetFrontTemperature(state.targetFrontTemperature, {
                  value: targets.front,
                  timestamp,
                }),
          targetMiddleTemperature:
            state.targetMiddleTemperature.current?.value === targets.middle
              ? state.targetMiddleTemperature
              : addTargetMiddleTemperature(state.targetMiddleTemperature, {
                  value: targets.middle,
                  timestamp,
                }),
          targetBackTemperature:
            state.targetBackTemperature.current?.value === targets.back
              ? state.targetBackTemperature
              : addTargetBackTemperature(state.targetBackTemperature, {
                  value: targets.back,
                  timestamp,
                }),
          targetNozzleTemperature:
            state.targetNozzleTemperature.current?.value === targets.nozzle
              ? state.targetNozzleTemperature
              : addTargetNozzleTemperature(state.targetNozzleTemperature, {
                  value: targets.nozzle,
                  timestamp,
                }),
        }));
      } else if (eventName === "LiveValuesEvent") {
        const validated = simulationLiveValuesEventSchema.parse(event);
        const timestamp = event.ts;
        const d = validated.data;
        updateStore((state) => ({
          ...state,
          liveValues: validated,
          frontTemperature: addFrontTemperature(state.frontTemperature, {
            value: d.sensor_c.front,
            timestamp,
          }),
          middleTemperature: addMiddleTemperature(state.middleTemperature, {
            value: d.sensor_c.middle,
            timestamp,
          }),
          backTemperature: addBackTemperature(state.backTemperature, {
            value: d.sensor_c.back,
            timestamp,
          }),
          nozzleTemperature: addNozzleTemperature(state.nozzleTemperature, {
            value: d.sensor_c.nozzle,
            timestamp,
          }),
          frontPower: addFrontPower(state.frontPower, {
            value: d.power_w.front,
            timestamp,
          }),
          middlePower: addMiddlePower(state.middlePower, {
            value: d.power_w.middle,
            timestamp,
          }),
          backPower: addBackPower(state.backPower, {
            value: d.power_w.back,
            timestamp,
          }),
          nozzlePower: addNozzlePower(state.nozzlePower, {
            value: d.power_w.nozzle,
            timestamp,
          }),
        }));
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

export const simulationNamespaceStore: StoreApi<SimulationNamespaceStore> =
  createSimulationNamespaceStore();

const simulationRoomImplementation = createNamespaceHookImplementation({
  createStore: () => simulationNamespaceStore,
  createEventHandler: simulationMessageHandler,
});

export function useSimulationNamespace(): SimulationNamespaceStore;
export function useSimulationNamespace<T>(
  selector: (s: SimulationNamespaceStore) => T,
): T;
export function useSimulationNamespace<T>(
  selector?: (s: SimulationNamespaceStore) => T,
) {
  const namespaceId = useRef({ type: "simulation" } satisfies NamespaceId);
  simulationRoomImplementation(namespaceId.current);
  return useStore(
    simulationNamespaceStore,
    selector ?? ((s) => s as unknown as T),
  );
}
