import { createRoute, Outlet } from "@tanstack/react-router";
import { RootRoute } from "./__root";
import React from "react";
import { z } from "zod";

import { ChooseVersionPage } from "@/setup/ChooseVersionPage";
import { fallback, zodValidator } from "@tanstack/zod-adapter";
import { githubSourceSchema } from "@/setup/GithubSourceDialog";
import { SidebarLayout } from "@/components/SidebarLayout";
import { SetupPage } from "@/setup/SetupPage";
import { EthercatPage } from "@/setup/EthercatPage";
import { MachinesPage } from "@/setup/MachinesPage";
import { ChangelogPage } from "@/setup/ChangelogPage";
import { TroubleshootPage } from "@/setup/Trobleshoot";
import { UpdateExecutePage } from "@/setup/UpdateExecutePage";

import { Winder2Page } from "@/machines/winder/winder2/Winder2Page";
import { Winder2ControlPage } from "@/machines/winder/winder2/Winder2ControlPage";
import { Winder2ManualPage } from "@/machines/winder/winder2/Winder2Manual";
import { Winder2SettingPage } from "@/machines/winder/winder2/Winder2Settings";
import { Winder2GraphsPage } from "@/machines/winder/winder2/Winder2Graphs";
import { Winder2PresetsPage } from "@/machines/winder/winder2/Winder2PresetsPage";

import { Extruder2Page } from "@/machines/extruder/extruder2/Extruder2Page";
import { Extruder2ControlPage } from "@/machines/extruder/extruder2/Extruder2ControlPage";
import { Extruder2SettingsPage } from "@/machines/extruder/extruder2/Extruder2Settings";
import { ExtruderV2ManualPage } from "@/machines/extruder/extruder2/Extruder2Manual";
import { Extruder2GraphsPage } from "@/machines/extruder/extruder2/Extruder2Graph";
import { Extruder2PresetsPage } from "@/machines/extruder/extruder2/Extruder2PresetsPage";

import { Extruder3Page } from "@/machines/extruder/extruder3/Extruder3Page";
import { Extruder3ControlPage } from "@/machines/extruder/extruder3/Extruder3ControlPage";
import { Extruder3SettingsPage } from "@/machines/extruder/extruder3/Extruder3Settings";
import { ExtruderV3ManualPage } from "@/machines/extruder/extruder3/Extruder3Manual";
import { Extruder3GraphsPage } from "@/machines/extruder/extruder3/Extruder3Graph";
import { Extruder3PresetsPage } from "@/machines/extruder/extruder3/Extruder3PresetsPage";

import { Buffer1ControlPage } from "@/machines/buffer/buffer1/Buffer1ControlPage";
import { Buffer1Page } from "@/machines/buffer/buffer1/Buffer1Page";
import { Buffer1SettingsPage } from "@/machines/buffer/buffer1/Buffer1Settings";

import { Laser1ControlPage } from "@/machines/laser/laser1/Laser1ControlPage";
import { Laser1GraphsPage } from "@/machines/laser/laser1/Laser1Graph";
import { Laser1Page } from "@/machines/laser/laser1/Laser1Page";
import { Laser1PresetsPage } from "@/machines/laser/laser1/Laser1PresetsPage";

import { Mock1ControlPage } from "@/machines/mock/mock1/Mock1ControlPage";
import { Mock1GraphPage } from "@/machines/mock/mock1/Mock1Graph";
import { Mock1ManualPage } from "@/machines/mock/mock1/Mock1Manual";
import { Mock1Page } from "@/machines/mock/mock1/Mock1Page";
import { Mock1PresetsPage } from "@/machines/mock/mock1/Mock1PresetsPage";

import { Aquapath1ControlPage } from "@/machines/aquapath/aquapath1/Aquapath1ControlPage";
import { Aquapath1Page } from "@/machines/aquapath/aquapath1/Aquapath1Page";
import { Aquapath1GraphPage } from "@/machines/aquapath/aquapath1/Aquapath1Graph";

import { TestMachinePage } from "@/machines/testmachine/TestMachinePage";
import { TestMachineControlPage } from "@/machines/testmachine/TestMachineControlPage";

import { AnalogInputTestMachine } from "@/machines/analoginputtestmachine/AnalogInputTestMachinePage";
import { AnalogInputTestMachineControl } from "@/machines/analoginputtestmachine/AnalogInputTestMachineControlPage";
import { IP20TestMachinePage } from "@/machines/ip20testmachine/IP20TestMachinePage";
import { IP20TestMachineControlPage } from "@/machines/ip20testmachine/IP20TestMachineControlPage";

import { MetricsGraphsPage } from "@/metrics/MetricsGraphsPage";
import { MetricsControlPage } from "@/metrics/MetricsControlPage";

import { WagoPower1Page } from "@/machines/wago_power/wago_power1/WagoPower1Page";
import { WagoPower1ControlPage } from "@/machines/wago_power/wago_power1/WagoPower1ControlPage";

// make a route tree like this
// _mainNavigation/machines/winder2/$serial/control
// _mainNavigation/configuration/a
// _mainNavigation/configuration/b
// the mainNavigation has a custom layout
// the winder2 winder2 and configuration also have a custom layout
// the leaf routes are just pages
export const testMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "testmachine/$serial",
  component: () => <TestMachinePage />,
});

// Leaf route: control page
export const testMachineControlRoute = createRoute({
  getParentRoute: () => testMachineSerialRoute,
  path: "control",
  component: () => <TestMachineControlPage />,
});

export const analogInputTestMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "analogInputTestMachine/$serial",
  component: () => <AnalogInputTestMachine />,
});

export const analogInputTestMachineControlRoute = createRoute({
  getParentRoute: () => analogInputTestMachineSerialRoute,
  path: "control",
  component: () => <AnalogInputTestMachineControl />,
});

export const ip20TestMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "ip20testmachine/$serial",
  component: () => <IP20TestMachinePage />,
});

export const ip20TestMachineControlRoute = createRoute({
  getParentRoute: () => ip20TestMachineSerialRoute,
  path: "control",
  component: () => <IP20TestMachineControlPage />,
});

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

export const extruder2PresetsRoute = createRoute({
  getParentRoute: () => extruder2Route,
  path: "presets",
  component: () => <Extruder2PresetsPage />,
});

export const extruder3Route = createRoute({
  getParentRoute: () => machinesRoute,
  path: "extruder3/$serial",
  component: () => <Extruder3Page />,
});

export const extruder3ControlRoute = createRoute({
  getParentRoute: () => extruder3Route,
  path: "control",
  component: () => <Extruder3ControlPage />,
});

export const extruder3SettingsRoute = createRoute({
  getParentRoute: () => extruder3Route,
  path: "settings",
  component: () => <Extruder3SettingsPage />,
});

export const extruder3ManualRoute = createRoute({
  getParentRoute: () => extruder3Route,
  path: "manual",
  component: () => <ExtruderV3ManualPage />,
});

export const extruder3GraphsRoute = createRoute({
  getParentRoute: () => extruder3Route,
  path: "graphs",
  component: () => <Extruder3GraphsPage />,
});

export const extruder3PresetsRoute = createRoute({
  getParentRoute: () => extruder3Route,
  path: "presets",
  component: () => <Extruder3PresetsPage />,
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

export const winder2PresetsRoute = createRoute({
  getParentRoute: () => winder2SerialRoute,
  path: "presets",
  component: () => <Winder2PresetsPage />,
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

export const laser1PresetsRoute = createRoute({
  getParentRoute: () => laser1SerialRoute,
  path: "presets",
  component: () => <Laser1PresetsPage />,
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

export const mock1PresetsRoute = createRoute({
  getParentRoute: () => mock1SerialRoute,
  path: "presets",
  component: () => <Mock1PresetsPage />,
});

export const aquapath1SerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "aquapath1/$serial",
  component: () => <Aquapath1Page />,
});

export const aquapath1GraphRoute = createRoute({
  getParentRoute: () => aquapath1SerialRoute,
  path: "graph",
  component: () => <Aquapath1GraphPage />,
});

export const aquapath1ControlRoute = createRoute({
  getParentRoute: () => aquapath1SerialRoute,
  path: "control",
  component: () => <Aquapath1ControlPage />,
});

export const buffer1SettingsRoute = createRoute({
  getParentRoute: () => buffer1SerialRoute,
  path: "settings",
  component: () => <Buffer1SettingsPage />,
});

export const wagoPower1SerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wago_power1/$serial",
  component: () => <WagoPower1Page />,
});

export const wagoPower1ControlRoute = createRoute({
  getParentRoute: () => wagoPower1SerialRoute,
  path: "control",
  component: () => <WagoPower1ControlPage />,
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
export const metricsRoute = createRoute({
  getParentRoute: () => setupRoute,
  path: "metrics",
  component: () => (
    <>
      <MetricsControlPage />
      <MetricsGraphsPage />
    </>
  ),
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
      metricsRoute,
    ]),
    machinesRoute.addChildren([
      laser1SerialRoute.addChildren([
        laser1ControlRoute,
        laser1GraphsRoute,
        laser1PresetsRoute,
      ]),
      testMachineSerialRoute.addChildren([testMachineControlRoute]),

      analogInputTestMachineSerialRoute.addChildren([
        analogInputTestMachineControlRoute,
      ]),

      ip20TestMachineSerialRoute.addChildren([ip20TestMachineControlRoute]),

      aquapath1SerialRoute.addChildren([
        aquapath1ControlRoute,
        aquapath1GraphRoute,
      ]),

      winder2SerialRoute.addChildren([
        winder2ControlRoute,
        winder2ManualRoute,
        winder2SettingsRoute,
        winder2GraphsRoute,
        winder2PresetsRoute,
      ]),

      extruder2Route.addChildren([
        extruder2ControlRoute,
        extruder2SettingsRoute,
        extruder2ManualRoute,
        extruder2GraphsRoute,
        extruder2PresetsRoute,
      ]),

      extruder3Route.addChildren([
        extruder3ControlRoute,
        extruder3SettingsRoute,
        extruder3ManualRoute,
        extruder3GraphsRoute,
        extruder3PresetsRoute,
      ]),

      mock1SerialRoute.addChildren([
        mock1ControlRoute,
        mock1GraphRoute,
        mock1ManualRoute,
        mock1PresetsRoute,
      ]),

      wagoPower1SerialRoute.addChildren([wagoPower1ControlRoute]),

      buffer1SerialRoute.addChildren([buffer1ControlRoute]),
    ]),
  ]),
]);
