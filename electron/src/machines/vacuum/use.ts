import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { vacuumSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useVacuumNamespace,
  StateEvent,
  LiveValuesEvent,
  Mode,
} from "./MachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { vacuum } from "@/machines/properties";
import { z } from "zod";

export function useVacuum() {
  const { serial: serialString } = vacuumSerialRoute.useParams();

  // Memoize machine identification
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
      machine_identification: vacuum.machine_identification,
      serial,
    };
  }, [serialString]);

  // Namespace state from backend
  const { state, liveValues, remaining_time, spin_shitter } = useVacuumNamespace(
    machineIdentification,
  );

  // Optimistic state
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  // Generic mutation sender
  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value:  z.any(),
    }),
  );

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState)
      stateOptimistic.setOptimistic(produce(currentState, producer));
    serverRequest?.();
  };

  const setMode = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.mode = mode;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetMode", value: mode },
        }),
    );
  };

  const setIntervalTimeOff = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.interval_time_off = value;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetIntervalTimeOff", value: value },
        }),
    );
  };

  const setIntervalTimeOn = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.interval_time_on = value;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetIntervalTimeOn", value: value },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    liveValues,
    remaining_time,
    spin_shitter,
    setMode,
    setIntervalTimeOff,
    setIntervalTimeOn
  };
}