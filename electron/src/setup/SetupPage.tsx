import { Topbar } from "@/components/Topbar";
import React from "react";

export function SetupPage() {
  return (
    <Topbar
      pathname="/_sidebar/setup"
      items={[
        {
          link: "ethercat",
          title: "EtherCat",
          icon: "lu:EthernetPort",
        },
        {
          link: "machines",
          title: "Machines",
          icon: "lu:Factory",
        },
        {
          link: "update",
          title: "Update",
          icon: "lu:CircleFadingArrowUp",
        },
      ]}
    />
  );
}
