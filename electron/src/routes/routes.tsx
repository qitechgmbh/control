import { createRoute } from "@tanstack/react-router";
import { RootRoute } from "./__root";
import React from "react";
import { SidebarLayout } from "@/components/SidebarLayout";
import { Winder1Page } from "@/machines/winder1/Winder1Page";
import { ConfigurationPage } from "@/configuration/ConfigurationPage";
import { DevicesPage } from "@/configuration/DevicesPage";

// make a route tree like this
// _mainNavigation/machines/$machineSerial/winder1/general
// _mainNavigation/machines/$machineSerial/winder1/handbook
// _mainNavigation/machines/$machineSerial/winder2/general
// _mainNavigation/machines/$machineSerial/winder2/handbook
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
  path: "machines/$machineSerial",
});

export const winder1Route = createRoute({
  getParentRoute: () => machinesRoute,
  path: "winder1",
  component: () => <Winder1Page />,
});

export const winder1GeneralRoute = createRoute({
  getParentRoute: () => winder1Route,
  path: "general",
  component: () => <div>winder1 general</div>,
});

export const configurationRoute = createRoute({
  getParentRoute: () => sidebarRoute,
  path: "configuration",
  component: () => <ConfigurationPage />,
});

export const devicesRoute = createRoute({
  getParentRoute: () => configurationRoute,
  path: "devices",
  component: () => <DevicesPage />,
});

export const rootTree = RootRoute.addChildren([
  sidebarRoute.addChildren([
    machinesRoute.addChildren([
      winder1Route.addChildren([winder1GeneralRoute]),
    ]),
    configurationRoute.addChildren([devicesRoute]),
  ]),
]);
