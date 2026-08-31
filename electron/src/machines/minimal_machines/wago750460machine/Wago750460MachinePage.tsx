import { Topbar } from "@/components/Topbar";
import { wago750460MachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago750460MachinePage() {
  const { serial } = wago750460MachineSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/wago750460machine/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:Thermometer",
        },
      ]}
    />
  );
}
