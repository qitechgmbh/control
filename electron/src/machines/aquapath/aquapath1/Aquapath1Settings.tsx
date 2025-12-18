import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { TouchSlider } from "@/components/touch/TouchSlider"
import { useAquapath1 } from "./useAquapath";

export function Aquapath1SettingsPage() {
  const {
    state,
    front_revolutions,
    back_revolutions,
    setFrontRevolutions,
    setBackRevolutions,
  } = useAquapath1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Front Fan RPM">
          <div className="grid grid-rows-2 gap-8">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolutions"
                unit="%"
                timeseries={front_revolutions}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <TouchSlider
                className=""
                value={state?.fan_states.front.revolutions ? [state.fan_states.front.revolutions] : undefined}
                onValueChange={(revolutions: Number) => { setFrontRevolutions(revolutions); }}
                min={0}
                max={100}
                step={1}
                inverted={false}
                unit={"%"}
                minLabel={"min"}
                maxLabel={"max"}
              />
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Back Fan RPM">
          <div className="grid grid-rows-2 gap-4">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolutions"
                unit="%"
                timeseries={back_revolutions}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <TouchSlider
                className=""
                value={state?.fan_states.back.revolutions? [state.fan_states.back.revolutions] : undefined}
                onValueChange={(revolutions: Number) => { setBackRevolutions(revolutions); }}
                min={0}
                max={100}
                step={1}
                inverted={false}
                unit={"%"}
                minLabel={"0"}
                maxLabel={"100"}
              />
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
