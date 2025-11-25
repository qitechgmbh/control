import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { useXtremZebra1 } from "./useXtremZebra";

export function XtremZebraControlPage() {
  const { weight, state } = useXtremZebra1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Scale">
          <TimeSeriesValueNumeric
            label="Current Weight"
            unit="kg"
            timeseries={weight}
            renderValue={(value) => value.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Total Weight"
            unit="kg"
            timeseries={weight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
