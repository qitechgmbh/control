import { DryerControlPage } from "../DryerControlPage";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerV1 } from "@/machines/properties";
import { dryerV1SerialRoute } from "@/routes/routes";
import { useDryerV1MaterialStore } from "../materialStore";
import React, { useMemo } from "react";

export function DryerV1ControlPage() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const materialStore = useDryerV1MaterialStore();

  return (
    <DryerControlPage
      machineIdentification={machineIdentification}
      materialStore={materialStore}
    />
  );
}
