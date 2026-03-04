import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { useAquapath1 } from "./useAquapath";
import React from "react";
import { Button } from "@/components/ui/button";

export function Aquapath1SettingsPage() {
  const {
    state,
    setFrontRevolutions,
    setBackRevolutions,
    setFrontHeatingTolerance,
    setFrontCoolingTolerance,
    setBackHeatingTolerance,
    setBackCoolingTolerance,
    setAmbientTemperatureCalibration,
  } = useAquapath1();
  const frontTemp = state?.temperature_states.front.temperature;
  const backTemp = state?.temperature_states.back.temperature;
  const currentSensorAmbientCandidate =
    frontTemp != null && backTemp != null
      ? Math.min(frontTemp, backTemp)
      : undefined;
  const canApplySensorAmbient =
    currentSensorAmbientCandidate != null && currentSensorAmbientCandidate < 30;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Front Fan Revolutions">
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Max Revolution Speed"
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
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Max Revolution Speed"
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

        <ControlCard title="Ambient Calibration">
          <Label label="Set Ambient Temperature">
            <EditValue
              title="Set Ambient Temperature"
              min={10}
              value={state?.ambient_temperature_calibration ?? 22}
              max={40}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setAmbientTemperatureCalibration(val);
              }}
            />
          </Label>
          <div className="mt-3 flex items-center gap-3">
            <Button
              type="button"
              size="sm"
              disabled={!canApplySensorAmbient}
              onClick={() => {
                if (currentSensorAmbientCandidate == null) return;
                setAmbientTemperatureCalibration(currentSensorAmbientCandidate);
              }}
            >
              Use Current Sensor Temp
            </Button>
            <span className="text-muted-foreground text-sm">
              Candidate:{" "}
              {currentSensorAmbientCandidate != null
                ? `${currentSensorAmbientCandidate.toFixed(1)} C`
                : "N/A"}
            </span>
          </div>
        </ControlCard>

        <ControlCard title="Front Temperature Tolerances">
          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Heating Tolerance"
              min={0}
              value={state?.tolerance_states.front.heating}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Cooling Tolerance">
            <EditValue
              title="Set Cooling Tolerance"
              min={0}
              value={state?.tolerance_states.front.cooling}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontCoolingTolerance(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Back Temperature Tolerances">
          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Heating Tolerance"
              min={0}
              value={state?.tolerance_states.back.heating}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Cooling Tolerance">
            <EditValue
              title="Set Cooling Tolerance"
              min={0}
              value={state?.tolerance_states.back.cooling}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackCoolingTolerance(val);
              }}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
