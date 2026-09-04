import { z } from "zod";
import { useRef } from "react";
import { produce } from "immer";
import { useSimulationMutate } from "@/client/useClient";
import {
  StrategyConfig,
  ZoneName,
  ZoneTuning,
  strategyConfigSchema,
  useSimulationNamespace,
  zoneNameSchema,
} from "@/client/simulationNamespace";

export type { StrategyConfig, ZoneName };

/**
 * The shipping controller's production PID gains, front/middle/back/nozzle —
 * see `ZoneTuning::PRODUCTION` in `harness.rs` — used as the seed the first
 * time a session switches to the "pid" algorithm, since the live simulation
 * doesn't start with any Pid gains cached.
 */
const DEFAULT_PID_GAINS: [ZoneTuning, ZoneTuning, ZoneTuning, ZoneTuning] = [
  { kp: 0.066, ki: 0, kd: 0 },
  { kp: 0.02, ki: 0.000003, kd: 0 },
  { kp: 0.02, ki: 0.000017, kd: 0 },
  { kp: 0.433, ki: 0.002, kd: 0 },
];

const zoneIndex: Record<ZoneName, 0 | 1 | 2 | 3> = {
  front: 0,
  middle: 1,
  back: 2,
  nozzle: 3,
};

export function useExtruderSimulation() {
  const { state, liveValues, ...timeSeries } = useSimulationNamespace();

  // The last strategy config seen of each kind, so switching the algorithm
  // back and forth doesn't lose the other one's gains. ObserverPi is what the
  // simulation starts with, so this is populated well before a user could
  // switch back to it; Pid falls back to the shipping gains above.
  const lastObserverPi = useRef<StrategyConfig["ObserverPi"]>(undefined);
  const lastPid = useRef<StrategyConfig["Pid"]>(DEFAULT_PID_GAINS);
  if (state?.data.strategy.ObserverPi) {
    lastObserverPi.current = state.data.strategy.ObserverPi;
  }
  if (state?.data.strategy.Pid) {
    lastPid.current = state.data.strategy.Pid;
  }

  const { request: requestSetSetpoint } = useSimulationMutate(
    z.object({
      SetSetpoint: z.object({ zone: zoneNameSchema, celsius: z.number() }),
    }),
  );
  const { request: requestSetStrategy } = useSimulationMutate(
    z.object({ SetStrategy: strategyConfigSchema }),
  );
  const { request: requestSetScrewRpm } = useSimulationMutate(
    z.object({ SetScrewRpm: z.number() }),
  );
  const { request: requestSetSpeed } = useSimulationMutate(
    z.object({ SetSpeed: z.number() }),
  );
  const { request: requestPlay } = useSimulationMutate(
    z.object({ Play: z.object({}) }),
  );
  const { request: requestPause } = useSimulationMutate(
    z.object({ Pause: z.object({}) }),
  );
  const { request: requestReset } = useSimulationMutate(
    z.object({ Reset: z.object({ initial_c: z.number() }) }),
  );

  const setSetpoint = (zone: ZoneName, celsius: number) => {
    requestSetSetpoint({ SetSetpoint: { zone, celsius } });
  };

  const setAlgorithm = (algorithm: "pid" | "observer-pi") => {
    if (algorithm === "pid") {
      requestSetStrategy({ SetStrategy: { Pid: lastPid.current } });
    } else if (lastObserverPi.current) {
      requestSetStrategy({
        SetStrategy: { ObserverPi: lastObserverPi.current },
      });
    }
  };

  const setPidGain = (
    zone: ZoneName,
    key: "kp" | "ki" | "kd",
    value: number,
  ) => {
    const current = state?.data.strategy.Pid;
    if (!current) return;
    const updated = produce(current, (draft) => {
      draft[zoneIndex[zone]][key] = value;
    });
    requestSetStrategy({ SetStrategy: { Pid: updated } });
  };

  const setObserverPiGain = (
    zone: ZoneName,
    key: "kp" | "ki",
    value: number,
  ) => {
    const current = state?.data.strategy.ObserverPi;
    if (!current) return;
    const updated = produce(current, (draft) => {
      draft[zoneIndex[zone]][key] = value;
    });
    requestSetStrategy({ SetStrategy: { ObserverPi: updated } });
  };

  const setScrewRpm = (rpm: number) => requestSetScrewRpm({ SetScrewRpm: rpm });
  const setSpeed = (speed: number) => requestSetSpeed({ SetSpeed: speed });
  const play = () => requestPlay({ Play: {} });
  const pause = () => requestPause({ Pause: {} });
  const reset = (initialC: number) =>
    requestReset({ Reset: { initial_c: initialC } });

  return {
    state: state?.data,
    liveValues: liveValues?.data,
    isLoading: !state,

    ...timeSeries,

    setSetpoint,
    setAlgorithm,
    setPidGain,
    setObserverPiGain,
    setScrewRpm,
    setSpeed,
    play,
    pause,
    reset,
  };
}
