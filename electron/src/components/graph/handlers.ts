import { RefObject } from "react";
import uPlot from "uplot";
import { GraphConfig, HandlerRefs } from "./types";
import { FPS_30 } from "@/lib/constants";

export interface HandlerCallbacks {
  updateYAxisScale: (xMin?: number, xMax?: number) => void;
  setViewMode: (mode: "default" | "all" | "manual") => void;
  setIsLiveMode: (isLive: boolean) => void;
  onZoomChange?: (graphId: string, range: { min: number; max: number }) => void;
  onViewModeChange?: (
    graphId: string,
    mode: "default" | "all" | "manual",
    isLive: boolean,
  ) => void;
}

export function createEventHandlers(
  containerRef: RefObject<HTMLDivElement | null>,
  uplotRef: RefObject<uPlot | null>,
  handlerRefs: HandlerRefs,
  callbacks: HandlerCallbacks,
  newData: any,
  config: GraphConfig,
  graphId: string,
  manualScaleRef: RefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>,
  width: number,
) {
  // Timeout references for debounced callbacks
  const callbackTimeouts = {
    zoom: null as NodeJS.Timeout | null,
    viewMode: null as NodeJS.Timeout | null,
  };

  // Debounced callback handlers
  const debouncedCallbacks = {
    onZoomChange: (range: { min: number; max: number }) => {
      if (callbackTimeouts.zoom) clearTimeout(callbackTimeouts.zoom);
      callbackTimeouts.zoom = setTimeout(() => {
        callbacks.onZoomChange?.(graphId, range);
      }, FPS_30);
    },
    onViewModeChange: (mode: "default" | "all" | "manual", isLive: boolean) => {
      if (callbackTimeouts.viewMode) clearTimeout(callbackTimeouts.viewMode);
      callbackTimeouts.viewMode = setTimeout(() => {
        callbacks.onViewModeChange?.(graphId, mode, isLive);
      }, FPS_30);
    },
  };

  // Updates the graph's scale and synchronizes it with other graphs
  const updateScaleAndSync = (newMin: number, newMax: number) => {
    if (!uplotRef.current) return;

    uplotRef.current.setScale("x", { min: newMin, max: newMax });
    callbacks.updateYAxisScale(newMin, newMax);

    manualScaleRef.current = {
      x: { min: newMin, max: newMax },
      y: {
        min: uplotRef.current.scales.y?.min ?? 0,
        max: uplotRef.current.scales.y?.max ?? 1,
      },
    };

    callbacks.setViewMode("manual");
    callbacks.setIsLiveMode(false);

    debouncedCallbacks.onZoomChange({ min: newMin, max: newMax });
    debouncedCallbacks.onViewModeChange("manual", false);
  };

  // Handles touch start events for drag and pinch gestures
  const handleTouchStart = (e: TouchEvent) => {
    const touch = e.touches[0];
    handlerRefs.touchStartRef.current = {
      x: touch.clientX,
      y: touch.clientY,
      time: Date.now(),
    };
    handlerRefs.touchDirectionRef.current = "unknown";

    if (e.touches.length === 2) {
      handlerRefs.isPinchingRef.current = true;
      handlerRefs.isDraggingRef.current = false;
      handlerRefs.touchDirectionRef.current = "horizontal";

      const touch1 = e.touches[0];
      const touch2 = e.touches[1];

      const distance = Math.sqrt(
        Math.pow(touch2.clientX - touch1.clientX, 2) +
          Math.pow(touch2.clientY - touch1.clientY, 2),
      );
      handlerRefs.lastPinchDistanceRef.current = distance;

      handlerRefs.pinchCenterRef.current = {
        x: (touch1.clientX + touch2.clientX) / 2,
        y: (touch1.clientY + touch2.clientY) / 2,
      };

      e.preventDefault();
    }
  };

  // Handles touch move events for drag and pinch gestures
  const handleTouchMove = (e: TouchEvent) => {
    if (!handlerRefs.touchStartRef.current) return;

    if (e.touches.length === 1) {
      const touch = e.touches[0];
      const deltaX = Math.abs(
        touch.clientX - handlerRefs.touchStartRef.current.x,
      );
      const deltaY = Math.abs(
        touch.clientY - handlerRefs.touchStartRef.current.y,
      );
      const timeDelta = Date.now() - handlerRefs.touchStartRef.current.time;

      if (
        handlerRefs.touchDirectionRef.current === "unknown" &&
        deltaX > 20 &&
        deltaY < 10 &&
        deltaX > deltaY * 4 &&
        timeDelta < 500
      ) {
        handlerRefs.touchDirectionRef.current = "horizontal";
        handlerRefs.isDraggingRef.current = true;
        handlerRefs.lastDragXRef.current = touch.clientX;
        e.preventDefault();
      } else if (handlerRefs.touchDirectionRef.current === "unknown") {
        return;
      }

      if (
        handlerRefs.touchDirectionRef.current === "horizontal" &&
        handlerRefs.isDraggingRef.current
      ) {
        e.preventDefault();

        const currentX = touch.clientX;
        const dragDelta = currentX - (handlerRefs.lastDragXRef.current || 0);
        handlerRefs.lastDragXRef.current = currentX;

        if (uplotRef.current && Math.abs(dragDelta) > 2) {
          const xScale = uplotRef.current.scales.x;
          if (xScale && xScale.min !== undefined && xScale.max !== undefined) {
            const pixelToTime = (xScale.max - xScale.min) / width;
            const timeDelta = -dragDelta * pixelToTime;

            const newMin = xScale.min + timeDelta;
            const newMax = xScale.max + timeDelta;

            updateScaleAndSync(newMin, newMax);
          }
        }
      }
    } else if (e.touches.length === 2 && handlerRefs.isPinchingRef.current) {
      e.preventDefault();

      const touch1 = e.touches[0];
      const touch2 = e.touches[1];

      const newDistance = Math.sqrt(
        Math.pow(touch2.clientX - touch1.clientX, 2) +
          Math.pow(touch2.clientY - touch1.clientY, 2),
      );

      if (handlerRefs.lastPinchDistanceRef.current && uplotRef.current) {
        const scaleFactor =
          newDistance / handlerRefs.lastPinchDistanceRef.current;
        const xScale = uplotRef.current.scales.x;

        if (
          xScale &&
          xScale.min !== undefined &&
          xScale.max !== undefined &&
          handlerRefs.pinchCenterRef.current
        ) {
          const rect = containerRef.current?.getBoundingClientRect();
          if (rect) {
            const touchXRelative =
              (handlerRefs.pinchCenterRef.current.x - rect.left) / rect.width;
            const centerTime =
              xScale.min + (xScale.max - xScale.min) * touchXRelative;

            const currentRange = xScale.max - xScale.min;
            const newRange = currentRange / scaleFactor;

            const leftRatio = (centerTime - xScale.min) / currentRange;
            const rightRatio = (xScale.max - centerTime) / currentRange;

            const newMin = centerTime - newRange * leftRatio;
            const newMax = centerTime + newRange * rightRatio;
            updateScaleAndSync(newMin, newMax);
          }
        }
      }

      handlerRefs.lastPinchDistanceRef.current = newDistance;
    }
  };

  // Handles touch end events and resets state
  const handleTouchEnd = (e: TouchEvent) => {
    setTimeout(() => {
      if (e.touches.length === 0) {
        handlerRefs.isDraggingRef.current = false;
        handlerRefs.isPinchingRef.current = false;
        handlerRefs.lastDragXRef.current = null;
        handlerRefs.lastPinchDistanceRef.current = null;
        handlerRefs.pinchCenterRef.current = null;
        handlerRefs.touchStartRef.current = null;
        handlerRefs.touchDirectionRef.current = "unknown";
      } else if (e.touches.length === 1 && handlerRefs.isPinchingRef.current) {
        handlerRefs.isPinchingRef.current = false;
        handlerRefs.lastPinchDistanceRef.current = null;
        handlerRefs.pinchCenterRef.current = null;

        const touch = e.touches[0];
        handlerRefs.touchStartRef.current = {
          x: touch.clientX,
          y: touch.clientY,
          time: Date.now(),
        };
        handlerRefs.touchDirectionRef.current = "unknown";
        handlerRefs.isDraggingRef.current = false;
      }

      if (
        handlerRefs.touchDirectionRef.current === "horizontal" &&
        handlerRefs.isDraggingRef.current
      ) {
        e.preventDefault();
      }
    }, 50);
  };

  // Handles mouse down events for zooming
  const handleMouseDown = (e: MouseEvent) => {
    if (e.button === 0) {
      handlerRefs.isUserZoomingRef.current = true;
    }
  };

  // Prevents default behavior for wheel events
  const handleWheel = (e: WheelEvent) => {
    e.preventDefault();
  };

  // Cleanup function to clear timeouts
  const cleanup = () => {
    if (callbackTimeouts.zoom) clearTimeout(callbackTimeouts.zoom);
    if (callbackTimeouts.viewMode) clearTimeout(callbackTimeouts.viewMode);
  };

  return {
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
    handleMouseDown,
    handleWheel,
    cleanup,
  };
}

export function attachEventHandlers(
  containerElement: HTMLDivElement,
  handlers: ReturnType<typeof createEventHandlers>,
) {
  containerElement.addEventListener("touchstart", handlers.handleTouchStart, {
    passive: false,
  });
  containerElement.addEventListener("touchmove", handlers.handleTouchMove, {
    passive: false,
  });
  containerElement.addEventListener("touchend", handlers.handleTouchEnd, {
    passive: false,
  });
  containerElement.addEventListener("mousedown", handlers.handleMouseDown);
  containerElement.addEventListener("wheel", handlers.handleWheel, {
    passive: false,
  });

  return () => {
    containerElement.removeEventListener(
      "touchstart",
      handlers.handleTouchStart,
    );
    containerElement.removeEventListener("touchmove", handlers.handleTouchMove);
    containerElement.removeEventListener("touchend", handlers.handleTouchEnd);
    containerElement.removeEventListener("mousedown", handlers.handleMouseDown);
    containerElement.removeEventListener("wheel", handlers.handleWheel);

    handlers.cleanup();
  };
}
