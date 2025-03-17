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
    case "C":
      return "lu:Thermometer";
    case "bar":
      return "lu:Expand";
    case "deg":
      return "lu:TriangleRight";
    default:
      return "lu:ChartNoAxesColumn";
  }
}

export function renderUnitSymbol(unit: Unit | undefined): string {
  switch (unit) {
    case "m/s":
      return "m/s";
    case "mm":
      return "mm";
    case "m":
      return "m";
    case "rpm":
      return "rpm";
    case "C":
      return "°C";
    case "bar":
      return "bar";
    case "deg":
      return "deg";
    default:
      return "";
  }
}

// example
// value: 10.0
// -> valueString: "10" (custom input)
// this function will add prefix/suffix to the valueString but without unit symbol
// -> "10°"
export function renderUnitSyntax(
  valueString: string | undefined,
  unit: Unit | undefined,
): string {
  if (!valueString) {
    return "";
  }
  switch (unit) {
    case "C":
      return `${valueString}`;
    case "deg":
      return `${valueString}°`;
    default:
      return valueString;
  }
}

export function renderUnitSymbolLong(unit: Unit): string {
  switch (unit) {
    case "m/s":
      return "meters/second";
    case "mm":
      return "millimeters";
    case "m":
      return "meters";
    case "rpm":
      return "revolutions/minute";
    case "C":
      return "degrees Celsius";
    case "bar":
      return "bars";
    case "deg":
      return "degrees";
    default:
      return "";
  }
}

export const units = ["m/s", "mm", "rpm", "deg", "m", "C", "bar"] as const;

export type Unit = (typeof units)[number];
