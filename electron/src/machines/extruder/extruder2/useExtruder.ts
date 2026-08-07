import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { extruder2 } from "@/machines/properties";
import { extruder2Route } from "@/routes/routes";
import { z } from "zod";
import { StateEvent, Mode, useExtruder2Namespace } from "./extruder2Namespace";
import { TuneZone } from "../temperatureAutoTuneSchema";
import { useEffect, useMemo } from "react";
import { produce } from "immer";

export function useExtruder2() {
  const { serial: serialString } = extruder2Route.useParams();

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
      machine_identification: extruder2.machine_identification,
      serial,
    };
  }, [serialString]);

  // Get consolidated state and live values from namespace
  const {
    state,
    defaultState,
    tuneTrace,
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

    // Target value history for graph target lines
    targetPressure,
    targetScrewRpm,
    targetNozzleTemperature,
    targetFrontTemperature,
    targetMiddleTemperature,
    targetBackTemperature,
  } = useExtruder2Namespace(machineIdentification);

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

  const setTemperatureTargetEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.extruder_settings_state.nozzle_temperature_target_enabled =
          enabled;
      },
      () =>
        requestNozzleTemperatureTargetEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetNozzleTemperatureTargetEnabled: enabled },
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

  /**
   * Debug/test measure: drive a heating zone at a fixed wattage instead of regulating it.
   * The zone runs open-loop while enabled and can overshoot its target temperature.
   */
  const setHeatingPowerOverride = (
    zone: "front" | "middle" | "back" | "nozzle",
    enabled: boolean,
    watts: number,
  ) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_power_override_states[zone].enabled = enabled;
        current.data.heating_power_override_states[zone].watts = watts;
      },
      () =>
        requestHeatingPowerOverride({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingPowerOverride: { zone, enabled, watts } },
        }),
    );
  };

  const resetInverter = () => {
    // No optimistic update needed for reset
    requestResetInverter({
      machine_identification_unique: machineIdentification,
      data: { ResetInverter: true },
    });
  };

  const startPressurePidAutoTune = (
    tuneDelta: number,
    frequencyStepHz: number,
  ) => {
    requestStartPressurePidAutoTune({
      machine_identification_unique: machineIdentification,
      data: {
        StartPressurePidAutoTune: {
          tune_delta: tuneDelta,
          frequency_step_hz: frequencyStepHz,
        },
      },
    });
  };

  const stopPressurePidAutoTune = () => {
    requestStopPressurePidAutoTune({
      machine_identification_unique: machineIdentification,
      data: { StopPressurePidAutoTune: {} },
    });
  };

  /**
   * Set all three gains for one zone in a single mutation.
   *
   * `setTemperaturePidValue` sends one gain at a time and rebuilds the payload from optimistic
   * state on each call, so applying a whole preset that way races with itself.
   * `SetTemperaturePidSettings` already carries all three, making a full preset four mutations
   * instead of twelve.
   */
  const setTemperaturePidZone = (
    zone: TuneZone,
    gains: { kp: number; ki: number; kd: number },
  ) => {
    updateStateOptimistically(
      (current) => {
        current.data.pid_settings.temperature[zone].kp = gains.kp;
        current.data.pid_settings.temperature[zone].ki = gains.ki;
        current.data.pid_settings.temperature[zone].kd = gains.kd;
      },
      () =>
        requestTemperaturePidSettings({
          machine_identification_unique: machineIdentification,
          data: { SetTemperaturePidSettings: { ...gains, zone } },
        }),
    );
  };

  /**
   * Start an IMC step test on one heating zone. The backend refuses unless the machine is in Heat
   * mode with the screw stopped, and unless the zone has enough duty headroom for the step.
   */
  const startTemperaturePidAutoTune = (
    zone: TuneZone,
    stepDuty: number,
    maxRiseCelsius: number,
    lambdaFactor: number,
  ) => {
    requestStartTemperaturePidAutoTune({
      machine_identification_unique: machineIdentification,
      data: {
        StartTemperaturePidAutoTune: {
          zone,
          step_duty: stepDuty,
          max_rise_celsius: maxRiseCelsius,
          lambda_factor: lambdaFactor,
        },
      },
    });
  };

  const stopTemperaturePidAutoTune = () => {
    requestStopTemperaturePidAutoTune({
      machine_identification_unique: machineIdentification,
      data: { StopTemperaturePidAutoTune: {} },
    });
  };

  /** Push a completed run's gains into the tuned zone. */
  const applyTemperatureAutoTuneResult = (form: "pi" | "pid") => {
    requestApplyTemperatureAutoTuneResult({
      machine_identification_unique: machineIdentification,
      data: { ApplyTemperatureAutoTuneResult: { form } },
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

  const { request: requestNozzleTemperatureTargetEnabled } = useMachineMutation(
    z.object({ SetNozzleTemperatureTargetEnabled: z.boolean() }),
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

  const { request: requestHeatingPowerOverride } = useMachineMutation(
    z.object({
      SetHeatingPowerOverride: z.object({
        zone: z.string(),
        enabled: z.boolean(),
        watts: z.number(),
      }),
    }),
  );

  const { request: requestResetInverter } = useMachineMutation(
    z.object({ ResetInverter: z.boolean() }),
  );

  const { request: requestStartPressurePidAutoTune } = useMachineMutation(
    z.object({
      StartPressurePidAutoTune: z.object({
        tune_delta: z.number(),
        frequency_step_hz: z.number(),
      }),
    }),
  );

  const { request: requestStopPressurePidAutoTune } = useMachineMutation(
    z.object({ StopPressurePidAutoTune: z.object({}) }),
  );

  const { request: requestStartTemperaturePidAutoTune } = useMachineMutation(
    z.object({
      StartTemperaturePidAutoTune: z.object({
        zone: z.string(),
        step_duty: z.number(),
        max_rise_celsius: z.number(),
        lambda_factor: z.number(),
      }),
    }),
  );

  const { request: requestStopTemperaturePidAutoTune } = useMachineMutation(
    z.object({ StopTemperaturePidAutoTune: z.object({}) }),
  );

  const { request: requestApplyTemperatureAutoTuneResult } = useMachineMutation(
    z.object({
      ApplyTemperatureAutoTuneResult: z.object({ form: z.string() }),
    }),
  );

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Recorded curve from the running or most recent temperature auto-tune
    tuneTrace,

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

    // Target value history for graph target lines
    targetPressure,
    targetScrewRpm,
    targetNozzleTemperature,
    targetFrontTemperature,
    targetMiddleTemperature,
    targetBackTemperature,

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
    setTemperatureTargetEnabled,
    setHeatingPowerOverride,
    resetInverter,
    startPressurePidAutoTune,
    stopPressurePidAutoTune,
    setTemperaturePidZone,
    startTemperaturePidAutoTune,
    stopTemperaturePidAutoTune,
    applyTemperatureAutoTuneResult,
  };
}
