import { Topbar } from "@/components/Topbar";
import { bottleSorterSerialRoute } from "@/routes/routes";
import React from "react";

export function BottleSorterPage(): React.JSX.Element {
  const { serial } = bottleSorterSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/bottlesorter/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:ToggleLeft",
        },
      ]}
    />
  );
}
