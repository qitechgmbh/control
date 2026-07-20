import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { dryerV1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerV1SerialRoute } from "@/routes/routes";
import { useDryerV1Namespace } from "./dryerV1Namespace";
import React, { useMemo } from "react";

export function DryerV1OverviewPage() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const {
    liveValues,
    ts_temp_process,
    ts_temp_regen_in,
    ts_temp_regen_out,
    ts_temp_fan_inlet,
    ts_temp_safety,
    ts_temp_return_air,
    ts_power_process,
    ts_power_regen,
    ts_pwm_fan1,
    ts_pwm_fan2,
  } = useDryerV1Namespace(machineIdentification);
  const v = liveValues?.data;

  return (
    <Page>
      <ControlGrid columns={3}>
        <ControlCard title="Process">
          <TimeSeriesValueNumeric
            label="Process Temperature"
            unit="C"
            timeseries={ts_temp_process}
            renderValue={(val) => val.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Safety Temperature"
            unit="C"
            timeseries={ts_temp_safety}
            renderValue={(val) => val.toFixed(1)}
          />
        </ControlCard>

        <ControlCard title="Regeneration">
          <TimeSeriesValueNumeric
            label="Regen Inlet Temperature"
            unit="C"
            timeseries={ts_temp_regen_in}
            renderValue={(val) => val.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Regen Outlet Temperature"
            unit="C"
            timeseries={ts_temp_regen_out}
            renderValue={(val) => val.toFixed(1)}
          />
        </ControlCard>

        <ControlCard title="Air Flow">
          <TimeSeriesValueNumeric
            label="Fan Inlet Temperature"
            unit="C"
            timeseries={ts_temp_fan_inlet}
            renderValue={(val) => val.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Return Air Temperature"
            unit="C"
            timeseries={ts_temp_return_air}
            renderValue={(val) => val.toFixed(1)}
          />
          <div className="text-sm text-gray-400">
            Dew Point:{" "}
            <span className="font-mono font-semibold text-gray-700">
              {v ? `${v.temp_dew_point.toFixed(1)} C` : "—"}
            </span>
          </div>
        </ControlCard>

        <ControlCard title="Fans">
          <TimeSeriesValueNumeric
            label="Fan 1 PWM"
            unit="%"
            timeseries={ts_pwm_fan1}
            renderValue={(val) => val.toFixed(0)}
          />
          <TimeSeriesValueNumeric
            label="Fan 2 PWM"
            unit="%"
            timeseries={ts_pwm_fan2}
            renderValue={(val) => val.toFixed(0)}
          />
        </ControlCard>

        <ControlCard title="Power">
          <TimeSeriesValueNumeric
            label="Process Power"
            unit="W"
            timeseries={ts_power_process}
            renderValue={(val) => val.toFixed(0)}
          />
          <TimeSeriesValueNumeric
            label="Regen Power"
            unit="W"
            timeseries={ts_power_regen}
            renderValue={(val) => val.toFixed(0)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
