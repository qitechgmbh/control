import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { useDre1 } from "./useDre";

import { DiameterVisualisation } from '../DiameterVisualisation';

export function Dre1ControlPage() {
    const {
        dreDiameter,
        dreState,
        dreSetTargetDiameter,
        dreSetLowerTolerance,
        dreSetHigherTolerance
    } = useDre1();

    // Controlled local states synced with dreState
    const targetDiameter = dreState?.data?.target_diameter ?? 0;
    const lowerTolerance = dreState?.data?.lower_tolerance ?? 0;
    const higherTolerance = dreState?.data?.higher_tolerance ?? 0;
    return (
        <Page>
            <ControlGrid columns={2}>
                <ControlCard title="Diameter Measurement">
                    <DiameterVisualisation
                        targetDiameter={targetDiameter}
                        lowTolerance={lowerTolerance}
                        highTolerance={higherTolerance}
                        dreDiameter={dreDiameter} />
                    <div className="flex flex-row items-center gap-6">
                        <TimeSeriesValueNumeric
                            label="Current Diameter"
                            unit="mm"
                            timeseries={dreDiameter}
                            renderValue={(value) => value.toFixed(3)}
                        />
                    </div>
                </ControlCard>
                <ControlCard title="DRE Settings">
                    <Label label="Set Target Diameter">
                        <EditValue
                            title="Set Target Diameter"
                            value={targetDiameter}
                            unit="mm"
                            step={0.01}
                            min={0}
                            max={5}
                            renderValue={(value) => value.toFixed(2)}
                            onChange={(val) => {
                                if (val < lowerTolerance) {
                                    dreSetLowerTolerance(val);
                                }
                                dreSetTargetDiameter(val);
                            }}
                            defaultValue={0}
                        />
                    </Label>
                    <Label label="Set Lower Tolerance">
                        <EditValue
                            title="Set Lower Tolerance"
                            value={lowerTolerance}
                            unit="mm"
                            step={0.01}
                            min={0}
                            max={Math.min(targetDiameter, 1)}
                            renderValue={(value) => value.toFixed(2)}
                            onChange={(val) => dreSetLowerTolerance(val)}
                            defaultValue={0}
                        />
                    </Label>
                    <Label label="Set Higher Tolerance">
                        <EditValue
                            title="Set Higher Tolerance"
                            value={higherTolerance}
                            unit="mm"
                            step={0.01}
                            min={0}
                            max={1}
                            renderValue={(value) => value.toFixed(2)}
                            onChange={(val) => dreSetHigherTolerance(val)}
                            defaultValue={0}
                        />
                    </Label>
                </ControlCard>
            </ControlGrid>
        </Page>
    );
}

