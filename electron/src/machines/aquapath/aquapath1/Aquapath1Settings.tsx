import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { TouchSlider } from "@/components/touch/TouchSlider"

export function Aquapath1SettingsPage() {
  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Fan 1 RPM">
          <TouchSlider
            
          />
        </ControlCard>

        <ControlCard title="Fan 1 RPM">
          <TouchSlider
            
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
