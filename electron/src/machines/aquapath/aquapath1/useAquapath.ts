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
    front_temperature,
    back_temperature,
    front_flow,
    back_flow,
    front_temp_reservoir,
    back_temp_reservoir,
    front_revolutions,
    back_revolutions,
    front_power,
    back_power,
    front_total_energy,
    back_total_energy,
    front_heating,
    back_heating,
    front_cooling_mode,
    back_cooling_mode,
    front_pump_cooldown_active,
    back_pump_cooldown_active,
    front_pump_cooldown_remaining,
    back_pump_cooldown_remaining,
    targetFrontTemperature,
    targetBackTemperature,
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

  const setFrontRevolutions = (revolutions: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.fan_states.front.revolutions = revolutions;
      },
      () => {
        requestFrontRevolutions({
          machine_identification_unique: machineIdentification,
          data: { SetFrontRevolutions: revolutions },
        });
      },
    );
  };

  const setBackRevolutions = (revolutions: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.fan_states.back.revolutions = revolutions;
      },
      () =>
        requestBackRevolutions({
          machine_identification_unique: machineIdentification,
          data: { SetBackRevolutions: revolutions },
        }),
    );
  };

  const setFrontHeatingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.front.heating = tolerance;
      },
      () => {
        requestFrontHeatingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetFrontHeatingTolerance: tolerance },
        });
      },
    );
  };

  const setBackHeatingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.back.heating = tolerance;
      },
      () => {
        requestBackHeatingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetBackHeatingTolerance: tolerance },
        });
      },
    );
  };

  const setFrontCoolingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.front.cooling = tolerance;
      },
      () => {
        requestFrontCoolingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetFrontCoolingTolerance: tolerance },
        });
      },
    );
  };

  const setBackCoolingTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.tolerance_states.back.cooling = tolerance;
      },
      () => {
        requestBackCoolingTolerance({
          machine_identification_unique: machineIdentification,
          data: { SetBackCoolingTolerance: tolerance },
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

  const setFrontPidKp = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.front.kp = value;
      },
      () => {
        requestFrontPidKp({
          machine_identification_unique: machineIdentification,
          data: { SetFrontPidKp: value },
        });
      },
    );
  };

  const setFrontPidKi = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.front.ki = value;
      },
      () => {
        requestFrontPidKi({
          machine_identification_unique: machineIdentification,
          data: { SetFrontPidKi: value },
        });
      },
    );
  };

  const setFrontPidKd = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.front.kd = value;
      },
      () => {
        requestFrontPidKd({
          machine_identification_unique: machineIdentification,
          data: { SetFrontPidKd: value },
        });
      },
    );
  };

  const setBackPidKp = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.back.kp = value;
      },
      () => {
        requestBackPidKp({
          machine_identification_unique: machineIdentification,
          data: { SetBackPidKp: value },
        });
      },
    );
  };

  const setBackPidKi = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.back.ki = value;
      },
      () => {
        requestBackPidKi({
          machine_identification_unique: machineIdentification,
          data: { SetBackPidKi: value },
        });
      },
    );
  };

  const setBackPidKd = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_states.back.kd = value;
      },
      () => {
        requestBackPidKd({
          machine_identification_unique: machineIdentification,
          data: { SetBackPidKd: value },
        });
      },
    );
  };

  const setFrontThermalFlowSettleDuration = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.front.shared_delay = value;
      },
      () => {
        requestFrontThermalFlowSettleDuration({
          machine_identification_unique: machineIdentification,
          data: { SetFrontThermalFlowSettleDuration: value },
        });
      },
    );
  };

  const setBackThermalFlowSettleDuration = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.back.shared_delay = value;
      },
      () => {
        requestBackThermalFlowSettleDuration({
          machine_identification_unique: machineIdentification,
          data: { SetBackThermalFlowSettleDuration: value },
        });
      },
    );
  };

  const setFrontPumpCooldownMinTemperature = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.front.cooldown_min_temperature =
          value;
      },
      () => {
        requestFrontPumpCooldownMinTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetFrontPumpCooldownMinTemperature: value },
        });
      },
    );
  };

  const setBackPumpCooldownMinTemperature = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.thermal_safety_states.back.cooldown_min_temperature =
          value;
      },
      () => {
        requestBackPumpCooldownMinTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetBackPumpCooldownMinTemperature: value },
        });
      },
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
  const { request: requestFrontRevolutions } = useMachineMutation(
    z.object({ SetFrontRevolutions: z.number() }),
  );
  const { request: requestBackRevolutions } = useMachineMutation(
    z.object({ SetBackRevolutions: z.number() }),
  );
  const { request: requestFrontHeatingTolerance } = useMachineMutation(
    z.object({ SetFrontHeatingTolerance: z.number() }),
  );
  const { request: requestFrontCoolingTolerance } = useMachineMutation(
    z.object({ SetFrontCoolingTolerance: z.number() }),
  );
  const { request: requestBackHeatingTolerance } = useMachineMutation(
    z.object({ SetBackHeatingTolerance: z.number() }),
  );
  const { request: requestBackCoolingTolerance } = useMachineMutation(
    z.object({ SetBackCoolingTolerance: z.number() }),
  );
  const { request: requestAmbientTemperatureCalibration } = useMachineMutation(
    z.object({ SetAmbientTemperatureCalibration: z.number() }),
  );
  const { request: requestFrontPidKp } = useMachineMutation(
    z.object({ SetFrontPidKp: z.number() }),
  );
  const { request: requestFrontPidKi } = useMachineMutation(
    z.object({ SetFrontPidKi: z.number() }),
  );
  const { request: requestFrontPidKd } = useMachineMutation(
    z.object({ SetFrontPidKd: z.number() }),
  );
  const { request: requestBackPidKp } = useMachineMutation(
    z.object({ SetBackPidKp: z.number() }),
  );
  const { request: requestBackPidKi } = useMachineMutation(
    z.object({ SetBackPidKi: z.number() }),
  );
  const { request: requestBackPidKd } = useMachineMutation(
    z.object({ SetBackPidKd: z.number() }),
  );
  const { request: requestFrontThermalFlowSettleDuration } = useMachineMutation(
    z.object({ SetFrontThermalFlowSettleDuration: z.number() }),
  );
  const { request: requestBackThermalFlowSettleDuration } = useMachineMutation(
    z.object({ SetBackThermalFlowSettleDuration: z.number() }),
  );
  const { request: requestFrontPumpCooldownMinTemperature } =
    useMachineMutation(
      z.object({ SetFrontPumpCooldownMinTemperature: z.number() }),
    );
  const { request: requestBackPumpCooldownMinTemperature } = useMachineMutation(
    z.object({ SetBackPumpCooldownMinTemperature: z.number() }),
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
    front_revolutions,
    back_revolutions,
    front_power,
    back_power,
    front_total_energy,
    back_total_energy,
    front_heating,
    back_heating,
    front_cooling_mode,
    back_cooling_mode,
    front_pump_cooldown_active,
    back_pump_cooldown_active,
    front_pump_cooldown_remaining,
    back_pump_cooldown_remaining,
    targetFrontTemperature,
    targetBackTemperature,

    setAquapathMode,
    setFrontTemperature,
    setBackTemperature,
    setFrontFlow,
    setBackFlow,
    setFrontRevolutions,
    setBackRevolutions,
    setFrontHeatingTolerance,
    setBackHeatingTolerance,
    setFrontCoolingTolerance,
    setBackCoolingTolerance,
    setAmbientTemperatureCalibration,
    setFrontPidKp,
    setFrontPidKi,
    setFrontPidKd,
    setBackPidKp,
    setBackPidKi,
    setBackPidKd,
    setFrontThermalFlowSettleDuration,
    setBackThermalFlowSettleDuration,
    setFrontPumpCooldownMinTemperature,
    setBackPumpCooldownMinTemperature,
  };
}
