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

  const { state } = useWagoWinderSmokeTestMachineNamespace(machineIdentification);
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

  const setStepperEnabled = (axis: number, enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.axes[axis].enabled = enabled;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperEnabled", value: { axis, enabled } },
        }),
    );
  };

  const setStepperVelocity = (axis: number, velocity: number) => {
    updateStateOptimistically(
      (current) => {
        current.axes[axis].target_velocity = velocity;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperVelocity", value: { axis, velocity } },
        }),
    );
  };

  const setStepperFreqRange = (axis: number, factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.axes[axis].freq_range_sel = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperFreqRange", value: { axis, factor } },
        }),
    );
  };

  const setStepperAccRange = (axis: number, factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.axes[axis].acc_range_sel = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperAccRange", value: { axis, factor } },
        }),
    );
  };

  const setDigitalOutput = (port: number, value: boolean) => {
    updateStateOptimistically(
      (current) => {
        if (port === 1) current.digital_output1 = value;
        if (port === 2) current.digital_output2 = value;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetDigitalOutput", value: { port, value } },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    setStepperEnabled,
    setStepperVelocity,
    setStepperFreqRange,
    setStepperAccRange,
    setDigitalOutput,
  };
}
