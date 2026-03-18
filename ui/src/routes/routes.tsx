import { createRoute, Outlet } from "@tanstack/react-router";
import { RootRoute } from "./__root";
import React from "react";
import { z } from "zod";

import { ChooseVersionPage } from "@ui/setup/ChooseVersionPage";
import { githubSourceSchema } from "@ui/setup/GithubSourceDialog";
import { SidebarLayout } from "@ui/components/SidebarLayout";
import { SetupPage } from "@ui/setup/SetupPage";
import { EthercatPage } from "@ui/setup/EthercatPage";
import { MachinesPage } from "@ui/setup/MachinesPage";
import { ChangelogPage } from "@ui/setup/ChangelogPage";
import { TroubleshootPage } from "@ui/setup/Trobleshoot";
import { UpdateExecutePage } from "@ui/setup/UpdateExecutePage";

import { Winder2Page } from "@ui/machines/winder/winder2/Winder2Page";
import { Winder2ControlPage } from "@ui/machines/winder/winder2/Winder2ControlPage";
import { Winder2ManualPage } from "@ui/machines/winder/winder2/Winder2Manual";
import { Winder2SettingPage } from "@ui/machines/winder/winder2/Winder2Settings";
import { Winder2GraphsPage } from "@ui/machines/winder/winder2/Winder2Graphs";
import { Winder2PresetsPage } from "@ui/machines/winder/winder2/Winder2PresetsPage";

import { Extruder2Page } from "@ui/machines/extruder/extruder2/Extruder2Page";
import { Extruder2ControlPage } from "@ui/machines/extruder/extruder2/Extruder2ControlPage";
import { Extruder2SettingsPage } from "@ui/machines/extruder/extruder2/Extruder2Settings";
import { ExtruderV2ManualPage } from "@ui/machines/extruder/extruder2/Extruder2Manual";
import { Extruder2GraphsPage } from "@ui/machines/extruder/extruder2/Extruder2Graph";
import { Extruder2PresetsPage } from "@ui/machines/extruder/extruder2/Extruder2PresetsPage";

import { Extruder3Page } from "@ui/machines/extruder/extruder3/Extruder3Page";
import { Extruder3ControlPage } from "@ui/machines/extruder/extruder3/Extruder3ControlPage";
import { Extruder3SettingsPage } from "@ui/machines/extruder/extruder3/Extruder3Settings";
import { ExtruderV3ManualPage } from "@ui/machines/extruder/extruder3/Extruder3Manual";
import { Extruder3GraphsPage } from "@ui/machines/extruder/extruder3/Extruder3Graph";
import { Extruder3PresetsPage } from "@ui/machines/extruder/extruder3/Extruder3PresetsPage";

import { Buffer1ControlPage } from "@ui/machines/buffer/buffer1/Buffer1ControlPage";
import { Buffer1Page } from "@ui/machines/buffer/buffer1/Buffer1Page";
import { Buffer1SettingsPage } from "@ui/machines/buffer/buffer1/Buffer1Settings";

import { Laser1ControlPage } from "@ui/machines/laser/laser1/Laser1ControlPage";
import { Laser1GraphsPage } from "@ui/machines/laser/laser1/Laser1Graph";
import { Laser1Page } from "@ui/machines/laser/laser1/Laser1Page";
import { Laser1PresetsPage } from "@ui/machines/laser/laser1/Laser1PresetsPage";
import { Laser1SettingsPage } from "@ui/machines/laser/laser1/Laser1SettingsPage";

import { WagoSerialPage } from "@ui/machines/wago_serial/WagoSerialPage";
import { WagoSerialControlPage } from "@ui/machines/wago_serial/WagoSerialControlPage";

import { Mock1ControlPage } from "@ui/machines/minimal_machines/mock/mock1/Mock1ControlPage";
import { Mock1GraphPage } from "@ui/machines/minimal_machines/mock/mock1/Mock1Graph";
import { Mock1ManualPage } from "@ui/machines/minimal_machines/mock/mock1/Mock1Manual";
import { Mock1Page } from "@ui/machines/minimal_machines/mock/mock1/Mock1Page";
import { Mock1PresetsPage } from "@ui/machines/minimal_machines/mock/mock1/Mock1PresetsPage";

import { Aquapath1ControlPage } from "@ui/machines/aquapath/aquapath1/Aquapath1ControlPage";
import { Aquapath1Page } from "@ui/machines/aquapath/aquapath1/Aquapath1Page";
import { Aquapath1GraphPage } from "@ui/machines/aquapath/aquapath1/Aquapath1Graph";
import { Aquapath1SettingsPage } from "@ui/machines/aquapath/aquapath1/Aquapath1Settings";

import { TestMachinePage } from "@ui/machines/minimal_machines/testmachine/TestMachinePage";
import { TestMachineControlPage } from "@ui/machines/minimal_machines/testmachine/TestMachineControlPage";

import { TestMachineStepperPage } from "@ui/machines/minimal_machines/testmachinestepper/TestMachineStepperPage";
import { TestMachineStepperControlPage } from "@ui/machines/minimal_machines/testmachinestepper/TestMachineStepperControlPage";
import { AnalogInputTestMachine } from "@ui/machines/minimal_machines/analoginputtestmachine/AnalogInputTestMachinePage";
import { AnalogInputTestMachineControl } from "@ui/machines/minimal_machines/analoginputtestmachine/AnalogInputTestMachineControlPage";
import { WagoAiTestMachine } from "@ui/machines/minimal_machines/wagoaitestmachine/WagoAiTestMachinePage";
import { WagoAiTestMachineControl } from "@ui/machines/minimal_machines/wagoaitestmachine/WagoAiTestMachineControlPage";

import { DigitalInputTestMachinePage } from "@ui/machines/minimal_machines/digitalinputtestmachine/DigitalInputTestMachinePage";
import { DigitalInputTestMachineControlPage } from "@ui/machines/minimal_machines/digitalinputtestmachine/DigitalInputTestMachineControlPage";

import { IP20TestMachinePage } from "@ui/machines/minimal_machines/ip20testmachine/IP20TestMachinePage";
import { IP20TestMachineControlPage } from "@ui/machines/minimal_machines/ip20testmachine/IP20TestMachineControlPage";
import { TestMotorPage } from "@ui/machines/minimal_machines/motor_test_machine/TestMotorPage";
import { TestMotorControlPage } from "@ui/machines/minimal_machines/motor_test_machine/TestMotorControlPage";

import { MetricsGraphsPage } from "@ui/metrics/MetricsGraphsPage";
import { MetricsControlPage } from "@ui/metrics/MetricsControlPage";

import { WagoPower1Page } from "@ui/machines/wago_power/wago_power1/WagoPower1Page";
import { WagoPower1ControlPage } from "@ui/machines/wago_power/wago_power1/WagoPower1ControlPage";
import { WagoDoTestMachinePage } from "@ui/machines/minimal_machines/wagodotestmachine/WagoDoTestMachinePage";
import { WagoDoTestMachineControlPage } from "@ui/machines/minimal_machines/wagodotestmachine/WagoDoTestMachineControlPage";
import { Wago8chDioTestMachinePage } from "@ui/machines/minimal_machines/wago8chdiotestmachine/wago8chDioTestMachinePage";
import { Wago8chDioTestMachineControlRoute } from "@ui/machines/minimal_machines/wago8chdiotestmachine/wago8chDioTestMachineControlPage";
import { Wago750_501TestMachinePage } from "@ui/machines/minimal_machines/wago750501testmachine/Wago750_501TestMachinePage";
import { Wago750_501TestMachineControlPage } from "@ui/machines/minimal_machines/wago750501testmachine/Wago750_501TestMachineControlPage";
import { Wago750430DiMachinePage } from "@ui/machines/minimal_machines/wago750430dimachine/Wago750430DiMachinePage";
import { Wago750430DiMachineControlPage } from "@ui/machines/minimal_machines/wago750430dimachine/Wago750430DiMachineControlPage";
import { Wago750_553MachinePage } from "@ui/machines/minimal_machines/wago750553machine/Wago750_553MachinePage";
import { Wago750_553MachineControlPage } from "@ui/machines/minimal_machines/wago750553machine/Wago750_553MachineControlPage";

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

export const wago8chDioTestMachineRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wago8chdiotestmachine/$serial",
  component: () => <Wago8chDioTestMachinePage />,
});

export const wago8chDioTestMachineControlRoute = createRoute({
  getParentRoute: () => wago8chDioTestMachineRoute,
  path: "control",
  component: () => <Wago8chDioTestMachineControlRoute />,
});

export const wago750430DiMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wago750430dimachine/$serial",
  component: () => <Wago750430DiMachinePage />,
});

export const wago750430DiMachineControlRoute = createRoute({
  getParentRoute: () => wago750430DiMachineSerialRoute,
  path: "control",
  component: () => <Wago750430DiMachineControlPage />,
});

export const wago750_553MachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wago750553machine/$serial",
  component: () => <Wago750_553MachinePage />,
});

export const wago750_553MachineControlRoute = createRoute({
  getParentRoute: () => wago750_553MachineSerialRoute,
  path: "control",
  component: () => <Wago750_553MachineControlPage />,
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

export const wagoAiTestMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wagoaitestmachine/$serial",
  component: () => <WagoAiTestMachine />,
});

export const wagoAiTestMachineControlRoute = createRoute({
  getParentRoute: () => wagoAiTestMachineSerialRoute,
  path: "control",
  component: () => <WagoAiTestMachineControl />,
});

export const digitalInputTestMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "digitalInputTestMachine/$serial",
  component: () => <DigitalInputTestMachinePage />,
});

export const digitalInputTestMachineControlRoute = createRoute({
  getParentRoute: () => digitalInputTestMachineSerialRoute,
  path: "control",
  component: () => <DigitalInputTestMachineControlPage />,
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

export const testMotorSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "testmotor/$serial",
  component: () => <TestMotorPage />,
});

export const testMotorControlRoute = createRoute({
  getParentRoute: () => testMotorSerialRoute,
  path: "control",
  component: () => <TestMotorControlPage />,
});

export const testMachineStepperSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "testmachinestepper/$serial",
  component: () => <TestMachineStepperPage />,
});

export const testMachineStepperControlRoute = createRoute({
  getParentRoute: () => testMachineStepperSerialRoute,
  path: "control",
  component: () => <TestMachineStepperControlPage />,
});

// Leaf route: control page
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

export const laser1SettingsRoute = createRoute({
  getParentRoute: () => laser1SerialRoute,
  path: "settings",
  component: () => <Laser1SettingsPage />,
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

export const aquapath1SettingsRoute = createRoute({
  getParentRoute: () => aquapath1SerialRoute,
  path: "settings",
  component: () => <Aquapath1SettingsPage />,
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

export const wagoDoTestMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wagodotestmachine/$serial",
  component: () => <WagoDoTestMachinePage />,
});

export const wagoDoTestMachineControlRoute = createRoute({
  getParentRoute: () => wagoDoTestMachineSerialRoute,
  path: "control",
  component: () => <WagoDoTestMachineControlPage />,
});

export const wagoSerialSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wago_serial/$serial",
  component: () => <WagoSerialPage />,
});

export const wagoSerialControlRoute = createRoute({
  getParentRoute: () => wagoSerialSerialRoute,
  path: "control",
  component: () => <WagoSerialControlPage />,
});

export const wago750_501TestMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "wago750501testmachine/$serial",
  component: () => <Wago750_501TestMachinePage />,
});

export const wago750_501TestMachineControlRoute = createRoute({
  getParentRoute: () => wago750_501TestMachineSerialRoute,
  path: "control",
  component: () => <Wago750_501TestMachineControlPage />,
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
    branch: z.string().optional(),
    commit: z.string().optional(),
    tag: z.string().optional(),
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
  validateSearch: versionSearchSchema,
});

export const updateExecuteRoute = createRoute({
  getParentRoute: () => updateRoute,
  path: "execute",
  component: () => <UpdateExecutePage />,
  validateSearch: versionSearchSchema,
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
        laser1SettingsRoute,
      ]),

      testMachineSerialRoute.addChildren([testMachineControlRoute]),

      testMachineStepperSerialRoute.addChildren([
        testMachineStepperControlRoute,
      ]),

      analogInputTestMachineSerialRoute.addChildren([
        analogInputTestMachineControlRoute,
      ]),

      wago8chDioTestMachineRoute.addChildren([
        wago8chDioTestMachineControlRoute,
      ]),

      wagoAiTestMachineSerialRoute.addChildren([wagoAiTestMachineControlRoute]),

      wagoSerialSerialRoute.addChildren([wagoSerialControlRoute]),

      digitalInputTestMachineSerialRoute.addChildren([
        digitalInputTestMachineControlRoute,
      ]),

      ip20TestMachineSerialRoute.addChildren([ip20TestMachineControlRoute]),

      testMotorSerialRoute.addChildren([testMotorControlRoute]),

      aquapath1SerialRoute.addChildren([
        aquapath1ControlRoute,
        aquapath1GraphRoute,
        aquapath1SettingsRoute,
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

      wagoDoTestMachineSerialRoute.addChildren([wagoDoTestMachineControlRoute]),

      wago750_501TestMachineSerialRoute.addChildren([
        wago750_501TestMachineControlRoute,
      ]),

      wago750430DiMachineSerialRoute.addChildren([
        wago750430DiMachineControlRoute,
      ]),

      wago750_553MachineSerialRoute.addChildren([
        wago750_553MachineControlRoute,
      ]),
    ]),
  ]),
]);
