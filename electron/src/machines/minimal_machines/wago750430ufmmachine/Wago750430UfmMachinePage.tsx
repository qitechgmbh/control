import { Topbar } from "@/components/Topbar";
import { wago750430UfmMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago750430UfmMachinePage() {
  const { serial } = wago750430UfmMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago750430ufmmachine/${serial}`}
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
