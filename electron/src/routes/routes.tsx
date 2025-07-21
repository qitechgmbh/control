import { createRoute, Outlet } from "@tanstack/react-router";
import { RootRoute } from "./__root";
import React from "react";
import { SidebarLayout } from "@/components/SidebarLayout";
import { SetupPage } from "@/setup/SetupPage";
import { EthercatPage } from "@/setup/EthercatPage";
import { MachinesPage } from "@/setup/MachinesPage";
import { Winder2Page } from "@/machines/winder/winder2/Winder2Page";
import { Winder2ControlPage } from "@/machines/winder/winder2/Winder2ControlPage";
import { Winder2ManualPage } from "@/machines/winder/winder2/Winder2Manual";
import { Winder2SettingPage } from "@/machines/winder/winder2/Winder2Settings";
import { Winder2GraphsPage } from "@/machines/winder/winder2/Winder2Graphs";
import { Extruder2Page } from "@/machines/extruder/extruder2/Extruder2Page";
import { Extruder2ControlPage } from "@/machines/extruder/extruder2/Extruder2ControlPage";
import { Extruder2SettingsPage } from "@/machines/extruder/extruder2/Extruder2Settings";
import { ExtruderV2ManualPage } from "@/machines/extruder/extruder2/Extruder2Manual";
import { ChooseVersionPage } from "@/setup/ChooseVersionPage";
import { fallback, zodValidator } from "@tanstack/zod-adapter";

import { githubSourceSchema } from "@/setup/GithubSourceDialog";
import { z } from "zod";
import { ChangelogPage } from "@/setup/ChangelogPage";
import { UpdateExecutePage } from "@/setup/UpdateExecutePage";
import { Laser1Page } from "@/machines/laser/laser1/Laser1Page";
import { Laser1ControlPage } from "@/machines/laser/laser1/Laser1ControlPage";
import { Mock1Page } from "@/machines/mock/mock1/Mock1Page";
import { Mock1ControlPage } from "@/machines/mock/mock1/Mock1ControlPage";
import { Laser1GraphsPage } from "@/machines/laser/laser1/Laser1Graph";
import { Mock1GraphPage } from "@/machines/mock/mock1/Mock1Graph";
import { Mock1ManualPage } from "@/machines/mock/mock1/Mock1Manual";
import { TroubleshootPage } from "@/setup/Trobleshoot";
import { Extruder2GraphsPage } from "@/machines/extruder/extruder2/Extruder2Graph";
import { Buffer1Page } from "@/machines/buffer/buffer1/Buffer1Page";
import { Buffer1ControlPage } from "@/machines/buffer/buffer1/Buffer1ControlPage";

// make a route tree like this
// _mainNavigation/machines/winder2/$serial/control
// _mainNavigation/configuration/a
// _mainNavigation/configuration/b
// the mainNavigation has a custom layout
// the winder2 winder2 and configuration also have a custom layout
// the leaf routes are just pages

export const sidebarRoute = createRoute({
  getParentRoute: () => RootRoute,
  path: "_sidebar",
  component: () => <SidebarLayout />,
});

export const machinesRoute = createRoute({
  getParentRoute: () => sidebarRoute,
  path: "machines",
});

export const extruder2Route = createRoute({
  getParentRoute: () => machinesRoute,
  path: "extruder2/$serial",
  component: () => <Extruder2Page />,
});

export const extruder2ControlRoute = createRoute({
  getParentRoute: () => extruder2Route,
  path: "control",
  component: () => <Extruder2ControlPage />,
});

export const extruder2SettingsRoute = createRoute({
  getParentRoute: () => extruder2Route,
  path: "settings",
  component: () => <Extruder2SettingsPage />,
});

export const extruder2ManualRoute = createRoute({
  getParentRoute: () => extruder2Route,
  path: "manual",
  component: () => <ExtruderV2ManualPage />,
});

export const extruder2GraphsRoute = createRoute({
  getParentRoute: () => extruder2Route,
  path: "graphs",
  component: () => <Extruder2GraphsPage />,
});

export const winder2SerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "winder2/$serial",
  component: () => <Winder2Page />,
});

export const winder2ControlRoute = createRoute({
  getParentRoute: () => winder2SerialRoute,
  path: "control",
  component: () => <Winder2ControlPage />,
});

export const winder2ManualRoute = createRoute({
  getParentRoute: () => winder2SerialRoute,
  path: "manual",
  component: () => <Winder2ManualPage />,
});

export const winder2SettingsRoute = createRoute({
  getParentRoute: () => winder2SerialRoute,
  path: "settings",
  component: () => <Winder2SettingPage />,
});

export const winder2GraphsRoute = createRoute({
  getParentRoute: () => winder2SerialRoute,
  path: "graphs",
  component: () => <Winder2GraphsPage />,
});

export const laser1SerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "laser1/$serial",
  component: () => <Laser1Page />,
});

export const laser1ControlRoute = createRoute({
  getParentRoute: () => laser1SerialRoute,
  path: "control",
  component: () => <Laser1ControlPage />,
});

export const laser1GraphsRoute = createRoute({
  getParentRoute: () => laser1SerialRoute,
  path: "graphs",
  component: () => <Laser1GraphsPage />,
});

export const mock1SerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "mock1/$serial",
  component: () => <Mock1Page />,
});

export const mock1ControlRoute = createRoute({
  getParentRoute: () => mock1SerialRoute,
  path: "control",
  component: () => <Mock1ControlPage />,
});

export const mock1GraphRoute = createRoute({
  getParentRoute: () => mock1SerialRoute,
  path: "graph",
  component: () => <Mock1GraphPage />,
});

export const mock1ManualRoute = createRoute({
  getParentRoute: () => mock1SerialRoute,
  path: "manual",
  component: () => <Mock1ManualPage />,
});

export const buffer1SerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "buffer1/$serial",
  component: () => <Buffer1Page />,
});

export const buffer1ControlRoute = createRoute({
  getParentRoute: () => buffer1SerialRoute,
  path: "control",
  component: () => <Buffer1ControlPage />,
});

export const setupRoute = createRoute({
  getParentRoute: () => sidebarRoute,
  path: "setup",
  component: () => <SetupPage />,
});

export const troubleshootRoute = createRoute({
  getParentRoute: () => setupRoute,
  path: "troubleshoot",
  component: () => <TroubleshootPage />, // Placeholder for future troubleshooting page
});

export const ethercatRoute = createRoute({
  getParentRoute: () => setupRoute,
  path: "ethercat",
  component: () => <EthercatPage />,
});

export const setupMachinesRoute = createRoute({
  getParentRoute: () => setupRoute,
  path: "machines",
  component: () => <MachinesPage />,
});

export const updateRoute = createRoute({
  getParentRoute: () => setupRoute,
  path: "update",
  component: () => <Outlet />,
});

export const updateChooseVersionRoute = createRoute({
  getParentRoute: () => updateRoute,
  path: "choose-version",
  component: () => <ChooseVersionPage />,
});

export const versionSearchSchema = z
  .object({
    branch: fallback(z.string().optional(), undefined),
    commit: fallback(z.string().optional(), undefined),
    tag: fallback(z.string().optional(), undefined),
  })
  .merge(githubSourceSchema)
  .refine(
    (data) => {
      const definedCount = [data.branch, data.commit, data.tag].filter(
        Boolean,
      ).length;
      return definedCount === 1;
    },
    {
      message: "Exactly one of branch, commit, or tag must be defined",
      path: ["error"],
    },
  );

export type VersionSearch = z.infer<typeof versionSearchSchema>;

export const updateChangelogRoute = createRoute({
  getParentRoute: () => updateRoute,
  path: "changelog",
  component: () => <ChangelogPage />,
  validateSearch: zodValidator(versionSearchSchema),
});

export const updateExecuteRoute = createRoute({
  getParentRoute: () => updateRoute,
  path: "execute",
  component: () => <UpdateExecutePage />,
  validateSearch: zodValidator(versionSearchSchema),
});

export const rootTree = RootRoute.addChildren([
  sidebarRoute.addChildren([
    setupRoute.addChildren([
      ethercatRoute,
      setupMachinesRoute,
      updateRoute.addChildren([
        updateChooseVersionRoute,
        updateChangelogRoute,
        updateExecuteRoute,
      ]),
      troubleshootRoute,
    ]),

    machinesRoute.addChildren([
      laser1SerialRoute.addChildren([laser1ControlRoute, laser1GraphsRoute]),

      winder2SerialRoute.addChildren([
        winder2ControlRoute,
        winder2ManualRoute,
        winder2SettingsRoute,
        winder2GraphsRoute,
      ]),

      extruder2Route.addChildren([
        extruder2ControlRoute,
        extruder2SettingsRoute,
        extruder2ManualRoute,
        extruder2GraphsRoute,
      ]),

      mock1SerialRoute.addChildren([
        mock1ControlRoute,
        mock1GraphRoute,
        mock1ManualRoute,
      ]),

      buffer1SerialRoute.addChildren([buffer1ControlRoute]),
    ]),
  ]),
]);
