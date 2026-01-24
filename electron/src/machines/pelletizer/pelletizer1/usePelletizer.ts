import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { MachineIdentificationUnique } from "@/machines/types";
import { pelletizer } from "@/machines/properties";
import { pelletizer1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { usePellet1Namespace, StateEvent } from "./namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

function usePellet(machine_identification_unique: MachineIdentificationUnique) {
    // Get consolidated state and live values from namespace
    const { state, defaultState, frequency, temperature, voltage, current } =
      usePellet1Namespace(machine_identification_unique);

    // Single optimistic state for all state management
    const stateOptimistic = useStateOptimistic<StateEvent>();

    useEffect(() => { if (state) { stateOptimistic.setReal(state); } }, [state]);

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

    // Mutation schemas
    const schemaRunMode = z.object({ SetRunMode: z.number() });
    const { request: requestRunMode } = useMachineMutation(schemaRunMode);
    
    const schemaFrequencyTarget = z.object({ SetFrequencyTarget: z.number() });
    const { request: requestFrequencyTarget } = useMachineMutation(schemaFrequencyTarget);

    const schemaAccelerationLevel = z.object({ SetAccelerationLevel: z.number() });
    const { request: requestAccelerationLevel } = useMachineMutation(schemaAccelerationLevel);

    const schemaDecelerationLevel = z.object({ SetDecelerationLevel: z.number() });
    const { request: requestDecelerationLevel } = useMachineMutation(schemaDecelerationLevel);

    const SetRunMode = (run_mode: number) => {
      updateStateOptimistically(
        (current) => {
          current.data.inverter_state.running_state = run_mode;
        },
        () =>
          requestRunMode({
            machine_identification_unique,
            data: {
              SetRunMode: run_mode,
            },
          }),
      );
    };

    const SetFrequencyTarget = (frequency_target: number) => 
    {
      updateStateOptimistically(
        (current) => {
          current.data.inverter_state.frequency_target = frequency_target;
        },
        () =>
          requestFrequencyTarget({
            machine_identification_unique,
            data: {
              SetFrequencyTarget: frequency_target,
            },
          }),
      );
    };

    const SetAccelerationLevel = (acceleration_level: number) => {
        updateStateOptimistically(
            (current) => {
            current.data.inverter_state.acceleration_level = acceleration_level;
            },
            () =>
            requestAccelerationLevel({
                machine_identification_unique,
                data: {
                SetAccelerationLevel: acceleration_level,
                },
            }),
        );
    };

    const SetDecelerationLevel = (deceleration_level: number) => {
        updateStateOptimistically(
            (current) => {
            current.data.inverter_state.deceleration_level = deceleration_level;
            },
            () =>
            requestDecelerationLevel({
                machine_identification_unique,
                data: {
                SetDecelerationLevel: deceleration_level,
                },
            }),
        );
    };

    return {
      // Consolidated state
      state: stateOptimistic.value?.data,

      // Default state for initial values
      defaultState: defaultState?.data,

      // Live values (TimeSeries)
      frequency,
      temperature,
      voltage,
      current,

      // Loading states
      isLoading: stateOptimistic.isOptimistic,
      isDisabled: !stateOptimistic.isInitialized,

      // Action functions
      SetRunMode,
      SetFrequencyTarget,
      SetAccelerationLevel,
      SetDecelerationLevel,
    };
}

export function usePellet1() 
{
    const { serial: serialString } = pelletizer1SerialRoute.useParams();

    // Memoize the machine identification to keep it stable between renders
    const machineIdentification: MachineIdentificationUnique = useMemo(() => 
    {
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
          machine_identification: pelletizer.machine_identification,
          serial,
        };
    }, [serialString]); // Only recreate when serialString changes

    const pellet = usePellet(machineIdentification);

    return {
        ...pellet,
    };
}
