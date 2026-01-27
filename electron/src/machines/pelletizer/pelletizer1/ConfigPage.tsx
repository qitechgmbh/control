import React from "react";

import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { MinMaxValue, TIMEFRAME_OPTIONS } from "@/control/MinMaxValue";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { usePellet1 as usePellet1 } from "./usePelletizer";

import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import { roundToDecimals } from "@/lib/decimal";
import { Badge } from "@/components/ui/badge";

export function Pellet1ConfigPage() {
    const {
        //state
        state,
        defaultState,
        
        //live values
        frequency,
        temperature,
        voltage,
        current,

        // mutation functions
        SetRunning,
        SetDirection,
        SetFrequencyTarget,
        SetAccelerationLevel,
        SetDecelerationLevel,
    } = usePellet1();

    // Extract values from consolidated state
    const running            = state?.inverter_state?.running ?? false;
    const direction          = state?.inverter_state?.direction ?? true;
    const frequency_target   = state?.inverter_state?.frequency_target ?? 0;
    const acceleration_level = state?.inverter_state?.acceleration_level ?? 0;
    const deceleration_level = state?.inverter_state?.deceleration_level ?? 0;

    // Shared timeframe state (default 5 minutes)
    const [timeframe, setTimeframe] = React.useState<number>(5 * 60 * 1000);

    return (
        <Page>
            <ControlGrid columns={2}>
                <ControlCard title="Advanced Inverter Configuration">
                    <SelectionGroupBoolean
                        value={running}
                        optionTrue={{ children: "On" }}
                        optionFalse={{ children: "Off" }}
                        onChange={(v) => {
                            SetRunning(v)
                        }}
                    />
                    
                    <Label label="Rotation Direction">
                        <SelectionGroupBoolean
                            value={direction}
                            optionTrue={{ children:  "Forward"  }}
                            optionFalse={{ children: "Backward" }}
                            onChange={(v) => {
                                SetDirection(v)
                            }}
                        />
                    </Label>

                    <Label label="Set Target Frequency">
                        <EditValue
                            title="Set Target Frequency"
                            value={frequency_target}
                            unit="Hz"
                            step={0.1}
                            min={0}
                            max={50}
                            renderValue={(value) => value.toFixed(1)}
                            onChange={(val) => {
                                SetFrequencyTarget(val);
                            }}
                            defaultValue={5}
                        />
                    </Label>
                    <Label label="Set Acceleration Level">
                        <EditValue
                            title="Set Acceleration Level"
                            value={acceleration_level}
                            unit={undefined}
                            step={1}
                            min={1}
                            max={15}
                            renderValue={(value) => value.toFixed(0)}
                            onChange={(val) => SetAccelerationLevel(val)}
                            defaultValue={7}
                        />
                    </Label>
                    <Label label="Set Deceleration Level">
                        <EditValue
                            title="Set Deceleration Level"
                            value={deceleration_level}
                            unit={undefined}
                            step={1}
                            min={1}
                            max={15}
                            renderValue={(value) => value.toFixed(0)}
                            onChange={(val) => SetDecelerationLevel(val)}
                            defaultValue={7}
                        />
                    </Label>
                </ControlCard>
            </ControlGrid>
        </Page>
    );
}