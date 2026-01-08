import { Topbar } from "@/components/Topbar";
import { bottleSorterSerialRoute } from "@/routes/routes";
import React from "react";

export function MinimalBottleSorterPage(): React.JSX.Element {
  const { serial } = bottleSorterSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/minimalbottlesorter/${serial}`}
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
