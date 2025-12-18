import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
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
        <ControlCard title="Front Fan Revolutions">
          <div className="grid grid-rows-2 gap-8">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolution Speed"
                unit="%"
                timeseries={front_revolutions}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <Label label="Set Target Revolution Speed">
                <EditValue
                  title="Set Target Revolution Speed"
                  min={0}
                  value={state?.fan_states.front.revolutions}
                  max={100}
                  unit="%"
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setFrontRevolutions(val);
                  }}
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Back Fan Revolutions">
          <div className="grid grid-rows-2 gap-4">
            <div className="flex flex-row">
              <TimeSeriesValueNumeric
                label="Revolution Speed"
                unit="%"
                timeseries={back_revolutions}
                renderValue={(value) => value.toFixed(1)}
              />
            </div>

            <div className="flex flex-row">
              <Label label="Set Target Revolution Speed">
                <EditValue
                  title="Set Target Revolution Speed"
                  min={0}
                  value={state?.fan_states.front.revolutions}
                  max={100}
                  unit="%"
                  renderValue={(value) => value.toFixed(1)}
                  onChange={(val) => {
                    setBackRevolutions(val);
                  }}
                />
              </Label>
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
