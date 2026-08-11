import { Topbar } from "@/components/Topbar";
import React from "react";

export function ExtruderWithMixerPage() {
  return (
    <Topbar
      pathname="/_sidebar/machines/extruder-with-mixer/0"
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
          title: "Config",
          icon: "lu:Settings",
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
