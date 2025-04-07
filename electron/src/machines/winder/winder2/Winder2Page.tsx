import { Topbar } from "@/components/Topbar";
import { winder2SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder1Page() {
  const { serial } = winder2SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/winder2/${serial}`}
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
