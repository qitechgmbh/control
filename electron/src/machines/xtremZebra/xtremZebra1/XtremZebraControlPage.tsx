import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { useXtremZebra1 } from "./useXtremZebra";

export function XtremZebraControlPage() {
  const { state, total_weight, current_weight } = useXtremZebra1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Current Weight">
          <TimeSeriesValueNumeric
            label=""
            unit="kg"
            timeseries={current_weight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
        <ControlCard title="Total Weight">
          <TimeSeriesValueNumeric
            label=""
            unit="kg"
            timeseries={total_weight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
