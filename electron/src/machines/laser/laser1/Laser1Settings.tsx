import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";
import { useLaser1 } from "./useLaser1";
import { ControlCard } from "@/control/ControlCard";
import { Checkbox } from "@/components/ui/checkbox-selection";

export function Laser1SettingsPage() {
  const { state, setAutoStopOnOutOfTolerance } = useLaser1();

  return (
    <Page>
      <ControlGrid>
        <ControlCard>
          <Checkbox
            label="Stop Winder on diameter tolerance break"
            checked={state?.laser_state.auto_stop_on_out_of_tolerance}
            onChange={(e) => setAutoStopOnOutOfTolerance(e.target.checked)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
