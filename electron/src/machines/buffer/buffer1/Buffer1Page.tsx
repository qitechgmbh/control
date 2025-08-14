import { Topbar } from "@/components/Topbar";
import { buffer1SerialRoute } from "@/routes/routes";
import React from "react";

export function Buffer1Page() {
  const { serial } = buffer1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/buffer1/${serial}`}
      items={[
        {
          link: "control",
          title: "Control",
          icon: "lu:CirclePlay",
          activeLink: "control",
        },
        {
          link: "settings",
          activeLink: "settings",
          title: "Config",
          icon: "lu:Settings",
        },
      ]}
    />
  );
}
