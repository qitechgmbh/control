import { Topbar } from "@/components/Topbar";
import { rewinderSerialRoute } from "@/routes/routes";
import React from "react";

export function RewinderPage() {
  const { serial } = rewinderSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/rewinder/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        {
          link: "settings",
          activeLink: "settings",
          title: "Settings",
          icon: "lu:Settings",
        },
        {
          link: "graphs",
          activeLink: "graphs",
          title: "Graphs",
          icon: "lu:ChartLine",
        },
      ]}
    />
  );
}
