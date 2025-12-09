import { Topbar } from "@/components/Topbar";
import React from "react";

export function MetricsPage() {
  return (
    <Topbar
      pathname="/_sidebar/metrics"
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:Cog", 
        },
        {
          link: "graphs",
          activeLink: "graphs",
          title: "Graphs",
          icon: "lu:ChartColumnIncreasing", 
        },
      ]}
    />
  );
}