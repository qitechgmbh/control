import { DryerOverviewPage } from "../DryerOverviewPage";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerSmart } from "@/machines/properties";
import { dryerSmartSerialRoute } from "@/routes/routes";
import React, { useMemo } from "react";

export function DryerSmartOverviewPage() {
  const { serial: serialString } = dryerSmartSerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerSmart.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  return <DryerOverviewPage machineIdentification={machineIdentification} />;
}
