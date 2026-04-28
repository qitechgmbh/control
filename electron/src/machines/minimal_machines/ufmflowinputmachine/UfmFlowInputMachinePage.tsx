import { Topbar } from "@/components/Topbar";
import { ufmFlowInputMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function UfmFlowInputMachinePage() {
  const { serial } = ufmFlowInputMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/ufmflowinputmachine/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:Droplets",
        },
      ]}
    />
  );
}
