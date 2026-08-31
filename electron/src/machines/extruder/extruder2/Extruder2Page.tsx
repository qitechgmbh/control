import { Topbar } from "@/components/Topbar";
import { extruder2Route } from "@/routes/routes";
import React from "react";

export function Extruder2Page() {
  const { serial } = extruder2Route.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/extruder2/${serial}`}
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
