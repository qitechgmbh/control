import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { wago671Slot1TestMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useWago671Slot1TestMachineNamespace,
  StateEvent,
} from "./wago671Slot1TestMachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { wago671Slot1TestMachine } from "@/machines/properties";
import { z } from "zod";

export function useWago671Slot1TestMachine() {
  const { serial: serialString } =
    wago671Slot1TestMachineSerialRoute.useParams();

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
      machine_identification: wago671Slot1TestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWago671Slot1TestMachineNamespace(machineIdentification);
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

  const setTargetSpeed = (target: number) => {
    updateStateOptimistically(
      (current) => {
        current.target_speed = target;
        current.target_velocity = target;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetTargetSpeed", value: { target } },
        }),
    );
  };

  const setEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.enabled = enabled;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetEnabled", value: { enabled } },
        }),
    );
  };

  const setFreq = (factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.freq = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetFreq", value: { factor } },
        }),
    );
  };

  const setAccFreq = (factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.acc_freq = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetAccFreq", value: { factor } },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    setTargetSpeed,
    setEnabled,
    setFreq,
    setAccFreq,
  };
}
