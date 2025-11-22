import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { useXtremZebra1 } from "./useXtremZebra";

export function XtremZebraControlPage() {
  const {} = useXtremZebra1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Weight"></ControlCard>
      </ControlGrid>
    </Page>
  );
}
