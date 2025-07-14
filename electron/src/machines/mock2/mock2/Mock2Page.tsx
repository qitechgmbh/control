import { Topbar } from "@/components/Topbar";
import { mock2SerialRoute } from "@/routes/routes";
import React from "react";

export function Mock2Page() {
  const { serial } = mock2SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/mock2/${serial}`}
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
          icon: "lu:CirclePlay",
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
