import { toastError } from "@ui/components/Toast";
import { wagoSerial } from "@ui/machines/properties";
import { MachineIdentificationUnique } from "@ui/machines/types";
import { wagoSerialSerialRoute } from "@ui/routes/routes";
import { useEffect, useMemo } from "react";
import { StateEvent, useWagoSerialNamespace } from "./wagoSerialNamespace";
import { useStateOptimistic } from "@ui/lib/useStateOptimistic";
import { produce } from "immer";
import z from "zod";
import { useMachineMutate } from "@ui/client/useClient";

export function useWagoSerial() {
  const { serial: serialString } = wagoSerialSerialRoute.useParams();
  // Memoize the machine identification to keep it stable between renders
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);
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
      machine_identification: wagoSerial.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state, defaultState } = useWagoSerialNamespace(machineIdentification);
  //
  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  const { request: requestSendMessage } = useMachineMutate(
    z.object({ SendMessage: z.string() }),
  );

  // Helper function for optimistic updates using produce
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

  const sendMessage = (msg: string) => {
    updateStateOptimistically(
      (current) => {
        current.data.current_message = msg;
      },
      () =>
        requestSendMessage({
          machine_identification_unique: machineIdentification,
          data: { SendMessage: msg },
        }),
    );
  };

  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  return {
    state: stateOptimistic.value?.data,
    isLoading,
    isDisabled,
    sendMessage,
  };
}
