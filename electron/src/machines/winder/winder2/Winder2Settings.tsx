import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import React from "react";
import { useWinder2 } from "./useWinder";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";

export function Winder1SettingPage() {
  const { spoolState, spoolSetSpeedMax, spoolSetSpeedMin } = useWinder2();
  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Spool">
          <Label label="Min Speed">
            <EditValue
              value={spoolState?.data.speed_min}
              title={"Min Speed"}
              unit="rpm"
              step={100}
              min={10}
              max={Math.min(spoolState?.data.speed_max || 1250, 1250)}
              defaultValue={100}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={(value) => spoolSetSpeedMin(value)}
            />
          </Label>
          <Label label="Max Speed">
            <EditValue
              value={spoolState?.data.speed_max}
              title={"Max Speed"}
              unit="rpm"
              min={Math.max(spoolState?.data.speed_min || 0, 100)}
              step={100}
              max={1250}
              defaultValue={1250}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={(value) => spoolSetSpeedMax(value)}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
