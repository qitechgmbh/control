import { createRoute } from "@tanstack/react-router";
import { RootRoute } from "./__root";
import React from "react";
import { SidebarLayout } from "@/components/SidebarLayout";
import { SetupPage } from "@/setup/SetupPage";
import { EthercatPage } from "@/setup/EthercatPage";
import { MachinesPage } from "@/setup/MachinesPage";
import { Winder1Page } from "@/machines/winder1/Winder1Page";
import { Winder1ControlPage } from "@/machines/winder1/Winder1ControlPage";

// make a route tree like this
// _mainNavigation/machines/winder1/$serial/control
// _mainNavigation/configuration/a
// _mainNavigation/configuration/b
// the mainNavigation has a custom layout
// the winder1 winder2 and configuration also have a custom layout
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

export const winder1SerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "winder1/$serial",
  component: () => <Winder1Page />,
});

export const winder1GeneralRoute = createRoute({
  getParentRoute: () => winder1SerialRoute,
  path: "control",
  component: () => <Winder1ControlPage />,
});

export const setupRoute = createRoute({
  getParentRoute: () => sidebarRoute,
  path: "setup",
  component: () => <SetupPage />,
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

export const rootTree = RootRoute.addChildren([
  sidebarRoute.addChildren([
    setupRoute.addChildren([ethercatRoute, setupMachinesRoute]),
    machinesRoute.addChildren([
      winder1SerialRoute.addChildren([winder1GeneralRoute]),
    ]),
  ]),
]);
