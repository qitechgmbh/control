import { Topbar } from "@/components/Topbar";
import { winder1SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder1Page() {
  const { serial } = winder1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/winder1/${serial}`}
      items={[
        {
          link: "control",
          title: "Steuerung",
          icon: "lu:CirclePlay",
        },
        {
          link: "graphs",
          title: "Graphen",
          icon: "lu:ChartSpline",
        },
        {
          link: "settings",
          title: "Einstellungen",
          icon: "lu:Settings",
        },
        {
          link: "manual",
          title: "Handbuch",
          icon: "lu:BookOpen",
        },
      ]}
    />
  );
}
