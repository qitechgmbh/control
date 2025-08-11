import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { buffer1, VENDOR_QITECH } from "@/machines/properties";
import {
  machineIdentificationUnique,
  MachineIdentificationUnique,
} from "@/machines/types";
import { buffer1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useEffect, useMemo } from "react";
import {
  StateEvent,
  Mode,
  useBuffer1Namespace,
  pullerRegulationSchema,
  PullerRegulation,
} from "./buffer1Namespace";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import { useMachines } from "@/client/useMachines";

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

  // Get machine identification unique
  const machine_identification_unique = machineIdentification;

  // Get consolidated state and live values from namespace
  const { state, defaultState, liftPosition, pullerSpeed } =
    useBuffer1Namespace(machineIdentification);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

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
        }),
    );
  };

  const setConnectedMachine = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          machineIdentificationUnique;
      },
      () =>
        requestConnectedMachine({
          machine_identification_unique,
          data: { SetConnectedMachine: machineIdentificationUnique },
        }),
    );
  };

  const disconnectMachine = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          null;
      },
      () =>
        requestDisconnectedMachine({
          machine_identification_unique,
          data: { DisconnectMachine: machineIdentificationUnique },
        }),
    );
  };

  const setCurrentInputSpeed = (current_input_speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.current_input_speed_state.current_input_speed =
          current_input_speed;
      },
      () =>
        requestSetCurrentInputSpeed({
          machine_identification_unique,
          data: { SetCurrentInputSpeed: current_input_speed },
        }),
    );
  };

  const setPullerTargetSpeed = (targetSpeed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_speed = targetSpeed;
      },
      () =>
        requestPullerSetTargetSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetPullerTargetSpeed: targetSpeed },
        }),
    );
  };

  const setPullerTargetDiameter = (targetDiameter: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_diameter = targetDiameter;
      },
      () =>
        requestPullerSetTargetDiameter({
          machine_identification_unique: machineIdentification,
          data: { SetPullerTargetDiameter: targetDiameter },
        }),
    );
  };

  const setPullerRegulationMode = (regulationMode: PullerRegulation) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.regulation = regulationMode;
      },
      () =>
        requestPullerSetRegulationMode({
          machine_identification_unique: machineIdentification,
          data: { SetPullerRegulationMode: regulationMode },
        }),
    );
  };

  const setPullerForward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.forward = forward;
      },
      () =>
        requestPullerSetForward({
          machine_identification_unique: machineIdentification,
          data: { SetPullerForward: forward },
        }),
    );
  };

  const setLiftLimitTop = (limitTop: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.lift_state.limit_top = limitTop;
      },
      () =>
        requestLiftSetLimitTop({
          machine_identification_unique: machineIdentification,
          data: { SetLiftLimitTop: limitTop },
        }),
    );
  };

  const setLiftLimitBottom = (limitBottom: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.lift_state.limit_bottom = limitBottom;
      },
      () =>
        requestLiftSetLimitBottom({
          machine_identification_unique: machineIdentification,
          data: { SetLiftLimitBottom: limitBottom },
        }),
    );
  };

  const gotoLiftLimitTop = () => {
    updateStateOptimistically(
      (current) => {
        current.data.lift_state.is_going_up = true;
      },
      () =>
        requestLiftGotoLimitTop({
          machine_identification_unique: machineIdentification,
          data: "GotoLiftLimitTop",
        }),
    );
  };

  const gotoLiftLimitBottom = () => {
    updateStateOptimistically(
      (current) => {
        current.data.lift_state.is_going_down = true;
      },
      () =>
        requestLiftGotoLimitBottom({
          machine_identification_unique: machineIdentification,
          data: "GotoLiftLimitBottom",
        }),
    );
  };

  const gotoLiftHome = () => {
    updateStateOptimistically(
      (current) => {
        current.data.lift_state.is_going_home = true;
      },
      () =>
        requestLiftGotoHome({
          machine_identification_unique: machineIdentification,
          data: "GotoLiftHome",
        }),
    );
  };

  const setLiftStepSize = (stepSize: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.lift_state.step_size = stepSize;
      },
      () =>
        requestLiftSetStepSize({
          machine_identification_unique: machineIdentification,
          data: { SetLiftStepSize: stepSize },
        }),
    );
  };

  const setLiftPadding = (padding: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.lift_state.padding = padding;
      },
      () =>
        requestLiftSetPadding({
          machine_identification_unique: machineIdentification,
          data: { SetLiftPadding: padding },
        }),
    );
  };

  // Mutation hooks
  const { request: requestBufferMode } = useMachineMutation(
    z.object({
      SetBufferMode: z.enum(["Standby", "Hold", "Filling", "Emptying"]),
    }),
  );

  const { request: requestConnectedMachine } = useMachineMutation(
    z.object({
      SetConnectedMachine: machineIdentificationUnique,
    }),
  );

  const { request: requestDisconnectedMachine } = useMachineMutation(
    z.object({
      DisconnectMachine: machineIdentificationUnique,
    }),
  );

  const { request: requestSetCurrentInputSpeed } = useMachineMutation(
    z.object({
      SetCurrentInputSpeed: z.number(),
    }),
  );

  const { request: requestPullerSetTargetSpeed } = useMachineMutation(
    z.object({ SetPullerTargetSpeed: z.number() }),
  );
  const { request: requestPullerSetTargetDiameter } = useMachineMutation(
    z.object({ SetPullerTargetDiameter: z.number() }),
  );
  const { request: requestPullerSetRegulationMode } = useMachineMutation(
    z.object({
      SetPullerRegulationMode: pullerRegulationSchema,
    }),
  );
  const { request: requestPullerSetForward } = useMachineMutation(
    z.object({ SetPullerForward: z.boolean() }),
  );

  const { request: requestLiftGotoLimitTop } = useMachineMutation(
    z.literal("GotoLiftLimitTop"),
  );
  const { request: requestLiftGotoLimitBottom } = useMachineMutation(
    z.literal("GotoLiftLimitBottom"),
  );
  const { request: requestLiftGotoHome } = useMachineMutation(
    z.literal("GotoLiftHome"),
  );
  const { request: requestLiftSetLimitTop } = useMachineMutation(
    z.object({ SetLiftLimitTop: z.number() }),
  );
  const { request: requestLiftSetLimitBottom } = useMachineMutation(
    z.object({ SetLiftLimitBottom: z.number() }),
  );
  const { request: requestLiftSetStepSize } = useMachineMutation(
    z.object({ SetLiftStepSize: z.number() }),
  );
  const { request: requestLiftSetPadding } = useMachineMutation(
    z.object({ SetLiftPadding: z.number() }),
  );

  // Calculate loading states
  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  // General Helper functions
  const machines = useMachines();
  // Filter machines for the correct type
  const filteredMachines = useMemo(
    () =>
      machines.filter(
        (m) =>
          m.machine_identification_unique.machine_identification.vendor ===
            VENDOR_QITECH &&
          m.machine_identification_unique.machine_identification.machine ===
            0x0002,
      ),
    [machines, machineIdentification],
  );

  // Get selected machine by serial
  const selectedMachine = useMemo(() => {
    const serial =
      state?.data.connected_machine_state?.machine_identification_unique
        ?.serial;

    return (
      filteredMachines.find(
        (m) => m.machine_identification_unique.serial === serial,
      ) ?? null
    );
  }, [filteredMachines, state]);

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    filteredMachines,
    selectedMachine,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Individual live values (TimeSeries)
    liftPosition,
    pullerSpeed,

    // Loading states
    isLoading,
    isDisabled,

    // Action functions (verb-first)
    setBufferMode,
    setLiftLimitTop,
    setLiftLimitBottom,
    gotoLiftLimitTop,
    gotoLiftLimitBottom,
    gotoLiftHome,
    setLiftStepSize,
    setLiftPadding,
    setConnectedMachine,
    disconnectMachine,
    setCurrentInputSpeed,
    setPullerTargetSpeed,
    setPullerTargetDiameter,
    setPullerRegulationMode,
    setPullerForward,
  };
}
