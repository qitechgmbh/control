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

export const scheduleDaySchema = z.object({
  start_time: z.number(),
  stop_time: z.number(),
});

export type ScheduleDay = z.infer<typeof scheduleDaySchema>;

export const smartTimerEntrySchema = z.object({
  weekly: z.boolean(),
  weekday: z.number(),
  hour_min: z.number(),
  year: z.number(),
  month_day: z.number(),
  enabled: z.boolean(),
  is_stop: z.boolean(),
});

export type SmartTimerEntry = z.infer<typeof smartTimerEntrySchema>;

export const smartDataSchema = z.object({
  sw_major: z.number(),
  sw_middle: z.number(),
  sw_minor: z.number(),
  timer_enabled: z.boolean(),
  timer_entries: z.array(smartTimerEntrySchema),
});

export type SmartData = z.infer<typeof smartDataSchema>;

export const liveValuesEventDataSchema = z.object({
  status: z.number(),
  alarm: z.number(),
  warning: z.number(),
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
  target_temperature: z.number(),
  schedule: z.array(scheduleDaySchema).length(7),
  smart_data: smartDataSchema,
});

export type LiveValuesEventData = z.infer<typeof liveValuesEventDataSchema>;

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;

export type DryerSmartNamespaceStore = {
  liveValues: LiveValuesEvent | null;
  ts_temp_process: TimeSeries;
  ts_temp_regen_in: TimeSeries;
  ts_temp_regen_out: TimeSeries;
  ts_temp_fan_inlet: TimeSeries;
  ts_temp_safety: TimeSeries;
  ts_temp_return_air: TimeSeries;
  ts_power_process: TimeSeries;
  ts_power_regen: TimeSeries;
  ts_pwm_fan1: TimeSeries;
  ts_pwm_fan2: TimeSeries;
};

const { initialTimeSeries: init_temp_process, insert: add_temp_process } =
  createTimeSeries();
const { initialTimeSeries: init_temp_regen_in, insert: add_temp_regen_in } =
  createTimeSeries();
const { initialTimeSeries: init_temp_regen_out, insert: add_temp_regen_out } =
  createTimeSeries();
const { initialTimeSeries: init_temp_fan_inlet, insert: add_temp_fan_inlet } =
  createTimeSeries();
const { initialTimeSeries: init_temp_safety, insert: add_temp_safety } =
  createTimeSeries();
const {
  initialTimeSeries: init_temp_return_air,
  insert: add_temp_return_air,
} = createTimeSeries();
const { initialTimeSeries: init_power_process, insert: add_power_process } =
  createTimeSeries();
const { initialTimeSeries: init_power_regen, insert: add_power_regen } =
  createTimeSeries();
const { initialTimeSeries: init_pwm_fan1, insert: add_pwm_fan1 } =
  createTimeSeries();
const { initialTimeSeries: init_pwm_fan2, insert: add_pwm_fan2 } =
  createTimeSeries();

export function dryerSmartMessageHandler(
  _store: StoreApi<DryerSmartNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<DryerSmartNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;
    try {
      if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const ts = event.ts;
        const d = liveValuesEvent.data;
        throttledUpdater.updateWith((state) => ({
          ...state,
          liveValues: liveValuesEvent,
          ts_temp_process: add_temp_process(state.ts_temp_process, {
            value: d.temp_process,
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
          ts_temp_safety: add_temp_safety(state.ts_temp_safety, {
            value: d.temp_safety,
            timestamp: ts,
          }),
          ts_temp_return_air: add_temp_return_air(state.ts_temp_return_air, {
            value: d.temp_return_air,
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
          ts_pwm_fan1: add_pwm_fan1(state.ts_pwm_fan1, {
            value: d.pwm_fan1,
            timestamp: ts,
          }),
          ts_pwm_fan2: add_pwm_fan2(state.ts_pwm_fan2, {
            value: d.pwm_fan2,
            timestamp: ts,
          }),
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

export const createDryerSmartNamespaceStore =
  (): StoreApi<DryerSmartNamespaceStore> =>
    create<DryerSmartNamespaceStore>(() => ({
      liveValues: null,
      ts_temp_process: init_temp_process,
      ts_temp_regen_in: init_temp_regen_in,
      ts_temp_regen_out: init_temp_regen_out,
      ts_temp_fan_inlet: init_temp_fan_inlet,
      ts_temp_safety: init_temp_safety,
      ts_temp_return_air: init_temp_return_air,
      ts_power_process: init_power_process,
      ts_power_regen: init_power_regen,
      ts_pwm_fan1: init_pwm_fan1,
      ts_pwm_fan2: init_pwm_fan2,
    }));

const useDryerSmartNamespaceImplementation =
  createNamespaceHookImplementation<DryerSmartNamespaceStore>({
    createStore: createDryerSmartNamespaceStore,
    createEventHandler: dryerSmartMessageHandler,
  });

export function useDryerSmartNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): DryerSmartNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };
  return useDryerSmartNamespaceImplementation(namespaceId);
}
