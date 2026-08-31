import { Topbar } from "@/components/Topbar";
import { wago750_553MachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago750_553MachinePage() {
  const { serial } = wago750_553MachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago750553machine/${serial}`}
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
