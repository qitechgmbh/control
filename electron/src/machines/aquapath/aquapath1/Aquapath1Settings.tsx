import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { TouchSlider } from "@/components/touch/TouchSlider"

export function Aquapath1SettingsPage() {
  return (
    <Page>
      <ControlGrid columns={1}>
        <ControlCard title="Adjust Fan RPM">
          <TouchSlider
            
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
