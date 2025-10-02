import { Topbar } from "@/components/Topbar";
import { laser1SerialRoute } from "@/routes/routes";
import React from "react";
import { LaserToastManager } from "../LaserToastManager";

export function Laser1Page() {
  const { serial } = laser1SerialRoute.useParams();
  return (
    <>
    <LaserToastManager/>
    <Topbar
      pathname={`/_sidebar/machines/laser1/${serial}`}
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
        // {
        //     link: "manual",
        //     activeLink: "manual",
        //     title: "Manual",
        //     icon: "lu:BookOpen",
        // },
        {
          link: "presets",
          activeLink: "presets",
          title: "Presets",
          icon: "lu:Save",
        },
      ]}
    />
    </>
  );
}
