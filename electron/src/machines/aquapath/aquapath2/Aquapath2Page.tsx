import { Topbar } from "@/components/Topbar";
import { aquapath2SerialRoute } from "@/routes/routes";
import React from "react";

export function Aquapath2Page() {
  const { serial } = aquapath2SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/aquapath2/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        {
          link: "graph",
          activeLink: "graph",
          title: "Graph",
          icon: "lu:ChartSpline",
        },
        {
          link: "settings",
          activeLink: "settings",
          title: "Config",
          icon: "lu:Settings",
        },
      ]}
    />
  );
}
