import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerV1 } from "@/machines/properties";
import { dryerV1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useEffect, useMemo } from "react";
import { produce } from "immer";
import {
  ScheduleDay,
  StateEvent,
  useDryerV1Namespace,
} from "./dryerV1Namespace";

const scheduleDaySchema = z.object({
  start_minutes: z.number(),
  stop_minutes: z.number(),
});

export function useDryerV1() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        machine_identification: { vendor: 0, machine: 0 },
        serial: 0,
      };
    }

    return {
      machine_identification: dryerV1.machine_identification,
      serial,
    };
  }, [serialString]);

  const {
    state,
    defaultState,
    liveValues,
    ts_temp_process,
    ts_temp_safety,
    ts_temp_regen_in,
    ts_temp_regen_out,
    ts_temp_fan_inlet,
    ts_temp_return_air,
    ts_pwm_fan1,
    ts_pwm_fan2,
    ts_power_process,
    ts_power_regen,
  } = useDryerV1Namespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state]);

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState && !stateOptimistic.isOptimistic) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest();
  };

  const setRunning = (running: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.status = running ? 4 : 1;
      },
      () =>
        requestRunning({
          machine_identification_unique: machineIdentification,
          data: { SetRunning: running },
        }),
    );
  };

  const setTargetTemperature = (celsius: number) => {
    updateStateOptimistically(
      (current) => {
        current.target_temperature = celsius;
      },
      () =>
        requestTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetTargetTemperature: celsius },
        }),
    );
  };

  const setAirVolume = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.air_volume = value;
      },
      () =>
        requestAirVolume({
          machine_identification_unique: machineIdentification,
          data: { SetAirVolume: value },
        }),
    );
  };

  const setDryingTimerMinutes = (minutes: number) => {
    updateStateOptimistically(
      (current) => {
        current.drying_timer_minutes = minutes;
      },
      () =>
        requestDryingTimerMinutes({
          machine_identification_unique: machineIdentification,
          data: { SetDryingTimerMinutes: minutes },
        }),
    );
  };

  const setSchedule = (schedule: ScheduleDay[]) => {
    updateStateOptimistically(
      (current) => {
        current.schedule = schedule;
      },
      () =>
        requestSchedule({
          machine_identification_unique: machineIdentification,
          data: { SetSchedule: { schedule } },
        }),
    );
  };

  const { request: requestRunning } = useMachineMutation(
    z.object({ SetRunning: z.boolean() }),
  );
  const { request: requestTargetTemperature } = useMachineMutation(
    z.object({ SetTargetTemperature: z.number() }),
  );
  const { request: requestAirVolume } = useMachineMutation(
    z.object({ SetAirVolume: z.number() }),
  );
  const { request: requestDryingTimerMinutes } = useMachineMutation(
    z.object({ SetDryingTimerMinutes: z.number() }),
  );
  const { request: requestSchedule } = useMachineMutation(
    z.object({
      SetSchedule: z.object({ schedule: z.array(scheduleDaySchema).length(7) }),
    }),
  );

  return {
    state: stateOptimistic.value,
    defaultState,
    liveValues,
    ts_temp_process,
    ts_temp_safety,
    ts_temp_regen_in,
    ts_temp_regen_out,
    ts_temp_fan_inlet,
    ts_temp_return_air,
    ts_pwm_fan1,
    ts_pwm_fan2,
    ts_power_process,
    ts_power_regen,

    setRunning,
    setTargetTemperature,
    setAirVolume,
    setDryingTimerMinutes,
    setSchedule,
  };
}
