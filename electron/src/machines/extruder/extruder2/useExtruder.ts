import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique, extruder2 } from "@/machines/types";
import { extruder2Route } from "@/routes/routes";
import { z } from "zod";
import {
  Heating,
  Mode,
  heatingStateDataSchema,
  useExtruder2Namespace,
} from "./extruder2Namespace";
import { useEffect, useMemo, useState } from "react";
import { TimeSeries } from "@/lib/timeseries";

export function useExtruder2() {
  const { serial: serialString } = extruder2Route.useParams();

  // Memoize the machine identification to keep it stable between renders
  const machineIdentification = useMemo(() => {
    const serial = parseInt(serialString); // Use 0 as fallback if NaN

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        vendor: 0,
        machine: 0,
        serial: 0,
      };
    }

    return {
      ...extruder2.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const inverter = useInverter(machineIdentification);
  const mode = useMode(machineIdentification);
  const motor = useMotor(machineIdentification);
  const heating = useHeating(machineIdentification);

  return {
    ...inverter,
    ...mode,
    ...motor,
    ...heating,
  };
}

export function useInverter(
  machine_identification_unique: MachineIdentificationUnique,
): {
  inverterSetRotation: (forward: boolean) => void;
  rotationState: boolean | undefined;
} {
  const state = useStateOptimistic();

  const schema = z.object({ SetRotation: z.boolean() });
  const { request: requestRotation } = useMachineMutation(schema);
  const inverterSetRotation = async (forward: boolean) => {
    state.setOptimistic(forward);
    requestRotation({
      machine_identification_unique,
      data: { SetRotation: forward },
    });
  };
  const { rotationState } = useExtruder2Namespace(
    machine_identification_unique,
  );
  useEffect(() => {
    if (rotationState?.data) {
      state.setReal(rotationState.data.forward);
    }
  }, [rotationState?.data.forward]);
  return { inverterSetRotation, rotationState: rotationState?.data.forward };
}

export function useMode(
  machine_identification_unique: MachineIdentificationUnique,
): {
  mode: Mode | undefined;
  SetMode: (value: Mode) => void;
  modeIsLoading: boolean;
  modeIsDisabled: boolean;
} {
  const state = useStateOptimistic<Mode>();

  // Write path
  const schema = z.object({
    SetMode: z.enum(["Heat", "Extrude", "Standby"]),
  });

  const { request } = useMachineMutation(schema);

  const SetMode = async (value: Mode) => {
    state.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { SetMode: value },
    })
      .then((response) => {
        if (!response.success) state.resetToReal();
      })
      .catch(() => state.resetToReal());
  };

  // Read path
  const { modeState } = useExtruder2Namespace(machine_identification_unique);
  useEffect(() => {
    if (modeState?.data) {
      state.setReal(modeState.data.mode);
    }
  }, [modeState]);

  return {
    mode: state.value,
    SetMode,
    modeIsLoading: state.isOptimistic || !state.isInitialized,
    modeIsDisabled: state.isOptimistic || !state.isInitialized,
  };
}

export function useMotor(
  machine_identification_unique: MachineIdentificationUnique,
): {
  rpm: number | undefined;
  bar: number | undefined;
  uses_rpm: boolean | undefined;
  SetTargetRpm: (rpm: number) => void;
  SetRegulation: (usesRpm: boolean) => void;
  // SetTargetPressure: (bar: number) => void;
  // SetRegulation: (usesRpm: boolean) => void;
} {
  const state = useStateOptimistic<number>();
  const SetTargetRpmSchema = z.object({
    SetTargetRpm: z.number(),
  });

  const SetRegulationSchema = z.object({
    SetRegulation: z.boolean(),
  });
  const { motorRpmState } = useExtruder2Namespace(
    machine_identification_unique,
  );
  const { request: reqestTargetRpm } = useMachineMutation(SetTargetRpmSchema);
  const SetTargetRpm = async (value: number) => {
    state.setOptimistic(value);
    reqestTargetRpm({
      machine_identification_unique,
      data: { SetTargetRpm: value },
    })
      .then((response) => {
        if (!response.success) state.resetToReal();
      })
      .catch(() => state.resetToReal());
  };

  const regulationState = useStateOptimistic<boolean>();
  const { request: regulationRequest } =
    useMachineMutation(SetRegulationSchema);

  const SetRegulation = async (value: boolean) => {
    regulationState.setOptimistic(value);
    regulationRequest({
      machine_identification_unique,
      data: { SetRegulation: value },
    })
      .then((response) => {
        if (!response.success) state.resetToReal();
      })
      .catch(() => state.resetToReal());
  };

  useEffect(() => {
    if (motorRpmState?.data) {
      state.setReal(motorRpmState.data.rpm);
    }
  }, [motorRpmState]);

  return {
    uses_rpm: regulationState.value,
    bar: 0,
    rpm: state.value,
    SetTargetRpm,
    SetRegulation,
  };
}

export function useHeating(
  machine_identification_unique: MachineIdentificationUnique,
): {
  SetHeatingFront: (value: Heating) => void;
  SetHeatingBack: (value: Heating) => void;
  SetHeatingMiddle: (value: Heating) => void;
  frontHeatingState: Heating | undefined;
  backHeatingState: Heating | undefined;
  middleHeatingState: Heating | undefined;
} {
  const frontHeatingState = useStateOptimistic<Heating>();
  const backHeatingState = useStateOptimistic<Heating>();
  const middleHeatingState = useStateOptimistic<Heating>();

  const SetFrontHeatingSchema = z.object({
    SetFrontHeating: z.object({
      temperature: z.number(),
      heating: z.boolean(),
      target_temperature: z.number(),
    }),
  });

  const SetBackHeatingSchema = z.object({
    SetBackHeating: z.object({
      temperature: z.number(),
      heating: z.boolean(),
      target_temperature: z.number(),
    }),
  });

  const SetMiddleHeatingSchema = z.object({
    SetMiddleHeating: z.object({
      temperature: z.number(),
      heating: z.boolean(),
      target_temperature: z.number(),
    }),
  });
  const SetHeatingFront = async (value: Heating) => {
    const { request } = useMachineMutation(SetFrontHeatingSchema);
    frontHeatingState.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { SetFrontHeating: value },
    })
      .then((response) => {
        if (!response.success) frontHeatingState.resetToReal();
      })
      .catch(() => frontHeatingState.resetToReal());
  };

  const SetHeatingBack = async (value: Heating) => {
    const { request } = useMachineMutation(SetBackHeatingSchema);
    backHeatingState.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { SetBackHeating: value },
    })
      .then((response) => {
        if (!response.success) backHeatingState.resetToReal();
      })
      .catch(() => backHeatingState.resetToReal());
  };

  const SetHeatingMiddle = async (value: Heating) => {
    const { request } = useMachineMutation(SetMiddleHeatingSchema);
    frontHeatingState.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { SetMiddleHeating: value },
    })
      .then((response) => {
        if (!response.success) middleHeatingState.resetToReal();
      })
      .catch(() => middleHeatingState.resetToReal());
  };

  // Read path
  const { heatingFrontState, heatingBackState, heatingMiddleState } =
    useExtruder2Namespace(machine_identification_unique);

  useEffect(() => {
    if (heatingFrontState?.data) {
      frontHeatingState.setReal(heatingFrontState.data);
    }
    if (heatingBackState?.data) {
      backHeatingState.setReal(heatingBackState.data);
    }
    if (heatingMiddleState?.data) {
      middleHeatingState.setReal(heatingMiddleState.data);
    }
  }, [frontHeatingState, backHeatingState, middleHeatingState]);

  return {
    SetHeatingFront,
    SetHeatingBack,
    SetHeatingMiddle,
    frontHeatingState: frontHeatingState.value,
    backHeatingState: backHeatingState.value,
    middleHeatingState: middleHeatingState.value,
  };
}
