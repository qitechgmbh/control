import { Topbar } from "@/components/Topbar";
import { EthernetPort, Factory } from "lucide-react";
import React from "react";

export function SetupPage() {
  return (
    <Topbar
      pathname="/_sidebar/setup"
      items={[
        {
          link: "ethercat",
          title: "Ethercat",
          icon: <EthernetPort size={20} />,
        },
        {
          link: "machines",
          title: "Maschinen",
          icon: <Factory size={20} />,
        },
      ]}
    />
  );
}
