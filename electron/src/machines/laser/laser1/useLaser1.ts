import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import {
  machineIdentification,
  machineIdentificationUnique,
  MachineIdentificationUnique,
} from "@/machines/types";
import { laser1, VENDOR_QITECH } from "@/machines/properties";
import { laser1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useLaser1Namespace, StateEvent } from "./laser1Namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import { useMachines } from "@/client/useMachines";

function useLaser(machine_identification_unique: MachineIdentificationUnique) {
  // Get consolidated state and live values from namespace
  const { state, defaultState, diameter, x_diameter, y_diameter, roundness } =
    useLaser1Namespace(machine_identification_unique);

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

  // Mutation schemas
  const schemaTargetDiameter = z.object({
    SetTargetDiameter: z.number(),
  });
  const { request: requestTargetDiameter } =
    useMachineMutation(schemaTargetDiameter);

  const schemaLowerTolerance = z.object({
    SetLowerTolerance: z.number(),
  });
  const { request: requestLowerTolerance } =
    useMachineMutation(schemaLowerTolerance);

  const schemaHigherTolerance = z.object({
    SetHigherTolerance: z.number(),
  });
  const { request: requestHigherTolerance } = useMachineMutation(
    schemaHigherTolerance,
  );
  const { request: requestConnectedWinder } = useMachineMutation(
    z.object({
      SetConnectedWinder: machineIdentificationUnique,
    }),
  );
  const { request: requestDisconnectWinder } = useMachineMutation(
    z.object({
      DisconnectWinder: machineIdentificationUnique,
    }),
  );
  const { request: requestSpeedPidSettings } = useMachineMutation(
    z.object({
      SetSpeedPidSettings: z.object({
        ki: z.number(),
        kp: z.number(),
        kd: z.number(),
      }),
    }),
  );

  // Action functions with verb-first names
  const setTargetDiameter = (target_diameter: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.laser_state.target_diameter = target_diameter;
      },
      () =>
        requestTargetDiameter({
          machine_identification_unique,
          data: {
            SetTargetDiameter: target_diameter,
          },
        }),
    );
  };

  const setLowerTolerance = (lower_tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.laser_state.lower_tolerance = lower_tolerance;
      },
      () =>
        requestLowerTolerance({
          machine_identification_unique,
          data: {
            SetLowerTolerance: lower_tolerance,
          },
        }),
    );
  };

  const setHigherTolerance = (higher_tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.laser_state.higher_tolerance = higher_tolerance;
      },
      () =>
        requestHigherTolerance({
          machine_identification_unique,
          data: {
            SetHigherTolerance: higher_tolerance,
          },
        }),
    );
  };

  const setSpeedPidKp = (kp: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.speed.kp = kp;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.speed,
            (draft) => {
              draft.kp = kp;
            },
          );
          requestSpeedPidSettings({
            machine_identification_unique,
            data: { SetSpeedPidSettings: settings },
          });
        }
      },
    );
  };

  const setSpeedPidKi = (ki: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.speed.ki = ki;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.speed,
            (draft) => {
              draft.ki = ki;
            },
          );
          requestSpeedPidSettings({
            machine_identification_unique,
            data: { SetSpeedPidSettings: settings },
          });
        }
      },
    );
  };

  const setSpeedPidKd = (kd: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.speed.kd = kd;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.speed,
            (draft) => {
              draft.kd = kd;
            },
          );
          requestSpeedPidSettings({
            machine_identification_unique,
            data: { SetSpeedPidSettings: settings },
          });
        }
      },
    );
  };

  const setSpeedPidDead = (dead: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.speed.dead = dead;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.speed,
            (draft) => {
              draft.dead = dead;
            },
          );
          requestSpeedPidSettings({
            machine_identification_unique,
            data: { SetSpeedPidSettings: settings },
          });
        }
      },
    );
  };

  const setConnectedWinder = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_winder_state.machine_identification_unique =
          machineIdentificationUnique;
      },
      () =>
        requestConnectedWinder({
          machine_identification_unique,
          data: { SetConnectedWinder: machineIdentificationUnique },
        }),
    );
  };

  const disconnectWinder = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_winder_state.machine_identification_unique =
          null;
      },
      () =>
        requestDisconnectWinder({
          machine_identification_unique,
          data: { DisconnectWinder: machineIdentificationUnique },
        }),
    );
  };

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
      state?.data.connected_winder_state?.machine_identification_unique?.serial;

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

    // Live values (TimeSeries)
    diameter,
    x_diameter,
    y_diameter,
    roundness,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions (verb-first)
    setTargetDiameter,
    setLowerTolerance,
    setHigherTolerance,
    setSpeedPidKp,
    setSpeedPidKi,
    setSpeedPidKd,
    setSpeedPidDead,
    setConnectedWinder,
    disconnectWinder,
  };
}

export function useLaser1() {
  const { serial: serialString } = laser1SerialRoute.useParams();

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
      machine_identification: laser1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const laser = useLaser(machineIdentification);

  return {
    ...laser,
  };
}
