import { Topbar } from "@ui/components/Topbar";
import { wagoPower1SerialRoute } from "@ui/routes/routes";
import React from "react";

export function WagoPower1Page() {
  const { serial } = wagoPower1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago_power1/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:PlugZap",
        },
      ]}
    />
  );
}
