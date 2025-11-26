import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { useGluetex } from "./useGluetex";
import { roundToDecimals } from "@/lib/decimal";
import { SpoolAutomaticActionMode } from "./gluetexNamespace";
import { SelectionGroup } from "@/control/SelectionGroup";
import { cn } from "@/lib/utils";

export function GluetexOverviewPage() {
  const {
    state,
    pullerSpeed,
    temperature1,
    temperature2,
    temperature3,
    temperature4,
    temperature5,
    temperature6,
    spoolProgress,
    setSpoolAutomaticRequiredMeters,
    setSpoolAutomaticAction,
    resetSpoolProgress,
    isLoading,
    isDisabled,
  } = useGluetex();

  // Helper function to get temperature status color
  const getTemperatureColor = (temp?: number, target?: number) => {
    if (!temp || !target) return "bg-gray-300";
    const diff = Math.abs(temp - target);
    if (diff < 5) return "bg-green-500";
    if (diff < 15) return "bg-yellow-500";
    return "bg-red-500";
  };

  // Helper function to get stepper status color
  const getStepperColor = (mode?: string) => {
    if (!mode) return "bg-gray-300";
    if (mode === "Run") return "bg-green-500";
    return "bg-gray-300";
  };

  const handleResetProgress = () => {
    resetSpoolProgress();
  };

  return (
    <Page>
      <ControlGrid>
        {/* Top Row: Speed (left) */}
        <ControlCard title="Speed">
          <TimeSeriesValueNumeric
            label="Speed"
            unit="m/min"
            timeseries={pullerSpeed}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
        </ControlCard>

        {/* Top Row: Quali (center) */}
        <ControlCard title="Quali">
          <TimeSeriesValueNumeric
            label="Temperature 1"
            unit="C"
            timeseries={temperature1}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          <TimeSeriesValueNumeric
            label="Temperature 2"
            unit="C"
            timeseries={temperature2}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
        </ControlCard>

        {/* Right Column: Tall tile spans two rows */}
        <ControlCard height={2} width={1} title="Temperature & Motors">
          <div className="flex flex-col gap-4">
            {/* Temperature Status Grid */}
            <div>
              <h3 className="mb-2 text-lg font-semibold">Temperatures</h3>
              <div className="grid grid-cols-3 gap-2">
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Temp 1</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getTemperatureColor(
                        temperature1.current?.value,
                        state?.heating_states?.zone_1?.target_temperature,
                      ),
                    )}
                  />
                  <span className="text-xs">
                    {temperature1.current?.value
                      ? roundToDecimals(temperature1.current.value, 1)
                      : "—"}
                    °C
                  </span>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Temp 2</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getTemperatureColor(
                        temperature2.current?.value,
                        state?.heating_states?.zone_2?.target_temperature,
                      ),
                    )}
                  />
                  <span className="text-xs">
                    {temperature2.current?.value
                      ? roundToDecimals(temperature2.current.value, 1)
                      : "—"}
                    °C
                  </span>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Temp 3</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getTemperatureColor(
                        temperature3.current?.value,
                        state?.heating_states?.zone_3?.target_temperature,
                      ),
                    )}
                  />
                  <span className="text-xs">
                    {temperature3.current?.value
                      ? roundToDecimals(temperature3.current.value, 1)
                      : "—"}
                    °C
                  </span>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Temp 4</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getTemperatureColor(
                        temperature4.current?.value,
                        state?.heating_states?.zone_4?.target_temperature,
                      ),
                    )}
                  />
                  <span className="text-xs">
                    {temperature4.current?.value
                      ? roundToDecimals(temperature4.current.value, 1)
                      : "—"}
                    °C
                  </span>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Temp 5</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getTemperatureColor(
                        temperature5.current?.value,
                        state?.heating_states?.zone_5?.target_temperature,
                      ),
                    )}
                  />
                  <span className="text-xs">
                    {temperature5.current?.value
                      ? roundToDecimals(temperature5.current.value, 1)
                      : "—"}
                    °C
                  </span>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Temp 6</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getTemperatureColor(
                        temperature6.current?.value,
                        state?.heating_states?.zone_6?.target_temperature,
                      ),
                    )}
                  />
                  <span className="text-xs">
                    {temperature6.current?.value
                      ? roundToDecimals(temperature6.current.value, 1)
                      : "—"}
                    °C
                  </span>
                </div>
              </div>
            </div>

            {/* Addon Motors Status Grid */}
            <div>
              <h3 className="mb-2 text-lg font-semibold">Addon Motors</h3>
              <div className="grid grid-cols-2 gap-2">
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Stepper 3</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getStepperColor(state?.stepper_state?.stepper3_mode),
                    )}
                  />
                  <span className="text-xs">
                    {state?.stepper_state?.stepper3_mode || "Standby"}
                  </span>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border p-3">
                  <span className="text-sm font-medium">Stepper 4</span>
                  <div
                    className={cn(
                      "h-8 w-8 rounded-full",
                      getStepperColor(state?.stepper_state?.stepper4_mode),
                    )}
                  />
                  <span className="text-xs">
                    {state?.stepper_state?.stepper4_mode || "Standby"}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </ControlCard>

        {/* Second Row: Spool Autostop (left) */}
        <ControlCard title="Spool Autostop">
          <TimeSeriesValueNumeric
            label="Pulled Distance"
            renderValue={(value) => roundToDecimals(value, 2)}
            unit="m"
            timeseries={spoolProgress}
          />

          <Label label="Target Length">
            <EditValue
              value={state?.spool_automatic_action_state.spool_required_meters}
              unit="m"
              title="Expected Meters"
              defaultValue={250}
              min={10}
              max={10000}
              step={10}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setSpoolAutomaticRequiredMeters}
            />
          </Label>

          <TouchButton
            variant="outline"
            onClick={handleResetProgress}
            disabled={isDisabled}
            isLoading={isLoading || state?.traverse_state?.is_going_out}
          >
            Reset Progress
          </TouchButton>

          <Label label="After Target Length Reached">
            <SelectionGroup<SpoolAutomaticActionMode>
              value={
                state?.spool_automatic_action_state.spool_automatic_action_mode
              }
              disabled={isDisabled}
              loading={isLoading}
              onChange={setSpoolAutomaticAction}
              orientation="vertical"
              options={{
                Hold: {
                  children: "Hold",
                  icon: "lu:CirclePause",
                  className: "h-full",
                },
                Pull: {
                  children: "Pull",
                  icon: "lu:ChevronsLeft",
                  className: "h-full",
                },
                NoAction: {
                  children: "No Action",
                  icon: "lu:RefreshCcw",
                  className: "h-full",
                },
              }}
            />
          </Label>
        </ControlCard>

        {/* Second Row: AI-Info (center) */}
        <ControlCard title="KI Info">
          <div className="flex h-full items-center justify-center">
            <p className="text-center text-gray-500">
              AI Information will be displayed here
            </p>
          </div>
        </ControlCard>

        {/* Bottom Row: Order Information (left, width 2) */}
        <ControlCard width={2} title="Auftragsdaten">
          <div className="grid gap-4">
            <Label label="Auftragsnummer">
              <EditValue
                value={0}
                title="Auftragsnummer"
                defaultValue={0}
                min={0}
                max={999999}
                renderValue={(value) => value.toString()}
                onChange={() => {}}
              />
            </Label>
            <Label label="Artikelbezeichnung">
              <div className="rounded-lg border bg-gray-50 px-4 py-2">
                <span className="text-gray-500">Placeholder</span>
              </div>
            </Label>
            <Label label="Seriennummer">
              <EditValue
                value={0}
                title="Seriennummer"
                defaultValue={0}
                min={0}
                max={999999}
                renderValue={(value) => value.toString()}
                onChange={() => {}}
              />
            </Label>
          </div>
        </ControlCard>

        {/* Bottom Row: Status (right, width 1) */}
        <ControlCard width={1} title="Status">
          <div className="flex flex-col gap-3">
            <TouchButton
              variant="outline"
              disabled
              className="h-16 text-lg"
              icon="lu:Power"
            >
              STANDBY
            </TouchButton>
            <TouchButton
              variant="outline"
              disabled
              className="h-16 text-lg"
              icon="lu:Loader"
            >
              STARTING
            </TouchButton>
            <TouchButton
              variant="outline"
              disabled
              className="h-16 text-lg"
              icon="lu:Play"
            >
              RUN
            </TouchButton>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
