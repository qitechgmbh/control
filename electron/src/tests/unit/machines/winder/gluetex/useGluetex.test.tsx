import React from "react";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTimeSeries } from "@/lib/timeseries";
import type {
  ExtendedStateEvent,
  GluetexNamespaceStore,
} from "@/machines/winder/gluetex/state/gluetexNamespace";

const {
  requestSpy,
  useMachineMutateMock,
  useParamsMock,
  useGluetexNamespaceMock,
  toastErrorMock,
} = vi.hoisted(() => ({
  requestSpy: vi.fn(),
  useMachineMutateMock: vi.fn(),
  useParamsMock: vi.fn(() => ({ serial: "42" })),
  useGluetexNamespaceMock: vi.fn(),
  toastErrorMock: vi.fn(),
}));

useMachineMutateMock.mockImplementation(() => ({ request: requestSpy }));

vi.mock("@/client/useClient", () => ({
  useMachineMutate: useMachineMutateMock,
}));

vi.mock("@/client/useMachines", () => ({
  useMachines: () => [],
}));

vi.mock("@/routes/routes", () => ({
  gluetexRoute: {
    useParams: useParamsMock,
  },
}));

vi.mock("@/components/Toast", () => ({
  toastError: toastErrorMock,
}));

vi.mock("@/machines/winder/gluetex/state/gluetexNamespace", async () => {
  const actual = await vi.importActual<
    typeof import("@/machines/winder/gluetex/state/gluetexNamespace")
  >("@/machines/winder/gluetex/state/gluetexNamespace");

  return {
    ...actual,
    useGluetexNamespace: useGluetexNamespaceMock,
  };
});

import { useGluetex } from "@/machines/winder/gluetex/hooks/useGluetex";
import { createGluetexNamespaceStore } from "@/machines/winder/gluetex/state/gluetexNamespace";

const createNamespaceSnapshot = (): GluetexNamespaceStore => {
  const store = createGluetexNamespaceStore();
  const snapshot = store.getState();
  const defaultState = snapshot.defaultState;

  if (!defaultState) {
    throw new Error("Expected default state");
  }

  const state: ExtendedStateEvent = {
    ...defaultState,
    ts: 123,
    data: structuredClone(defaultState.data),
  };

  const timeSeries = createTimeSeries().initialTimeSeries;

  return {
    ...snapshot,
    state,
    defaultState: state,
    traversePosition: timeSeries,
    pullerSpeed: timeSeries,
    slavePullerSpeed: timeSeries,
    inletFeederTensionArmAngle: timeSeries,
    tapeFeederTensionArmAngle: timeSeries,
    spoolRpm: timeSeries,
    tensionArmAngle: timeSeries,
    spoolProgress: timeSeries,
    temperature1: timeSeries,
    temperature2: timeSeries,
    temperature3: timeSeries,
    temperature4: timeSeries,
    temperature5: timeSeries,
    temperature6: timeSeries,
    heater1Power: timeSeries,
    heater2Power: timeSeries,
    heater3Power: timeSeries,
    heater4Power: timeSeries,
    heater5Power: timeSeries,
    heater6Power: timeSeries,
    optris1Voltage: timeSeries,
    optris2Voltage: timeSeries,
    addonMotor5Rpm: timeSeries,
    reconfigureLongBuffers: vi.fn(),
  };
};

describe("useGluetex", () => {
  beforeEach(() => {
    requestSpy.mockReset();
    useMachineMutateMock.mockClear();
    toastErrorMock.mockReset();
    useParamsMock.mockReturnValue({ serial: "42" });
    useGluetexNamespaceMock.mockReturnValue(createNamespaceSnapshot());
  });

  it("forwards monitor boundary values to backend unchanged (no frontend safety clamping)", () => {
    const { result } = renderHook(() => useGluetex());

    requestSpy.mockClear();

    act(() => {
      result.current.setWinderTensionArmMonitorMinAngle(-123.45);
    });

    expect(requestSpy).toHaveBeenCalledTimes(1);
    expect(requestSpy).toHaveBeenCalledWith({
      machine_identification_unique: {
        machine_identification: {
          vendor: 1,
          machine: 11,
        },
        serial: 42,
      },
      data: { SetWinderTensionArmMonitorMinAngle: -123.45 },
    });
  });

  it("falls back to a safe placeholder machine id when route serial is invalid", () => {
    useParamsMock.mockReturnValue({ serial: "not-a-number" });

    const { result } = renderHook(() => useGluetex());

    requestSpy.mockClear();

    act(() => {
      result.current.setMode("Pull");
    });

    expect(toastErrorMock).toHaveBeenCalledWith(
      "Invalid Serial Number",
      '"not-a-number" is not a valid serial number.',
    );
    expect(requestSpy).toHaveBeenCalledTimes(1);
    expect(requestSpy).toHaveBeenCalledWith({
      machine_identification_unique: {
        machine_identification: {
          vendor: 0,
          machine: 0,
        },
        serial: 0,
      },
      data: { SetMode: "Pull" },
    });
  });
});
