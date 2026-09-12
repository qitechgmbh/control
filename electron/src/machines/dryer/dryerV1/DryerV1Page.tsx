import { Topbar } from "@/components/Topbar";
import { dryerV1SerialRoute } from "@/routes/routes";
import React from "react";

export function DryerV1Page() {
  const { serial } = dryerV1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/dryerV1/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        {
          link: "overview",
          activeLink: "overview",
          title: "Overview",
          icon: "lu:LayoutDashboard",
        },
        {
          link: "schedule",
          activeLink: "schedule",
          title: "Schedule",
          icon: "lu:CalendarClock",
        },
      ]}
    />
  );
}
