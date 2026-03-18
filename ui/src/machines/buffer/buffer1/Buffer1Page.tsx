import { Topbar } from "@ui/components/Topbar";
import { buffer1SerialRoute } from "@ui/routes/routes";
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
