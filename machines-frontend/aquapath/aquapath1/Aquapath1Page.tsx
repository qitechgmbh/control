import { Topbar } from "@/components/Topbar";
import { useParams } from "@tanstack/react-router";
import React from "react";

export function Aquapath1Page() {
  const { serial } = useParams({ strict: false });
  return (
    <Topbar
      pathname={`/_sidebar/machines/aquapath1/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        {
          link: "graph",
          title: "Graph",
          icon: "lu:ChartSpline",
          activeLink: "graph",
        },
        {
          link: "settings",
          title: "Config",
          icon: "lu:Settings",
          activeLink: "settings",
        },
      ]}
    />
  );
}
