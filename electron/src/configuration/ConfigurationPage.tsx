import { Topbar } from "@/components/Topbar";
import { Server } from "lucide-react";
import React from "react";

export function ConfigurationPage() {
  return (
    <Topbar
      pathname="/_sidebar/configuration"
      items={[
        {
          link: "devices",
          title: "Geräte",
          icon: <Server size={20} />,
        },
      ]}
    />
  );
}
