import { Page } from "@/components/Page";
import { BigGraph, GraphConfig } from "@/helpers/BigGraph";
import React from "react";
import { useMock1 } from "./useMock";

export function Mock1GraphPage() {
  const { sineWave } = useMock1();

  const config: GraphConfig = {
    title: "Sine Wave",
    defaultTimeWindow: 1 * 60 * 1000, // 1 minute
    exportFilename: "sine_wave_data",
  };

  return (
    <Page>
      <BigGraph
        newData={sineWave}
        unit="mm"
        renderValue={(value) => value.toFixed(3)}
        config={config}
      />
    </Page>
  );
}
