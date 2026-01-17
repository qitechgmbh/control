import { Topbar } from "@/components/Topbar";
import { laser1SerialRoute } from "@/routes/routes";
import React from "react";

export function Laser1Page() {
  const { serial } = laser1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/laser1/${serial}`}
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
        // {
        //   link: "settings",
        //   activeLink: "settings",
        //   title: "Config",
        //   icon: "lu:Settings",
        // },
        // {
        //     link: "manual",
        //     activeLink: "manual",
        //     title: "Manual",
        //     icon: "lu:BookOpen",
        // },
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
