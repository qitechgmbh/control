import React from "react";
import { Page } from "@/components/Page";
import { ExtruderSimulationGraph } from "./ExtruderSimulationGraph";
import { ExtruderSimulationSettings } from "./ExtruderSimulationSettings";

export function ExtruderSimulationPage() {
  return (
    <Page className="pb-25">
      <h1 className="text-3xl font-bold">Extruder Heating Simulation</h1>
      <div className="grid grid-cols-1 gap-6 xl:grid-cols-[24rem_1fr]">
        <ExtruderSimulationSettings />
        <ExtruderSimulationGraph />
      </div>
    </Page>
  );
}
