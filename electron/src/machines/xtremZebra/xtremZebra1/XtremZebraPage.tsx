import { Topbar } from "@/components/Topbar";
import { xtremZebraSerialRoute } from "@/routes/routes";
import React from "react";

export function XtremZebraPage() {
  const { serial } = xtremZebraSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/xtremZebra/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        // {
        //     link: "manual",
        //     activeLink: "manual",
        //     title: "Manual",
        //     icon: "lu:BookOpen",
        // },
        {
          link: "graphs",
          activeLink: "graphs",
          title: "Graphs",
          icon: "lu:ChartSpline",
        },
      ]}
    />
  );
}
