import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { TouchSlider } from "@/components/touch/TouchSlider"
import { useAquapath1 } from "./useAquapath";

export function Aquapath1SettingsPage() {
  const {
    state,
  } = useAquapath1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Front Fan RPM">
          <TouchSlider
            className=""
            minLabel=""
            maxLabel=""
            defaultValue={[0]}
            inverted={false}
            renderValue={(x: number)=>x.toString()}
            unit="%"
            defaultValue={[50]}
            min={0}
            value={state?.fan_states.front}
            max={100}
          />
        </ControlCard>

        <ControlCard title="Back Fan RPM">
          <TouchSlider
            className=""
            minLabel=""
            maxLabel=""
            defaultValue={[0]}
            inverted={false}
            renderValue={(x: number)=>x.toString()}
            unit="%"
            defaultValue={[50]}
            min={0}
            value={state?.fan_states.front}
            max={100}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
