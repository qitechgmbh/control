// ============================================================================
// MyMachinePage.tsx — Top navigation bar for this machine
// ============================================================================
// This component renders the tab bar at the top of the machine page.
// Each `items` entry creates one tab that links to a sub-route.
//
// FIND & REPLACE to adapt this template:
//   MyMachine             → YourMachineName
//   myMachine             → yourMachineName
//   myMachineSerialRoute  → the route export from routes.tsx  (see step 8 in README)
//   mymachine             → the URL segment used in routes.tsx (e.g. "wagodotestmachine")
// ============================================================================

import { Topbar } from "@/components/Topbar";
import { myMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function MyMachinePage() {
  const { serial } = myMachineSerialRoute.useParams();

  return (
    <Topbar
      // The pathname must match the route defined in routes.tsx.
      pathname={`/_sidebar/machines/mymachine/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
        // TODO: add more tabs here if your machine needs multiple views, e.g.:
        // {
        //   link: "diagnostics",
        //   activeLink: "diagnostics",
        //   title: "Diagnostics",
        //   icon: "lu:Activity",
        // },
      ]}
    />
  );
}
