import { DryerOverviewPage } from "../DryerOverviewPage";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerV1 } from "@/machines/properties";
import { dryerV1SerialRoute } from "@/routes/routes";
import React, { useMemo } from "react";

export function DryerV1OverviewPage() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  return <DryerOverviewPage machineIdentification={machineIdentification} />;
}
