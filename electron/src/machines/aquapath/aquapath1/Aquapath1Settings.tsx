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
            className="w-[48rem]"
            value={state?.fan_states.front ? [state.fan_states.front] : undefined}
            onValueChange={(x: Number) => "${x}"}
            min={0}
            max={100}
            step={1}
            inverted={false}
            unit={"%"}
            minLabel={"min"}
            maxLabel={"max"}
          />
        </ControlCard>

        <ControlCard title="Front Fan RPM">
          <TouchSlider
            className="w-[48rem]"
            value={state?.fan_states.front ? [state.fan_states.front] : undefined}
            onValueChange={(x: Number) => "${x}"}
            min={0}
            max={100}
            step={1}
            inverted={false}
            unit={"%"}
            minLabel={"min"}
            maxLabel={"max"}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
