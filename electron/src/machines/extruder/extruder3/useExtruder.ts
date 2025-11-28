import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { extruder3 } from "@/machines/properties";
import { extruder3Route } from "@/routes/routes";
import { z } from "zod";
import { StateEvent, Mode, useExtruder3Namespace } from "./extruder3Namespace";
import { useEffect, useMemo } from "react";
import { produce } from "immer";

export function useExtruder3() {
  const { serial: serialString } = extruder3Route.useParams();

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
      machine_identification: extruder3.machine_identification,
      serial,
    };
  }, [serialString]);

  // Get consolidated state and live values from namespace
  const {
    state,
    defaultState,
    motorCurrent,
    motorFrequency,
    motorScrewRpm,
    motorPower,
    pressure,

    nozzleTemperature,
    frontTemperature,
    backTemperature,
    middleTemperature,
    nozzlePower,
    frontPower,
    middlePower,
    backPower,
    combinedPower,
    totalEnergyKWh,
  } = useExtruder3Namespace(machineIdentification);

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
  const setInverterRotationDirection = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.rotation_state.forward = forward;
      },
      () =>
        requestInverterRotationDirection({
          machine_identification_unique: machineIdentification,
          data: { SetInverterRotationDirection: forward },
        }),
    );
  };

  const setExtruderMode = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () =>
        requestExtruderMode({
          machine_identification_unique: machineIdentification,
          data: { SetExtruderMode: mode },
        }),
    );
  };

  const setInverterRegulation = (usesRpm: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.regulation_state.uses_rpm = usesRpm;
      },
      () =>
        requestInverterRegulation({
          machine_identification_unique: machineIdentification,
          data: { SetInverterRegulation: usesRpm },
        }),
    );
  };

  const setInverterTargetRpm = (rpm: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.screw_state.target_rpm = rpm;
      },
      () =>
        requestInverterTargetRpm({
          machine_identification_unique: machineIdentification,
          data: { SetInverterTargetRpm: rpm },
        }),
    );
  };

  const setInverterTargetPressure = (pressure: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pressure_state.target_bar = pressure;
      },
      () =>
        requestInverterTargetPressure({
          machine_identification_unique: machineIdentification,
          data: { SetInverterTargetPressure: pressure },
        }),
    );
  };

  const setNozzleHeatingTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.nozzle.target_temperature = temperature;
      },
      () =>
        requestNozzleHeatingTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetNozzleHeatingTemperature: temperature },
        }),
    );
  };

  const setFrontHeatingTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.front.target_temperature = temperature;
      },
      () =>
        requestFrontHeatingTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetFrontHeatingTargetTemperature: temperature },
        }),
    );
  };

  const setBackHeatingTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.back.target_temperature = temperature;
      },
      () =>
        requestBackHeatingTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetBackHeatingTargetTemperature: temperature },
        }),
    );
  };

  const setMiddleHeatingTemperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.middle.target_temperature = temperature;
      },
      () =>
        requestMiddleHeatingTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetMiddleHeatingTemperature: temperature },
        }),
    );
  };

  const setExtruderPressureLimit = (pressureLimit: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.extruder_settings_state.pressure_limit = pressureLimit;
      },
      () =>
        requestExtruderPressureLimit({
          machine_identification_unique: machineIdentification,
          data: { SetExtruderPressureLimit: pressureLimit },
        }),
    );
  };

  const setExtruderPressureLimitEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.extruder_settings_state.pressure_limit_enabled = enabled;
      },
      () =>
        requestExtruderPressureLimitEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetExtruderPressureLimitIsEnabled: enabled },
        }),
    );
  };

  const setPressurePidKp = (kp: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.pressure.kp = kp;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.pressure,
            (draft) => {
              draft.kp = kp;
            },
          );
          requestPressurePidSettings({
            machine_identification_unique: machineIdentification,
            data: { SetPressurePidSettings: settings },
          });
        }
      },
    );
  };

  const setPressurePidKi = (ki: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.pressure.ki = ki;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.pressure,
            (draft) => {
              draft.ki = ki;
            },
          );
          requestPressurePidSettings({
            machine_identification_unique: machineIdentification,
            data: { SetPressurePidSettings: settings },
          });
        }
      },
    );
  };

  const setPressurePidKd = (kd: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.pressure.kd = kd;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.pressure,
            (draft) => {
              draft.kd = kd;
            },
          );
          requestPressurePidSettings({
            machine_identification_unique: machineIdentification,
            data: { SetPressurePidSettings: settings },
          });
        }
      },
    );
  };

  const setTemperaturePidValue = (
    zone: "front" | "middle" | "back" | "nozzle",
    key: "kp" | "ki" | "kd",
    value: number,
  ) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.temperature[zone][key] = value;
      },
      () => {
        const currentState = stateOptimistic.value;
        if (currentState) {
          const settings = produce(
            currentState.data.pid_settings.temperature[zone],
            (draft) => {
              draft[key] = value;
            },
          );
          requestTemperaturePidSettings({
            machine_identification_unique: machineIdentification,
            data: { SetTemperaturePidSettings: settings },
          });
        }
      },
    );
  };

  const resetInverter = () => {
    // No optimistic update needed for reset
    requestResetInverter({
      machine_identification_unique: machineIdentification,
      data: { ResetInverter: true },
    });
  };

  // Mutation hooks
  const { request: requestInverterRotationDirection } = useMachineMutation(
    z.object({ SetInverterRotationDirection: z.boolean() }),
  );

  const { request: requestExtruderMode } = useMachineMutation(
    z.object({ SetExtruderMode: z.enum(["Heat", "Extrude", "Standby"]) }),
  );

  const { request: requestInverterRegulation } = useMachineMutation(
    z.object({ SetInverterRegulation: z.boolean() }),
  );

  const { request: requestInverterTargetRpm } = useMachineMutation(
    z.object({ SetInverterTargetRpm: z.number() }),
  );

  const { request: requestInverterTargetPressure } = useMachineMutation(
    z.object({ SetInverterTargetPressure: z.number() }),
  );

  const { request: requestNozzleHeatingTemperature } = useMachineMutation(
    z.object({ SetNozzleHeatingTemperature: z.number() }),
  );

  const { request: requestFrontHeatingTemperature } = useMachineMutation(
    z.object({ SetFrontHeatingTargetTemperature: z.number() }),
  );

  const { request: requestBackHeatingTemperature } = useMachineMutation(
    z.object({ SetBackHeatingTargetTemperature: z.number() }),
  );

  const { request: requestMiddleHeatingTemperature } = useMachineMutation(
    z.object({ SetMiddleHeatingTemperature: z.number() }),
  );

  const { request: requestExtruderPressureLimit } = useMachineMutation(
    z.object({ SetExtruderPressureLimit: z.number() }),
  );

  const { request: requestExtruderPressureLimitEnabled } = useMachineMutation(
    z.object({ SetExtruderPressureLimitIsEnabled: z.boolean() }),
  );

  const { request: requestPressurePidSettings } = useMachineMutation(
    z.object({
      SetPressurePidSettings: z.object({
        ki: z.number(),
        kp: z.number(),
        kd: z.number(),
      }),
    }),
  );

  const { request: requestTemperaturePidSettings } = useMachineMutation(
    z.object({
      SetTemperaturePidSettings: z.object({
        ki: z.number(),
        kp: z.number(),
        kd: z.number(),
        zone: z.string(),
      }),
    }),
  );

  const { request: requestResetInverter } = useMachineMutation(
    z.object({ ResetInverter: z.boolean() }),
  );

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Individual live values (TimeSeries)
    motorCurrent,
    motorFrequency,
    motorScrewRpm,
    motorPower,

    pressure,
    nozzleTemperature,
    frontTemperature,
    backTemperature,
    middleTemperature,
    nozzlePower,
    frontPower,
    middlePower,
    backPower,
    combinedPower,
    totalEnergyKWh,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions (verb-first)
    setInverterRotationDirection,
    setExtruderMode,
    setInverterRegulation,
    setInverterTargetRpm,
    setInverterTargetPressure,
    setNozzleHeatingTemperature,
    setFrontHeatingTemperature,
    setBackHeatingTemperature,
    setMiddleHeatingTemperature,
    setExtruderPressureLimit,
    setExtruderPressureLimitEnabled,
    setPressurePidKp,
    setPressurePidKi,
    setPressurePidKd,
    setTemperaturePidValue,
    resetInverter,
  };
}
