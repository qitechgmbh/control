import React from "react";
import { TimeSeries } from "@/lib/timeseries";
import { cva } from "class-variance-authority";

type DiameterVisualisationProps = {
  targetDiameter: number;
  lowTolerance: number;
  highTolerance: number;
  diameter: TimeSeries;
  x_diameter?: TimeSeries;
  y_diameter?: TimeSeries;
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

// Dynamic animated circle/ellipse
const dynamicCircleClass = cva(
  "absolute rounded-full border-2 border-black bg-transparent z-30 transition-all duration-300 ease-in-out",
);

export function DiameterVisualisation({
  targetDiameter,
  lowTolerance,
  highTolerance,
  diameter,
  x_diameter,
  y_diameter,
}: DiameterVisualisationProps) {
  const actualDiameter = diameter.current?.value ?? 0.0;
  const actualX = x_diameter?.current?.value ?? actualDiameter;
  const actualY = y_diameter?.current?.value ?? actualDiameter;

  const minDia = targetDiameter - lowTolerance;
  const maxDia = targetDiameter + highTolerance;

  const pixelMin = 100;
  const pixelMax = 200;

  const lerp = (
    value: number,
    inputMin: number,
    inputMax: number,
    outputMin: number,
    outputMax: number,
  ) => {
    if (inputMax === inputMin) {
      return (outputMin + outputMax) / 2;
    }
    return (
      outputMin +
      ((value - inputMin) * (outputMax - outputMin)) / (inputMax - inputMin)
    );
  };

  const clamp = (value: number, min: number, max: number) =>
    Math.min(Math.max(value, min), max);

  const radiusX = clamp(
    lerp(actualX, minDia, maxDia, pixelMin / 2, pixelMax / 2),
    pixelMin / 2,
    pixelMax / 2,
  );

  const radiusY = clamp(
    lerp(actualY, minDia, maxDia, pixelMin / 2, pixelMax / 2),
    pixelMin / 2,
    pixelMax / 2,
  );

  const bigRadius = pixelMax / 2;

  // Center the shape
  const centerStyle = (rx: number, ry: number) => ({
    width: rx * 2,
    height: ry * 2,
    top: `${bigRadius - ry}px`,
    left: `${bigRadius - rx}px`,
  });

  // Check if current measurement is in tolerance
  const inTolerance =
    actualX >= minDia &&
    actualX <= maxDia &&
    actualY >= minDia &&
    actualY <= maxDia;

  return (
    <div
      className="relative mx-auto"
      style={{ width: pixelMax, height: pixelMax }}
    >
      <div
        className={`${ringClass({ state: inTolerance ? "inRange" : "outOfRange" })} top-0 left-0 z-10`}
        style={{ width: pixelMax, height: pixelMax }}
      />
      <div
        className="absolute z-20 rounded-full bg-white"
        style={centerStyle(pixelMin / 2, pixelMin / 2)}
      />
      <div
        className={dashedRingClass()}
        style={centerStyle(pixelMax / 2, pixelMax / 2)}
      />
      <div
        className={dynamicCircleClass()}
        style={{
          ...centerStyle(radiusX, radiusY),
          /* rotate by 45deg to mimic the rotated axis of the Laser */
          transform: "rotate(45deg)",
        }}
      />
    </div>
  );
}
