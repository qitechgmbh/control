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
    temperature_sensor1,
    temperature_sensor2,
    flow_sensor1,
    flow_sensor2,
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

  const setFrontCoolingTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.cooling_states.front.target_temperature = temperature;
      },
      () =>
        requestFrontCoolingTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetFrontTemperature: temperature },
        }),
    );
  };

  const setBackCoolingTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.cooling_states.back.target_temperature = temperature;
      },
      () =>
        requestBackCoolingTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetBackTemperature: temperature },
        }),
    );
  };

  // Mutation hooks
  const { request: requestAquapathMode } = useMachineMutation(
    z.object({ SetAquaPathMode: z.enum(["Standby", "Cool", "Heat"]) }),
  );
  const { request: requestFrontCoolingTemperature } = useMachineMutation(
    z.object({ SetFrontTemperature: z.number() }),
  );
  const { request: requestBackCoolingTemperature } = useMachineMutation(
    z.object({ SetBackTemperature: z.number() }),
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
    flow_sensor1,
    flow_sensor2,
    temperature_sensor1,
    temperature_sensor2,

    setAquapathMode,
    setFrontCoolingTemperature,
    setBackCoolingTemperature,
  };
}
