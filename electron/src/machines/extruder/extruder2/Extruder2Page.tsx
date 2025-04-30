import { Topbar } from "@/components/Topbar";
import { extruder2Route } from "@/routes/routes";
import React from "react";

export function Extruder2Page() {
  let { serial } = extruder2Route.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/extruder2/${serial}`}
      items={[
        {
          link: "control",
          title: "Steuerung",
          icon: "lu:CirclePlay",
        },
        {
          link: "graphs",
          title: "Graphs",
          icon: "lu:ChartSpline",
        },
        {
          link: "settings",
          title: "Config",
          icon: "lu:Settings",
        },
        {
          link: "manual",
          title: "Manual",
          icon: "lu:BookOpen",
        },
      ]}
    />
  );
}
