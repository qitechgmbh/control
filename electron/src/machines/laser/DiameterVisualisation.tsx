import React from "react";
import { TimeSeries } from "@/lib/timeseries";
import { cva } from "class-variance-authority";

type AnimateCableProps = {
  targetDiameter: number;
  lowTolerance: number;
  highTolerance: number;
  diameter: TimeSeries;
};

// Ring color depending on tolerance
const ringClass = cva("absolute rounded-full", {
  variants: {
    state: {
      inRange: "bg-green-200",
      outOfRange: "bg-gray-300",
    },
  },
});

// Dashed middle ring
const dashedRingClass = cva(
  "absolute rounded-full border border-dashed border-black z-15",
);

// Dynamic animated circle
const dynamicCircleClass = cva(
  "absolute rounded-full border-2 border-black bg-transparent z-30 transition-all duration-300 ease-in-out",
);

export function DiameterVisualisation({
  targetDiameter,
  lowTolerance,
  highTolerance,
  diameter,
}: AnimateCableProps) {
  const actualDiameter = diameter.current?.value ?? 0.0;

  const minDia = targetDiameter - lowTolerance;
  const maxDia = targetDiameter + highTolerance;
  const totalTolerance = lowTolerance + highTolerance;
  const midDia = minDia + (maxDia - minDia) * (lowTolerance / totalTolerance);

  const pixelMin = 100;
  const pixelMax = 200;
  const lerp = (
    value: number,
    inputMin: number,
    inputMax: number,
    outputMin: number,
    outputMax: number,
  ) => {
    return (
      outputMin +
      ((value - inputMin) * (outputMax - outputMin)) / (inputMax - inputMin)
    );
  };
  const pixelMid = lerp(midDia, minDia, maxDia, pixelMin, pixelMax);
  let dynamicDiameterPx: number | null = null;
  const clamp = (value: number, min: number, max: number) =>
    Math.min(Math.max(value, min), max);

  if (actualDiameter > 0) {
    if (actualDiameter < minDia) {
      dynamicDiameterPx = lerp(actualDiameter, 0, minDia, 0, pixelMin);
    } else if (actualDiameter <= maxDia) {
      dynamicDiameterPx = lerp(
        actualDiameter,
        minDia,
        maxDia,
        pixelMin,
        pixelMax,
      );
    } else {
      dynamicDiameterPx = lerp(
        actualDiameter,
        maxDia,
        maxDia * 1.5,
        pixelMax,
        pixelMax * 1.1,
      );
    }

    dynamicDiameterPx = clamp(
      lerp(actualDiameter, minDia, maxDia, pixelMin, pixelMax),
      pixelMin,
      pixelMax * 1.1,
    );
  }

  const bigRadius = pixelMax / 2;
  const smallRadius = pixelMin / 2;
  const midRadius = pixelMid / 2;
  const dynamicRadius = dynamicDiameterPx ? dynamicDiameterPx / 2 : 0;
  console.log(dynamicRadius);
  const inTolerance = actualDiameter >= minDia && actualDiameter <= maxDia;

  const centerStyle = (radius: number) => ({
    width: radius * 2,
    height: radius * 2,
    top: `${bigRadius - radius}px`,
    left: `${bigRadius - radius}px`,
  });

  return (
    <div
      className="relative mx-auto"
      style={{ width: pixelMax, height: pixelMax }}
    >
      {/* Outer ring background */}
      <div
        className={`${ringClass({
          state: inTolerance ? "inRange" : "outOfRange",
        })} top-0 left-0 z-10`}
        style={{ width: pixelMax, height: pixelMax }}
      />

      {/* Inner white circle to cut out the center */}
      <div
        className="absolute z-20 rounded-full bg-white"
        style={centerStyle(smallRadius)}
      />

      {/* Dashed ring in the middle */}
      <div className={dashedRingClass()} style={centerStyle(midRadius)} />

      {/* Dynamic circle */}
      {dynamicDiameterPx !== null && (
        <div
          className={dynamicCircleClass()}
          style={centerStyle(dynamicRadius)}
        />
      )}
    </div>
  );
}
