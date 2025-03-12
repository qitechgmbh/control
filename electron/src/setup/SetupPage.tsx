import { Topbar } from "@/components/Topbar";
import React from "react";

export function SetupPage() {
  return (
    <Topbar
      pathname="/_sidebar/setup"
      items={[
        {
          link: "ethercat",
          title: "Ethercat",
          icon: "lu:EthernetPort",
        },
        {
          link: "machines",
          title: "Maschinen",
          icon: "lu:Factory",
        },
      ]}
    />
  );
}
