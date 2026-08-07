import { DryerControlPage } from "../DryerControlPage";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerSmart } from "@/machines/properties";
import { dryerSmartSerialRoute } from "@/routes/routes";
import { useDryerSmartMaterialStore } from "../materialStore";
import React, { useMemo } from "react";

export function DryerSmartControlPage() {
  const { serial: serialString } = dryerSmartSerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerSmart.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const materialStore = useDryerSmartMaterialStore();

  return (
    <DryerControlPage
      machineIdentification={machineIdentification}
      materialStore={materialStore}
    />
  );
}
