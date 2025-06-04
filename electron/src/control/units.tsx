import { ReactNode } from "@tanstack/react-router";
import { IconName } from "@/components/Icon";
import React from "react";
import { LoadingSpinner } from "@/components/LoadingSpinner";

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
    case "m/min":
      return "lu:Gauge";
    case "mHz":
      return "lu:AudioWaveform";
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
    case "m/min":
      return "m/min";
    case "mHz":
      return "mHz";
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
    case "m/min":
      return "meters/minute";
    case "mHz":
      return "millihertz";
    default:
      return "";
  }
}

export const units = [
  "m/s",
  "mm",
  "rpm",
  "deg",
  "m",
  "C",
  "bar",
  "m/min",
  "mHz",
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
