import { Topbar } from "@/components/layout/Topbar";
import { bottlecapsTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function BottlecapsTestMachinePage() {
  const { serial } = bottlecapsTestMachineSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/bottlecapstest/${serial}`}
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
