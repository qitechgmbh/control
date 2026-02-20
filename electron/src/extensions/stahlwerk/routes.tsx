import { createRoute, Outlet } from "@tanstack/react-router";

import React from "react";
import { z } from "zod";

import { Laser1ControlPage } from "@/extensions/stahlwerk/machines/ff01_mock/Page";
import { Laser1GraphsPage } from "@/machines/laser/laser1/Laser1Graph";
import { Laser1Page } from "@/machines/laser/laser1/Laser1Page";
import { Laser1PresetsPage } from "@/machines/laser/laser1/Laser1PresetsPage";
import { machinesRoute } from "@/routes/routes";

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