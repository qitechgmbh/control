import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import React, { useState } from "react";
import { useLaser1 } from "./useLaser1";
import { ControlCard } from "@/control/ControlCard";
import { Checkbox } from "@/components/ui/checkbox-selection";

export function Buffer1SettingsPage() {
  const {
    state,
  } = useLaser1();

  const [ checked, setChecked ] = useState(false);

  return (
    <Page>
      <ControlGrid>
        <ControlCard>
          <Checkbox
            label="Stop Winder on diameter tolerance break"
            checked={checked}
            onChange={(e) => setChecked(e.target.checked)}
            />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
