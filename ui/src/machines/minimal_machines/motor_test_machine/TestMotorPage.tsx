import { Topbar } from "@ui/components/Topbar";
import { testMotorSerialRoute } from "@ui/routes/routes";
import React from "react";

export function TestMotorPage() {
  // Access is allowed here because this function is called later
  const { serial } = testMotorSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/testmotor/${serial}`}
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
