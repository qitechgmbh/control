import React from "react";

import { createRoute } from "@tanstack/react-router";

import { machinesRoute } from "@/routes/routes";

import { Page } from "./pages/Page";
import { ControlPage } from "./pages/ControlPage";
import { GraphPage } from "./pages/GraphPage";

export const serialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "ff01/$serial",
  component: () => <Page />,
});

export const controlRoute = createRoute({
  getParentRoute: () => serialRoute,
  path: "control",
  component: () => <ControlPage />,
});

export const graphRoute = createRoute({
  getParentRoute: () => serialRoute,
  path: "graphs",
  component: () => <GraphPage />,
});

export const tree = serialRoute.addChildren([
    controlRoute,
    graphRoute,
    graphRoute
]);