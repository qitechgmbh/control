import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { aquapath1 } from "@/machines/properties";
import { aquapath1SerialRoute } from "@/routes/routes";
import { Mode, StateEvent, useAquapath1Namespace } from "./aquapath1Namespace";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";

import { useEffect, useMemo } from "react";
import { produce } from "immer";
import { z } from "zod";

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

  // Get consolidated state and live values from namespace
  const {
    state,
    defaultState,
    front_temperature: front_temperature,
    back_temperature: back_temperature,
    front_flow: front_flow,
    back_flow: back_flow,
    front_temp_reservoir: front_temp_reservoir,
    back_temp_reservoir: back_temp_reservoir,
  } = useAquapath1Namespace(machineIdentification);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state, stateOptimistic]);

  const setAquapathMode = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () =>
        requestAquapathMode({
          machine_identification_unique: machineIdentification,
          data: { SetAquaPathMode: mode },
        }),
    );
  };

  const setFrontTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.temperature_states.front.target_temperature = temperature;
      },
      () =>
        requestFrontTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetFrontTemperature: temperature },
        }),
    );
  };

  const setBackTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.temperature_states.back.target_temperature = temperature;
      },
      () =>
        requestBackTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetBackTemperature: temperature },
        }),
    );
  };

  const setFrontFlow = (flow: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.flow_states.front.should_flow = flow;
      },
      () =>
        requestFrontFlow({
          machine_identification_unique: machineIdentification,
          data: { SetFrontFlow: flow },
        }),
    );
  };

  const setBackFlow = (flow: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.flow_states.back.should_flow = flow;
      },
      () =>
        requestBackFlow({
          machine_identification_unique: machineIdentification,
          data: { SetBackFlow: flow },
        }),
    );
  };

  // Mutation hooks
  const { request: requestAquapathMode } = useMachineMutation(
    z.object({ SetAquaPathMode: z.enum(["Standby", "Auto"]) }),
  );
  const { request: requestFrontTemperature } = useMachineMutation(
    z.object({ SetFrontTemperature: z.number() }),
  );
  const { request: requestBackTemperature } = useMachineMutation(
    z.object({ SetBackTemperature: z.number() }),
  );
  const { request: requestFrontFlow } = useMachineMutation(
    z.object({ SetFrontFlow: z.boolean() }),
  );
  const { request: requestBackFlow } = useMachineMutation(
    z.object({ SetBackFlow: z.boolean() }),
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

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,
    front_flow,
    back_flow,
    front_temperature,
    back_temperature,
    front_temp_reservoir,
    back_temp_reservoir,

    setAquapathMode,
    setFrontTemperature,
    setBackTemperature,
    setFrontFlow,
    setBackFlow,
  };
}
