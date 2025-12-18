import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { useAquapath1 } from "./useAquapath";

export function Aquapath1SettingsPage() {
  const {
    state,
    setFrontRevolutions,
    setBackRevolutions,
  } = useAquapath1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Front Fan Revolutions">
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
        </ControlCard>

        <ControlCard title="Back Fan Revolutions">
          <Label label="Set Target Revolution Speed">
            <EditValue
              title="Set Target Revolution Speed"
              min={0}
              value={state?.fan_states.back.revolutions}
              max={100}
              unit="%"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackRevolutions(val);
              }}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
