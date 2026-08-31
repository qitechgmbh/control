import { Topbar } from "@/components/Topbar";
import { wago750_531MachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago750_531MachinePage() {
  const { serial } = wago750_531MachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago750531machine/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
      ]}
    />
  );
}
