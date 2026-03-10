import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { ToggleButton } from "@/components/touch/TouchToggleButton";
import { useLaser1 } from "./useLaser1";

export function Laser1SettingsPage() {

  const {
    state,
    toggleGlobalWarning,
    isLoading,
    isDisabled,
  } = useLaser1();

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Global Error Warning">
          <ToggleButton
            enabled={state?.laser_state.global_warning ?? false}
            onToggle={toggleGlobalWarning}
            label="Global Warning"
            iconOn="lu:BellRing"
            iconOff="lu:BellOff"
            isLoading={isLoading}
            disabled={isDisabled}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
