import { Topbar } from "@/components/Topbar";
import { testMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago8chDioTestMachinePage() {
  const { serial } = testMachineSerialRoute.useParams();
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
