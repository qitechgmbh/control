import { Topbar } from "@/components/Topbar";
import { extruder3Route } from "@/routes/routes";
import React from "react";

export function Extruder3Page() {
  const { serial } = extruder3Route.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/extruder3/${serial}`}
      items={[
        {
          link: "control",
          title: "Control",
          icon: "lu:CirclePlay",
          activeLink: "control",
        },
        {
          link: "graphs",
          title: "Graphs",
          icon: "lu:ChartSpline",
          activeLink: "graphs",
        },
        {
          link: "settings",
          title: "Config",
          icon: "lu:Settings",
          activeLink: "settings",
        },
        {
          link: "manual",
          title: "Manual",
          icon: "lu:BookOpen",
          activeLink: "manual",
        },
        {
          link: "presets",
          title: "Presets",
          icon: "lu:Save",
          activeLink: "presets",
        },
      ]}
    />
  );
}
