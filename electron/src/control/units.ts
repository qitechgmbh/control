import { IconName } from "@/components/Icon";

export function getUnitIcon(unit: Unit): IconName {
  switch (unit) {
    case "m/s":
      return "lu:Gauge";
    case "mm":
      return "lu:Ruler";
    case "m":
      return "lu:Ruler";
    case "rpm":
      return "lu:Gauge";
    case "deg":
      return "lu:TriangleRight";
    default:
      return "lu:ChartNoAxesColumn";
  }
}

export const units = ["m/s", "mm", "rpm", "deg", "m"] as const;

export type Unit = (typeof units)[number];
