import { describe, expect, it } from "vitest";
import type { ThrottledStoreUpdater } from "@/client/socketioStore";
import {
  createGluetexNamespaceStore,
  gluetexMessageHandler,
  getGearRatioMultiplier,
  type ExtendedStateEventData,
  type GluetexNamespaceStore,
} from "@/machines/winder/gluetex/state/gluetexNamespace";

type TestUpdater = ThrottledStoreUpdater<GluetexNamespaceStore>;

const createBufferedUpdater = (
  store: ReturnType<typeof createGluetexNamespaceStore>,
): { updater: TestUpdater; flush: () => void } => {
  let buffer = store.getState();

  const updater = {
    updateWith: (
      stateUpdater: (state: GluetexNamespaceStore) => GluetexNamespaceStore,
    ) => {
      buffer = stateUpdater(buffer);
    },
    forceSync: () => {
      store.setState(buffer);
    },
  } as unknown as TestUpdater;

  return {
    updater,
    flush: () => updater.forceSync(),
  };
};

const buildStateEventData = (
  overrides: Partial<ExtendedStateEventData> = {},
): ExtendedStateEventData => {
  const store = createGluetexNamespaceStore();
  const defaultState = store.getState().defaultState;

  if (!defaultState) {
    throw new Error("Expected Gluetex default state to be present");
  }

  return {
    ...structuredClone(defaultState.data),
    ...overrides,
  };
};

describe("gluetexNamespace", () => {
  it("maps gear ratio multipliers consistently", () => {
    expect(getGearRatioMultiplier("OneToOne")).toBe(1);
    expect(getGearRatioMultiplier("OneToFive")).toBe(5);
    expect(getGearRatioMultiplier("OneToTen")).toBe(10);
    expect(getGearRatioMultiplier(undefined)).toBe(1);
  });

  it("reflects backend safety monitor states without frontend reinterpretation", () => {
    const store = createGluetexNamespaceStore();
    const { updater, flush } = createBufferedUpdater(store);
    const handleMessage = gluetexMessageHandler(store, updater);

    handleMessage({
      name: "StateEvent",
      ts: 111,
      data: buildStateEventData({
        mode_state: {
          mode: "Hold",
          operation_mode: "Production",
          can_wind: false,
        },
        winder_tension_arm_monitor_state: {
          enabled: true,
          min_angle: 25,
          max_angle: 75,
          triggered: true,
        },
        optris_1_monitor_state: {
          enabled: true,
          min_voltage: 2.2,
          max_voltage: 7.8,
          delay_mm: 10,
          triggered: true,
        },
      }),
    });

    flush();

    const current = store.getState().state;
    expect(current?.data.mode_state.can_wind).toBe(false);
    expect(current?.data.winder_tension_arm_monitor_state).toEqual({
      enabled: true,
      min_angle: 25,
      max_angle: 75,
      triggered: true,
    });
    expect(current?.data.optris_1_monitor_state).toEqual({
      enabled: true,
      min_voltage: 2.2,
      max_voltage: 7.8,
      delay_mm: 10,
      triggered: true,
    });
  });

  it("does not compute trigger/stop decisions from live boundary values on the frontend", () => {
    const store = createGluetexNamespaceStore();
    const { updater, flush } = createBufferedUpdater(store);
    const handleMessage = gluetexMessageHandler(store, updater);

    handleMessage({
      name: "StateEvent",
      ts: 222,
      data: buildStateEventData({
        winder_tension_arm_monitor_state: {
          enabled: true,
          min_angle: 20,
          max_angle: 90,
          triggered: false,
        },
        optris_1_monitor_state: {
          enabled: true,
          min_voltage: 2,
          max_voltage: 8,
          delay_mm: 0,
          triggered: false,
        },
        optris_2_monitor_state: {
          enabled: true,
          min_voltage: 2,
          max_voltage: 8,
          delay_mm: 0,
          triggered: false,
        },
      }),
    });
    flush();

    handleMessage({
      name: "LiveValuesEvent",
      ts: 223,
      data: {
        traverse_position: 40,
        puller_speed: 1,
        spool_rpm: 10,
        tension_arm_angle: 130,
        spool_progress: 50,
        temperature_1: 200,
        temperature_2: 201,
        temperature_3: 202,
        temperature_4: 203,
        temperature_5: 204,
        temperature_6: 205,
        heater_1_power: 10,
        heater_2_power: 11,
        heater_3_power: 12,
        heater_4_power: 13,
        heater_5_power: 14,
        heater_6_power: 15,
        slave_puller_speed: 1,
        inlet_feeder_tension_arm_angle: 5,
        tape_feeder_tension_arm_angle: 6,
        optris_1_voltage: 9.5,
        optris_2_voltage: 9.8,
        addon_motor_5_rpm: 120,
      },
    });
    flush();

    const current = store.getState().state;
    expect(current?.data.winder_tension_arm_monitor_state.triggered).toBe(
      false,
    );
    expect(current?.data.optris_1_monitor_state.triggered).toBe(false);
    expect(current?.data.optris_2_monitor_state.triggered).toBe(false);
  });
});
