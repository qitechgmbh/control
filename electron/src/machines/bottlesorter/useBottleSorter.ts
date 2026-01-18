import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { bottleSorterSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useBottleSorterNamespace,
  StateEvent,
  LiveValuesEvent,
} from "./bottleSorterNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { bottleSorter } from "@/machines/properties";
import { z } from "zod";

export function useBottleSorter() {
  const { serial: serialString } = bottleSorterSerialRoute.useParams();

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
      machine_identification: bottleSorter.machine_identification,
      serial,
    };
  }, [serialString]);

  // Namespace state from backend
  const { state, liveValues } = useBottleSorterNamespace(
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
      value: z.any(),
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

  const setStepperSpeed = (speed_mm_s: number) => {
    updateStateOptimistically(
      (current) => {
        current.stepper_speed_mm_s = speed_mm_s;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperSpeed", value: { speed_mm_s } },
        }),
    );
  };

  const setStepperEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.stepper_enabled = enabled;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperEnabled", value: { enabled } },
        }),
    );
  };

  const pulseOutput = (index: number) => {
    // Immediately pulse the output
    updateStateOptimistically(
      (current) => {
        current.outputs[index] = true;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "PulseOutput", value: { index } },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    liveValues,
    setStepperSpeed,
    setStepperEnabled,
    pulseOutput,
  };
}
