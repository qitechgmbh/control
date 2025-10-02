import { Topbar } from "@/components/Topbar";
import { aquapath1SerialRoute } from "@/routes/routes";
import React from "react";

export function Aquapath1Page() {
  const { serial } = aquapath1SerialRoute.useParams();
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
          activeLink: "graphs",
        },
      ]}
    />
  );
}
