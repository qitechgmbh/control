import { Topbar } from "@/components/Topbar";
import { dryerSmartSerialRoute } from "@/routes/routes";
import React from "react";

export function DryerSmartPage() {
  const { serial } = dryerSmartSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/dryer_smart/${serial}`}
      items={[
        {
          link: "control",
          title: "Control",
          icon: "lu:CirclePlay",
          activeLink: "control",
        },
        {
          link: "overview",
          title: "Overview",
          icon: "lu:LayoutDashboard",
          activeLink: "overview",
        },
        {
          link: "schedule",
          title: "Schedule",
          icon: "lu:CalendarClock",
          activeLink: "schedule",
        },
        {
          link: "material",
          title: "Material",
          icon: "lu:FlaskConical",
          activeLink: "material",
        },
      ]}
    />
  );
}
