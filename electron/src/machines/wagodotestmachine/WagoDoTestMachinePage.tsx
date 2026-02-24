import { Topbar } from "@/components/Topbar";
import { wagoDoTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function WagoDoTestMachinePage() {
  const { serial } = wagoDoTestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wagodotestmachine/${serial}`}
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
