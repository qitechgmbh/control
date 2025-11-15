import { Topbar } from "@/components/Topbar";
import React from "react";

export function GluetexPage() {
  return (
    <Topbar
      pathname={`/_sidebar/machines/gluetex`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        {
          link: "addons",
          activeLink: "addons",
          title: "Addons",
          icon: "lu:Puzzle",
        },
        {
          link: "graphs",
          activeLink: "graphs",
          title: "Graphs",
          icon: "lu:ChartSpline",
        },
        {
          link: "settings",
          activeLink: "settings",
          title: "Config",
          icon: "lu:Settings",
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
