import { toastError } from "@/components/Toast";
import { useMachineMutate } from "@/client/useClient";
import { wago671Slot12TestMachine } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { z } from "zod";
import { wago671Slot12TestMachineSerialRoute } from "@/routes/routes";
import {
  StateEvent,
  useWago671Slot12TestMachineNamespace,
} from "./wago671Slot12TestMachineNamespace";

export function useWago671Slot12TestMachine() {
  const { serial: serialString } =
    wago671Slot12TestMachineSerialRoute.useParams();

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
      machine_identification: wago671Slot12TestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWago671Slot12TestMachineNamespace(machineIdentification);
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

  const setTargetSpeed = (axis: 1 | 2, target: number) => {
    updateStateOptimistically(
      (current) => {
        const slot = axis === 1 ? current.slot1 : current.slot2;
        slot.target_speed = target;
        slot.target_velocity = target;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetTargetSpeed", value: { axis, target } },
        }),
    );
  };

  const setEnabled = (axis: 1 | 2, enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        const slot = axis === 1 ? current.slot1 : current.slot2;
        slot.enabled = enabled;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetEnabled", value: { axis, enabled } },
        }),
    );
  };

  const setAcceleration = (axis: 1 | 2, acceleration: number) => {
    updateStateOptimistically(
      (current) => {
        const slot = axis === 1 ? current.slot1 : current.slot2;
        slot.acceleration = acceleration;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetAcceleration", value: { axis, acceleration } },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    setTargetSpeed,
    setEnabled,
    setAcceleration,
  };
}
