import { Topbar } from "@/components/Topbar";
import { useParams } from "@tanstack/react-router";
import React from "react";

export function WagoPower1Page() {
  const { serial } = useParams({ strict: false });
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago_power1/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:PlugZap",
        },
      ]}
    />
  );
}
