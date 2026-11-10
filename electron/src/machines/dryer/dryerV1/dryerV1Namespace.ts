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
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import { useMemo } from "react";

const scheduleDaySchema = z.object({
  start_minutes: z.number(),
  stop_minutes: z.number(),
});

export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),

  status: z.number(),
  alarm: z.number(),
  warning: z.number(),
  is_smart: z.boolean(),

  target_temperature: z.number(),
  air_volume: z.number(),
  drying_timer_minutes: z.number(),

  schedule: z.array(scheduleDaySchema).length(7),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;
export type ScheduleDay = z.infer<typeof scheduleDaySchema>;

export const liveValuesEventDataSchema = z.object({
  temp_process: z.number(),
  temp_safety: z.number(),
  temp_regen_in: z.number(),
  temp_regen_out: z.number(),
  temp_fan_inlet: z.number(),
  temp_return_air: z.number(),
  temp_dew_point: z.number(),
  pwm_fan1: z.number(),
  pwm_fan2: z.number(),
  power_process: z.number(),
  power_regen: z.number(),
  remaining_seconds: z.number().nullable(),
});

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export type LiveValues = z.infer<typeof liveValuesEventDataSchema>;

export type DryerV1NamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;
  liveValues: LiveValues | null;
  ts_temp_process: TimeSeries;
  ts_temp_safety: TimeSeries;
  ts_temp_regen_in: TimeSeries;
  ts_temp_regen_out: TimeSeries;
  ts_temp_fan_inlet: TimeSeries;
  ts_temp_return_air: TimeSeries;
  ts_pwm_fan1: TimeSeries;
  ts_pwm_fan2: TimeSeries;
  ts_power_process: TimeSeries;
  ts_power_regen: TimeSeries;
};

const { initialTimeSeries: init_temp_process, insert: add_temp_process } =
  createTimeSeries();
const { initialTimeSeries: init_temp_safety, insert: add_temp_safety } =
  createTimeSeries();
const { initialTimeSeries: init_temp_regen_in, insert: add_temp_regen_in } =
  createTimeSeries();
const { initialTimeSeries: init_temp_regen_out, insert: add_temp_regen_out } =
  createTimeSeries();
const { initialTimeSeries: init_temp_fan_inlet, insert: add_temp_fan_inlet } =
  createTimeSeries();
const {
  initialTimeSeries: init_temp_return_air,
  insert: add_temp_return_air,
} = createTimeSeries();
const { initialTimeSeries: init_pwm_fan1, insert: add_pwm_fan1 } =
  createTimeSeries();
const { initialTimeSeries: init_pwm_fan2, insert: add_pwm_fan2 } =
  createTimeSeries();
const { initialTimeSeries: init_power_process, insert: add_power_process } =
  createTimeSeries();
const { initialTimeSeries: init_power_regen, insert: add_power_regen } =
  createTimeSeries();

export const createDryerV1NamespaceStore =
  (): StoreApi<DryerV1NamespaceStore> =>
    create<DryerV1NamespaceStore>(() => ({
      state: null,
      defaultState: null,
      liveValues: null,
      ts_temp_process: init_temp_process,
      ts_temp_safety: init_temp_safety,
      ts_temp_regen_in: init_temp_regen_in,
      ts_temp_regen_out: init_temp_regen_out,
      ts_temp_fan_inlet: init_temp_fan_inlet,
      ts_temp_return_air: init_temp_return_air,
      ts_pwm_fan1: init_pwm_fan1,
      ts_pwm_fan2: init_pwm_fan2,
      ts_power_process: init_power_process,
      ts_power_regen: init_power_regen,
    }));

export function dryerV1MessageHandler(
  store: StoreApi<DryerV1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<DryerV1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: DryerV1NamespaceStore) => DryerV1NamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);

        updateStore((state) => ({
          ...state,
          state: parsed.data,
          defaultState: parsed.data.is_default_state
            ? parsed.data
            : state.defaultState,
        }));
      } else if (event.name === "LiveValuesEvent") {
        const parsed = liveValuesEventSchema.parse(event);
        const ts = event.ts;
        const d = parsed.data;

        updateStore((state) => ({
          ...state,
          liveValues: d,
          ts_temp_process: add_temp_process(state.ts_temp_process, {
            value: d.temp_process,
            timestamp: ts,
          }),
          ts_temp_safety: add_temp_safety(state.ts_temp_safety, {
            value: d.temp_safety,
            timestamp: ts,
          }),
          ts_temp_regen_in: add_temp_regen_in(state.ts_temp_regen_in, {
            value: d.temp_regen_in,
            timestamp: ts,
          }),
          ts_temp_regen_out: add_temp_regen_out(state.ts_temp_regen_out, {
            value: d.temp_regen_out,
            timestamp: ts,
          }),
          ts_temp_fan_inlet: add_temp_fan_inlet(state.ts_temp_fan_inlet, {
            value: d.temp_fan_inlet,
            timestamp: ts,
          }),
          ts_temp_return_air: add_temp_return_air(state.ts_temp_return_air, {
            value: d.temp_return_air,
            timestamp: ts,
          }),
          ts_pwm_fan1: add_pwm_fan1(state.ts_pwm_fan1, {
            value: d.pwm_fan1,
            timestamp: ts,
          }),
          ts_pwm_fan2: add_pwm_fan2(state.ts_pwm_fan2, {
            value: d.pwm_fan2,
            timestamp: ts,
          }),
          ts_power_process: add_power_process(state.ts_power_process, {
            value: d.power_process,
            timestamp: ts,
          }),
          ts_power_regen: add_power_regen(state.ts_power_regen, {
            value: d.power_regen,
            timestamp: ts,
          }),
        }));
      } else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Error processing ${event.name}:`, error);
      throw error;
    }
  };
}

const useDryerV1NamespaceImplementation =
  createNamespaceHookImplementation<DryerV1NamespaceStore>({
    createStore: createDryerV1NamespaceStore,
    createEventHandler: dryerV1MessageHandler,
  });

export function useDryerV1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): DryerV1NamespaceStore {
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  return useDryerV1NamespaceImplementation(namespaceId);
}
