import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useEffect, useMemo } from "react";
import { MachineIdentificationUnique } from "../../types";
import { VENDOR_QITECH, aquapath1 } from "../../properties";
import { toastError } from "@/components/Toast";
import { StateEvent, useAquapath1Namespace } from "./aquapath1Namespace";
import { aquapath1SerialRoute } from "@/routes/routes";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import z from "zod";
import { produce } from "immer";

export function useAquapath1() {
  const { serial: serialString } = aquapath1SerialRoute.useParams();

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
      machine_identification: aquapath1.machine_identification,
      serial,
    };
  }, [serialString]);

  const machine_identification_unique = machineIdentification;

  // Get consolidated state and live values from namespace
  const { state, defaultState, fanRpm, waterTemperature, flowRate } =
    useAquapath1Namespace(machineIdentification);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  // Request function for all operations
  const { request: requestSetTargetTemperature } = useMachineMutation(
    z.object({ SetTargetTemperature: z.number() }),
  );

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

  // Action functions
  const setTargetTemperature = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.target_temperature = value;
      },
      () =>
        requestSetTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetTargetTemperature: value },
        }),
    );
  };

  // Calculate loading states
  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Individual live values (TimeSeries)
    fanRpm,
    waterTemperature,
    flowRate,

    // Loading states
    isLoading,
    isDisabled,

    // Action functions
    setTargetTemperature,
  };
}
