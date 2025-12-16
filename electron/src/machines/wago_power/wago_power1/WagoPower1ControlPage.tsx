import { Page } from "@/components/Page";
import React from "react";
import { useWagoPower1Namespace } from "./wagoPower1Namespace";
import { useWagoPower1 } from "./useWagoPower1";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlCard } from "@/control/ControlCard";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

export function WagoPower1ControlPage() {
  const { voltage, current } = useWagoPower1();

  return (
    <Page>
      <ControlGrid columns={1}>
        <ControlCard title="Power">
          <TimeSeriesValueNumeric
            label="Voltage"
            unit="V"
            timeseries={voltage}
            renderValue={(value) => value.toFixed(2)}
          />
          <TimeSeriesValueNumeric
            label="Current"
            unit="mA"
            timeseries={current}
            renderValue={(value) => value.toFixed(2)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
