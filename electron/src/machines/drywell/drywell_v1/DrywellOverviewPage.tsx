import React, { useMemo } from "react";
import { drywellV1SerialRoute } from "@/routes/routes";
import { drywellV1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { useDrywellNamespace } from "./drywellNamespace";
import { Page } from "@/components/Page";

const drywellImage = "/images/drywell/drywell_machine.png";

type LabelSide = "left" | "right";

interface LabelProps {
  id: string;
  value: string;
  x: number;
  y: number;
  side: LabelSide;
}

function SensorLabel({ id, value, x, y, side }: LabelProps) {
  return (
    <div
      className="absolute flex items-center"
      style={{
        left: `${x}%`,
        top: `${y}%`,
        transform: "translateY(-50%)",
        flexDirection: side === "left" ? "row-reverse" : "row",
      }}
    >
      <div className="h-px w-10 shrink-0 bg-green-400" />
      <div className="rounded border border-green-400 bg-black/70 px-1.5 py-0.5 font-mono text-xs leading-tight whitespace-nowrap text-green-300">
        <span className="mr-1 font-bold">{id}</span>
        {value}
      </div>
    </div>
  );
}

function fmt1(v: number | undefined) {
  return v !== undefined ? `${v.toFixed(1)}°C` : "—";
}

function fmtPct(v: number | undefined) {
  return v !== undefined ? `${v.toFixed(0)}%` : "—";
}

export function DrywellOverviewPage() {
  const { serial: serialString } = drywellV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: drywellV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const { liveValues } = useDrywellNamespace(machineIdentification);
  const v = liveValues?.data;

  return (
    <Page>
      <div className="mx-auto w-full max-w-4xl select-none">
        <div className="relative w-full" style={{ paddingBottom: "72%" }}>
          <img
            src={drywellImage}
            alt="Drywell machine"
            className="absolute inset-0 h-full w-full object-contain"
            draggable={false}
          />

          <SensorLabel
            id="T4"
            value={fmt1(v?.temp_process)}
            x={36}
            y={28}
            side="left"
          />
          <SensorLabel
            id="T5"
            value={fmt1(v?.temp_safety)}
            x={36}
            y={32}
            side="left"
          />
          <SensorLabel
            id="S2"
            value={fmtPct(v?.pwm_fan2)}
            x={28}
            y={50}
            side="left"
          />
          <SensorLabel
            id="T3"
            value={fmt1(v?.temp_fan_inlet)}
            x={27}
            y={62}
            side="left"
          />
          <SensorLabel
            id="S1"
            value={fmtPct(v?.pwm_fan1)}
            x={30}
            y={79}
            side="left"
          />

          <SensorLabel
            id="T6"
            value={fmt1(v?.temp_return_air)}
            x={66}
            y={24}
            side="right"
          />
          <SensorLabel
            id="T1"
            value={fmt1(v?.temp_regen_in)}
            x={56}
            y={47}
            side="right"
          />
          <SensorLabel
            id="R1"
            value={fmtPct(v?.power_process)}
            x={63}
            y={52}
            side="right"
          />
          <SensorLabel
            id="R2"
            value={fmtPct(v?.power_regen)}
            x={66}
            y={60}
            side="right"
          />
          <SensorLabel
            id="T2"
            value={fmt1(v?.temp_regen_out)}
            x={55}
            y={66}
            side="right"
          />
        </div>
      </div>
    </Page>
  );
}
