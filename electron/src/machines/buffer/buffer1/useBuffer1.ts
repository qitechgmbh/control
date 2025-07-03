import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { buffer1 } from "@/machines/properties";
import { machineIdentificationUnique, MachineIdentificationUnique } from "@/machines/types";
import { buffer1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useEffect, useMemo } from "react";
import { StateEvent, Mode, useBuffer1Namespace } from "./buffer1Namespace";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

export function useBuffer1() {
  const { serial: serialString } = buffer1SerialRoute.useParams();

  // Memoize the machine identification to keep it stable between renders
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString); // Use 0 as fallback if NaN

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        machine_identification: {
          vendor: 0,
          machine: 0,
        },
        serial: 0,
      };
    }

    return {
      machine_identification: buffer1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  // Get consolidated state and live values from namespace
  const {
    state,
  } = useBuffer1Namespace(machineIdentification);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state, stateOptimistic])

  // Helper function for optimistic updates using produce
  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest();
  };

  // Action functions with verb-first names
  const setBufferMode = async (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () =>
        requestBufferMode({
          machine_identification_unique: machineIdentification,
          data: { SetBufferMode: mode },
        })
    )
  };

  // Mutation hooks
  const { request: requestBufferMode } = useMachineMutation(
    z.object({ SetBufferMode: z.enum(["Standby", "FillingBuffer", "EmptyingBuffer"])}),
  );

 
  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Action functions (verb-first)
    setBufferMode,
  };
}