import { Topbar } from "@/components/Topbar";
import { winder2SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder2Page() {
  const { serial } = winder2SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/winder2/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        {
          link: "graphs",
          activeLink: "graphs",
          title: "Graphs",
          icon: "lu:ChartSpline",
        },
        {
          link: "settings",
          activeLink: "settings",
          title: "Config",
          icon: "lu:Settings",
        },
        {
          link: "manual",
          activeLink: "manual",
          title: "Manual",
          icon: "lu:BookOpen",
        },
        {
          link: "presets",
          activeLink: "presets",
          title: "Presets",
          icon: "lu:Save",
        },
      ]}
    />
  );
}
