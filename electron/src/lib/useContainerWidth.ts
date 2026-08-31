import React, { useEffect } from "react";

export function useContainerDimensions(
  containerRef: React.RefObject<HTMLDivElement | null>,
) {
  const [width, setWidth] = React.useState(0);
  const [height, setHeight] = React.useState(0);

  React.useEffect(() => {
    const element = containerRef.current;
    if (!element) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setWidth(entry.contentRect.width);
        setHeight(entry.contentRect.height);
      }
    });

    resizeObserver.observe(element);

    // Set initial dimensions
    const rect = element.getBoundingClientRect();
    setWidth(rect.width);
    setHeight(rect.height);

    return () => {
      resizeObserver.disconnect();
    };
  }, [containerRef]);

  return { width, height };
}

export function useMaxContainerMaxDimension(
  ref: React.RefObject<HTMLDivElement | null>,
  refresh: number = 1000,
) {
  const [maxWidthA, setMaxWidthA] = React.useState(0);
  const [maxWidthB, setMaxWidthB] = React.useState(0);
  const [maxHeightA, setMaxHeightA] = React.useState(0);
  const [maxHeightB, setMaxHeightB] = React.useState(0);
  const [useA, setUseA] = React.useState(true);
  const [hasStarted, setHasStarted] = React.useState(false);

  const { width, height } = useContainerDimensions(ref);

  // Switch between A and B every refresh milliseconds
  useEffect(() => {
    const interval = setInterval(() => {
      if (!hasStarted) {
        setHasStarted(true);
      } else {
        setUseA((prev) => {
          const newUseA = !prev;
          // Reset the set we're about to start sampling
          if (newUseA) {
            // We're switching to return A, so reset B (which we'll sample)
            setMaxWidthB(0);
            setMaxHeightB(0);
          } else {
            // We're switching to return B, so reset A (which we'll sample)
            setMaxWidthA(0);
            setMaxHeightA(0);
          }
          return newUseA;
        });
      }
    }, refresh);

    return () => clearInterval(interval);
  }, [refresh, hasStarted]);

  // Update the dimension values
  useEffect(() => {
    if (!hasStarted) {
      // First cycle: sample both A and B
      if (width > maxWidthA) {
        setMaxWidthA(width);
      }
      if (width > maxWidthB) {
        setMaxWidthB(width);
      }
      if (height > maxHeightA) {
        setMaxHeightA(height);
      }
      if (height > maxHeightB) {
        setMaxHeightB(height);
      }
    } else {
      // After first cycle: update the inactive set
      if (useA) {
        // We're returning A, so sample for B
        if (width > maxWidthB) {
          setMaxWidthB(width);
        }
        if (height > maxHeightB) {
          setMaxHeightB(height);
        }
      } else {
        // We're returning B, so sample for A
        if (width > maxWidthA) {
          setMaxWidthA(width);
        }
        if (height > maxHeightA) {
          setMaxHeightA(height);
        }
      }
    }
  }, [
    width,
    height,
    maxWidthA,
    maxWidthB,
    maxHeightA,
    maxHeightB,
    useA,
    hasStarted,
  ]);

  return {
    maxWidth: useA ? maxWidthA : maxWidthB,
    maxHeight: useA ? maxHeightA : maxHeightB,
  };
}
