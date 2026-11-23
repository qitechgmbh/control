import React from "react";

type Props = {
  prefix?: React.ReactNode;
  prefixSpace?: boolean;
  value: React.ReactNode;
  suffixSpace?: boolean;
  suffix?: React.ReactNode;
};

export function Value({
  prefix,
  prefixSpace,
  value,
  suffixSpace,
  suffix,
}: Props) {
  return (
    <span className="rounded-full bg-neutral-100 p-1 px-2 font-mono">
      {prefix}
      {prefixSpace && <span className="font-sans">&thinsp;</span>}
      {value}
      {suffixSpace && <span className="font-sans">&thinsp;</span>}
      {suffix}
    </span>
  );
}

type HexProps = {
  value: number | null | undefined;
  bytes?: number;
};

export function Hex({ value, bytes = 1 }: HexProps) {
  let valueString = value?.toString(16);
  // pad with zeros

  const padding = bytes * 2 - (valueString?.length || 0);
  if (padding > 0) {
    valueString = "0".repeat(padding) + (valueString || "");
  }

  return (
    <Value
      prefix={<span className="text-neutral-400">0x</span>}
      value={<span>{valueString}</span>}
    />
  );
}

type BoolProps = {
  value: number | boolean | null | undefined;
};

export function Bool({ value }: BoolProps) {
  const boolValue =
    value === undefined ? undefined : value === null ? null : Boolean(value);
  const valueString =
    boolValue === undefined
      ? "undefined"
      : boolValue === null
        ? "null"
        : boolValue
          ? "true"
          : "false";
  return <Value value={valueString} />;
}

type UnitProps = {
  value: number | null | undefined;
  unit: "ns" | "us" | "ms" | "s";
};

export function Unit({ value, unit }: UnitProps) {
  const valueString = value?.toString();
  let unitString = "";
  switch (unit) {
    case "ns":
      unitString = "ns";
      break;
    case "us":
      unitString = "µs";
      break;
    case "ms":
      unitString = "ms";
      break;
    case "s":
      unitString = "s";
      break;
  }
  return (
    <Value
      value={<span>{valueString}</span>}
      suffixSpace
      suffix={<span className="text-neutral-400">{unitString}</span>}
    />
  );
}

type EthercatVendorIdProps = {
  value: number | null | undefined;
};

export function EthercatVendorId({ value }: EthercatVendorIdProps) {
  if (value === 2) {
    return <span>Beckhoff Automation</span>;
  } else {
    return <Hex value={value} />;
  }
}
