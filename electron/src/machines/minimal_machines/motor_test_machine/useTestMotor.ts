import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { testMotorSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import { useTestMotorNamespace, StateEvent } from "./testMotorNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { TestMotor } from "@/machines/properties";
import { z } from "zod";

export function useTestMotor() {
  const { serial: serialString } = testMotorSerialRoute.useParams();

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
      machine_identification: TestMotor.machine_identification,
      serial,
    };
  }, [serialString]);

  // Namespace state from backend
  const { state } = useTestMotorNamespace(machineIdentification);

  // Optimistic state
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  // Generic mutation sender
  const { request: sendMutation } = useMachineMutate(
    z.object({
      type: z.string(),
      payload: z.any(),
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

  // --- ACTIONS ---

  const setMotorOn = (on: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.motor_enabled = on;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          // Must exactly match the 'Mutation' enum!
          data: { type: "SetMotorOn", payload: on },
        }),
    );
  };

  const setVelocity = (velocity: number) => {
    updateStateOptimistically(
      (current) => {
        current.motor_velocity = velocity;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { type: "SetMotorVelocity", payload: velocity },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    setMotorOn,
    setVelocity,
  };
}
