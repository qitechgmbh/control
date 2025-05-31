import { Page } from "@/components/Page";
import { BigGraph } from "@/helpers/BigGraph";
import React from "react";
import { useDre1 } from "./useDre";
import { ControlCard } from "@/control/ControlCard";

export function Dre1GraphsPage() {
  const {
    dreDiameter,
    dreState,
  } = useDre1();

  // Controlled local states synced with dreState
  const targetDiameter = dreState?.data?.target_diameter ?? 0;
  const lowerTolerance = dreState?.data?.lower_tolerance ?? 0;
  const higherTolerance = dreState?.data?.higher_tolerance ?? 0;

  return <Page>
    <ControlCard title="Diameter Graph">
      <div style={{
        width: '100%',
        height: '80vh', // 80% of viewport height
        minHeight: '400px', // Minimum height to ensure usability
        maxHeight: '800px' // Maximum height to prevent it from being too large
      }}>
        <BigGraph
          newData={dreDiameter}
          threshold1={targetDiameter + higherTolerance}
          threshold2={targetDiameter - lowerTolerance}
          target={targetDiameter}
          unit="mm"
          renderValue={(value) => value.toFixed(3)}
        />
      </div>
    </ControlCard>
  </Page>;
}
