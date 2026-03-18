import { Topbar } from "@/components/Topbar";
import { serialRoute } from "../routes";
import React from "react";

export function Page() {
  const { serial } = serialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/ff01/${serial}`}
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
      ]}
    />
  );
}
