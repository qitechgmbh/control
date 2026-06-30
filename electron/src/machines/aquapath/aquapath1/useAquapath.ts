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
    left_temperature,
    right_temperature,
    left_flow,
    right_flow,
    left_temp_reservoir,
    right_temp_reservoir,
    left_revolutions,
    right_revolutions,
    left_power,
    right_power,
    combinedPower,
    left_total_energy,
    right_total_energy,
    totalEnergyKWh,
    left_heating,
    right_heating,
    left_cooling_mode,
    right_cooling_mode,
    left_pump_cooldown_active,
    right_pump_cooldown_active,
    left_pump_cooldown_remaining,
    right_pump_cooldown_remaining,
    left_heating_startup_wait_active,
    right_heating_startup_wait_active,
    left_heating_startup_wait_remaining,
    right_heating_startup_wait_remaining,
    targetLeftTemperature,
    targetRightTemperature,
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

  const setLeftTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.temperature_states.left.target_temperature = temperature;
      },
      () =>
        requestLeftTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetLeftTemperature: temperature },
        }),
    );
  };

  const setRightTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.temperature_states.right.target_temperature = temperature;
      },
      () =>
        requestRightTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetRightTemperature: temperature },
        }),
    );
  };

  const setLeftFlow = (flow: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.flow_states.left.should_flow = flow;
      },
      () =>
        requestLeftFlow({
          machine_identification_unique: machineIdentification,
          data: { SetLeftFlow: flow },
        }),
    );
  };

  const setRightFlow = (flow: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.flow_states.right.should_flow = flow;
      },
      () =>
        requestRightFlow({
          machine_identification_unique: machineIdentification,
          data: { SetRightFlow: flow },
        }),
    );
  };

  const setLeftRevolutions = (revolutions: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.fan_states.left.revolutions = revolutions;
      },
      () => {
        requestLeftRevolutions({
          machine_identification_unique: machineIdentification,
          data: { SetLeftRevolutions: revolutions },
        });
      },
    );
  };

  const setRightRevolutions = (revolutions: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.fan_states.right.revolutions = revolutions;
      },
      () =>
        requestRightRevolutions({
          machine_identification_unique: machineIdentification,
          data: { SetRightRevolutions: revolutions },
        }),
    );
  };

  const setLeftHeatingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.left.heating = tolerance;
      },
      () => {
        requestLeftHeatingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetLeftHeatingTolerance: tolerance },
        });
      },
    );
  };

  const setRightHeatingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.right.heating = tolerance;
      },
      () => {
        requestRightHeatingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetRightHeatingTolerance: tolerance },
        });
      },
    );
  };

  const setLeftCoolingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.left.cooling = tolerance;
      },
      () => {
        requestLeftCoolingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetLeftCoolingTolerance: tolerance },
        });
      },
    );
  };

  const setRightCoolingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.right.cooling = tolerance;
      },
      () => {
        requestRightCoolingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetRightCoolingTolerance: tolerance },
        });
      },
    );
  };

  const setAmbientTemperatureCalibration = (ambientTemp: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.ambient_temperature_calibration = ambientTemp;
      },
      () => {
        requestAmbientTemperatureCalibration({
          machine_identification_unique: machineIdentification,
          data: { SetAmbientTemperatureCalibration: ambientTemp },
        });
      },
    );
  };

  const setLeftPidKp = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.left.kp = value;
      },
      () => {
        requestLeftPidKp({
          machine_identification_unique: machineIdentification,
          data: { SetLeftPidKp: value },
        });
      },
    );
  };

  const setLeftPidKi = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.left.ki = value;
      },
      () => {
        requestLeftPidKi({
          machine_identification_unique: machineIdentification,
          data: { SetLeftPidKi: value },
        });
      },
    );
  };

  const setLeftPidKd = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.left.kd = value;
      },
      () => {
        requestLeftPidKd({
          machine_identification_unique: machineIdentification,
          data: { SetLeftPidKd: value },
        });
      },
    );
  };

  const setRightPidKp = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.right.kp = value;
      },
      () => {
        requestRightPidKp({
          machine_identification_unique: machineIdentification,
          data: { SetRightPidKp: value },
        });
      },
    );
  };

  const setRightPidKi = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.right.ki = value;
      },
      () => {
        requestRightPidKi({
          machine_identification_unique: machineIdentification,
          data: { SetRightPidKi: value },
        });
      },
    );
  };

  const setRightPidKd = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.right.kd = value;
      },
      () => {
        requestRightPidKd({
          machine_identification_unique: machineIdentification,
          data: { SetRightPidKd: value },
        });
      },
    );
  };

  const setLeftThermalFlowSettleDuration = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.left.thermal_delay = value;
      },
      () => {
        requestLeftThermalFlowSettleDuration({
          machine_identification_unique: machineIdentification,
          data: { SetLeftThermalFlowSettleDuration: value },
        });
      },
    );
  };

  const setRightThermalFlowSettleDuration = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.right.thermal_delay = value;
      },
      () => {
        requestRightThermalFlowSettleDuration({
          machine_identification_unique: machineIdentification,
          data: { SetRightThermalFlowSettleDuration: value },
        });
      },
    );
  };

  const setLeftPumpCooldownMinTemperature = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.left.cooldown_min_temperature =
          value;
      },
      () => {
        requestLeftPumpCooldownMinTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetLeftPumpCooldownMinTemperature: value },
        });
      },
    );
  };

  const setRightPumpCooldownMinTemperature = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.right.cooldown_min_temperature =
          value;
      },
      () => {
        requestRightPumpCooldownMinTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetRightPumpCooldownMinTemperature: value },
        });
      },
    );
  };

  // Mutation hooks
  const { request: requestAquapathMode } = useMachineMutation(
    z.object({ SetAquaPathMode: z.enum(["Standby", "Auto"]) }),
  );
  const { request: requestLeftTemperature } = useMachineMutation(
    z.object({ SetLeftTemperature: z.number() }),
  );
  const { request: requestRightTemperature } = useMachineMutation(
    z.object({ SetRightTemperature: z.number() }),
  );
  const { request: requestLeftFlow } = useMachineMutation(
    z.object({ SetLeftFlow: z.boolean() }),
  );
  const { request: requestRightFlow } = useMachineMutation(
    z.object({ SetRightFlow: z.boolean() }),
  );
  const { request: requestLeftRevolutions } = useMachineMutation(
    z.object({ SetLeftRevolutions: z.number() }),
  );
  const { request: requestRightRevolutions } = useMachineMutation(
    z.object({ SetRightRevolutions: z.number() }),
  );
  const { request: requestLeftHeatingTolerance } = useMachineMutation(
    z.object({ SetLeftHeatingTolerance: z.number() }),
  );
  const { request: requestLeftCoolingTolerance } = useMachineMutation(
    z.object({ SetLeftCoolingTolerance: z.number() }),
  );
  const { request: requestRightHeatingTolerance } = useMachineMutation(
    z.object({ SetRightHeatingTolerance: z.number() }),
  );
  const { request: requestRightCoolingTolerance } = useMachineMutation(
    z.object({ SetRightCoolingTolerance: z.number() }),
  );
  const { request: requestAmbientTemperatureCalibration } = useMachineMutation(
    z.object({ SetAmbientTemperatureCalibration: z.number() }),
  );
  const { request: requestLeftPidKp } = useMachineMutation(
    z.object({ SetLeftPidKp: z.number() }),
  );
  const { request: requestLeftPidKi } = useMachineMutation(
    z.object({ SetLeftPidKi: z.number() }),
  );
  const { request: requestLeftPidKd } = useMachineMutation(
    z.object({ SetLeftPidKd: z.number() }),
  );
  const { request: requestRightPidKp } = useMachineMutation(
    z.object({ SetRightPidKp: z.number() }),
  );
  const { request: requestRightPidKi } = useMachineMutation(
    z.object({ SetRightPidKi: z.number() }),
  );
  const { request: requestRightPidKd } = useMachineMutation(
    z.object({ SetRightPidKd: z.number() }),
  );
  const { request: requestLeftThermalFlowSettleDuration } = useMachineMutation(
    z.object({ SetLeftThermalFlowSettleDuration: z.number() }),
  );
  const { request: requestRightThermalFlowSettleDuration } = useMachineMutation(
    z.object({ SetRightThermalFlowSettleDuration: z.number() }),
  );
  const { request: requestLeftPumpCooldownMinTemperature } = useMachineMutation(
    z.object({ SetLeftPumpCooldownMinTemperature: z.number() }),
  );
  const { request: requestRightPumpCooldownMinTemperature } =
    useMachineMutation(
      z.object({ SetRightPumpCooldownMinTemperature: z.number() }),
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
    left_flow,
    right_flow,
    left_temperature,
    right_temperature,
    left_temp_reservoir,
    right_temp_reservoir,
    left_revolutions,
    right_revolutions,
    left_power,
    right_power,
    combinedPower,
    left_total_energy,
    right_total_energy,
    totalEnergyKWh,
    left_heating,
    right_heating,
    left_cooling_mode,
    right_cooling_mode,
    left_pump_cooldown_active,
    right_pump_cooldown_active,
    left_pump_cooldown_remaining,
    right_pump_cooldown_remaining,
    left_heating_startup_wait_active,
    right_heating_startup_wait_active,
    left_heating_startup_wait_remaining,
    right_heating_startup_wait_remaining,
    targetLeftTemperature,
    targetRightTemperature,

    setAquapathMode,
    setLeftTemperature,
    setRightTemperature,
    setLeftFlow,
    setRightFlow,
    setLeftRevolutions,
    setRightRevolutions,
    setLeftHeatingTolerance,
    setRightHeatingTolerance,
    setLeftCoolingTolerance,
    setRightCoolingTolerance,
    setAmbientTemperatureCalibration,
    setLeftPidKp,
    setLeftPidKi,
    setLeftPidKd,
    setRightPidKp,
    setRightPidKi,
    setRightPidKd,
    setLeftThermalFlowSettleDuration,
    setRightThermalFlowSettleDuration,
    setLeftPumpCooldownMinTemperature,
    setRightPumpCooldownMinTemperature,
  };
}
