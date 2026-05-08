import type { ComponentType } from "react";
import type { MachineProperties } from "./types";

export type MachineRouteNode = {
  /** path segment, e.g. "winder2/$serial" or "control" */
  path: string;
  component: ComponentType;
  /** child routes; segments are relative to this node */
  children?: MachineRouteNode[];
};

export type MachineModule = {
  /** must match backend slug() and properties.slug */
  slug: string;
  properties: MachineProperties;
  /** the $serial-level route descriptor for this machine */
  route: MachineRouteNode;
};
