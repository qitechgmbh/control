import { useMachineMutate } from "@/client/useClient";
import { toastError } from "@/components/Toast";
import { wagoWinderSmokeTestMachine } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { z } from "zod";
import { wagoWinderSmokeTestMachineSerialRoute } from "@/routes/routes";
import {
  StateEvent,
  useWagoWinderSmokeTestMachineNamespace,
} from "./wagoWinderSmokeTestMachineNamespace";

export function useWagoWinderSmokeTestMachine() {
  const { serial: serialString } =
    wagoWinderSmokeTestMachineSerialRoute.useParams();

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
      machine_identification: wagoWinderSmokeTestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWagoWinderSmokeTestMachineNamespace(
    machineIdentification,
  );
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value: z.any(),
    }),
  );

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest?.();
  };

  const setStepperEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.enabled = enabled;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperEnabled", value: enabled },
        }),
    );
  };

  const setStepperVelocity = (velocity: number) => {
    updateStateOptimistically(
      (current) => {
        current.target_velocity = velocity;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperVelocity", value: velocity },
        }),
    );
  };

  const setStepperPosition = (position: number) => {
    updateStateOptimistically(
      (current) => {
        current.position = position;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperPosition", value: position },
        }),
    );
  };

  const setStepperFreqRange = (factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.freq_range_sel = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperFreqRange", value: factor },
        }),
    );
  };

  const setStepperAccRange = (factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.acc_range_sel = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperAccRange", value: factor },
        }),
    );
  };

  const sendSimpleAction = (
    action:
      | "StartCoarseSeek"
      | "StopByZeroVelocity"
      | "StopByStop2N"
      | "ReleaseStop2N",
  ) => {
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { action },
    });
  };

  return {
    state: stateOptimistic.value,
    setStepperEnabled,
    setStepperVelocity,
    setStepperPosition,
    setStepperFreqRange,
    setStepperAccRange,
    startCoarseSeek: () => sendSimpleAction("StartCoarseSeek"),
    stopByZeroVelocity: () => sendSimpleAction("StopByZeroVelocity"),
    stopByStop2N: () => sendSimpleAction("StopByStop2N"),
    releaseStop2N: () => sendSimpleAction("ReleaseStop2N"),
  };
}
