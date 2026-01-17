import { ReactNode } from "react";
import { IconName } from "@/components/Icon";
import React from "react";
import { LoadingSpinner } from "@/components/LoadingSpinner";

export function getUnitIcon(unit: Unit): IconName {
  switch (unit) {
    case "m/s":
      return "lu:Gauge";
    case "mm":
      return "lu:Ruler";
    case "cm":
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
    case "m/min":
      return "lu:Gauge";
    case "mHz":
      return "lu:AudioWaveform";
    case "Hz":
      return "lu:AudioWaveform";
    case "W":
    case "V":
    case "A":
    case "mA":
      return "lu:Zap";
    case "kWh":
      return "lu:BatteryFull";
    case "l/min":
      return "lu:Waves";
    case "%":
      return "lu:ChartNoAxesColumn";
    case "µs":
      return "lu:RefreshCcw";
    case "s":
      return "lu:Clock";
    case "MiB":
      return "lu:MemoryStick";
    case "Mbit/s":
      return "lu:Network";
    case "/s":
      return "lu:Gauge";
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
    case "cm":
      return "cm";
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
    case "m/min":
      return "m/min";
    case "mHz":
      return "mHz";
    case "Hz":
      return "Hz";
    case "W":
      return "W";
    case "V":
      return "V";
    case "mA":
      return "mA";
    case "A":
      return "A";
    case "kWh":
      return "kWh";
    case "l/min":
      return "l/min";
    case "%":
      return "%";
    case "µs":
      return "µs";
    case "s":
      return "s";
    case "MiB":
      return "MiB";
    case "Mbit/s":
      return "Mbit/s";
    case "/s":
      return "/s";
    default:
      return "";
  }
}

// this function will add prefix/suffix to the valueString but without unit symbol.
// example
//   const f = 3.1415;
//   renderUnitSyntax(f.toFixed(2), "deg")
// result: "3.14°"
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
    case "cm":
      return "centimeters";
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
    case "m/min":
      return "meters/minute";
    case "mHz":
      return "millihertz";
    case "Hz":
      return "hertz";
    case "W":
      return "watts";
    case "V":
      return "volts";
    case "mA":
      return "milliamperes";
    case "A":
      return "amperes";
    case "kWh":
      return "kilowatt-hours";
    case "l/min":
      return "liters/minute";
    case "%":
      return "percent";
    case "µs":
      return "microseconds";
    case "s":
      return "seconds";
    case "MiB":
      return "mebibytes";
    case "Mbit/s":
      return "megabits/second";
    case "/s":
      return "per second";
    default:
      return "";
  }
}

export const units = [
  "m/s",
  "mm",
  "cm",
  "rpm",
  "deg",
  "m",
  "C",
  "bar",
  "m/min",
  "mHz",
  "Hz",
  "W",
  "V",
  "mA",
  "A",
  "kWh",
  "l/min",
  "%",
  "µs",
  "s",
  "MiB",
  "Mbit/s",
  "/s",
] as const;

export type Unit = (typeof units)[number];

export function renderValueToReactNode<T>(
  value?: T | undefined | null,
  unit?: Unit,
  renderValue?: (value: T) => string,
): ReactNode {
  if (value === undefined || value === null) {
    return (
      // We use an invisible `1` to mock the height if a character was there
      <span className="relative inline-flex items-center justify-center">
        <span className="opacity-0">1</span>
        <LoadingSpinner />
      </span>
    );
  }

  const valueString = renderValue ? renderValue(value) : value.toString();

  const valueStringSplit = valueString.split(".");
  const reactNodes: ReactNode[] = [];
  for (let i = 0; i < valueStringSplit.length; i++) {
    if (i > 0) {
      reactNodes.push(
        <span key={`dot-${i}`} className="font-serif">
          .
        </span>,
      );
    }
    reactNodes.push(<span key={`value-${i}`}>{valueStringSplit[i]}</span>);
  }

  return <>{reactNodes}</>;
}
