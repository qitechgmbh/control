import { Topbar } from "@/components/Topbar";
import { wago8chDioTestMachineRoute } from "@/routes/routes";
import React from "react";

export function Wago8chDioTestMachinePage() {
  const { serial } = wago8chDioTestMachineRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago8chdiotestmachine/${serial}`}
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
