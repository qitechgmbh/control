import React, { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { useKioskStore } from "@/stores/kioskStore";
import { Icon } from "./Icon";

const HOLD_DURATION_MS = 10_000;

// Above everything, including dialogs and sonner toasts
const OVERLAY_Z_INDEX = 2147483647;

function toggleKiosk() {
  const { locked, setLocked } = useKioskStore.getState();
  setLocked(!locked);
  if (locked) {
    toast.success("Kiosk mode unlocked");
  } else {
    toast.success("Kiosk mode locked");
  }
}

type KioskHoldAreaProps = {
  className?: string;
  children?: React.ReactNode;
};

// Toggles the kiosk lock after being held for HOLD_DURATION_MS
export function KioskHoldArea({ className, children }: KioskHoldAreaProps) {
  const [progress, setProgress] = useState(0);
  // Pointer holding the area; ignore other fingers until it is released
  const pointerId = useRef<number | null>(null);
  const frame = useRef<number | null>(null);

  const stopHold = () => {
    if (frame.current !== null) {
      cancelAnimationFrame(frame.current);
      frame.current = null;
    }
    pointerId.current = null;
    setProgress(0);
  };

  useEffect(() => stopHold, []);

  const onPointerDown = (event: React.PointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    if (pointerId.current !== null) return;
    pointerId.current = event.pointerId;
    // Keep receiving events if the finger drifts off the logo
    event.currentTarget.setPointerCapture(event.pointerId);

    const start = performance.now();
    const tick = (now: number) => {
      const value = Math.min((now - start) / HOLD_DURATION_MS, 1);
      setProgress(value);
      if (value >= 1) {
        // Stays "held" until release so the lock doesn't flip back
        frame.current = null;
        toggleKiosk();
        return;
      }
      frame.current = requestAnimationFrame(tick);
    };
    frame.current = requestAnimationFrame(tick);
  };

  const onPointerEnd = (event: React.PointerEvent<HTMLDivElement>) => {
    if (event.pointerId !== pointerId.current) return;
    stopHold();
  };

  return (
    <div
      className={`relative touch-none select-none ${className ?? ""}`}
      onPointerDown={onPointerDown}
      onPointerUp={onPointerEnd}
      onPointerCancel={onPointerEnd}
      onContextMenu={(event) => event.preventDefault()}
    >
      {children}
      {progress > 0 && (
        <div className="absolute right-4 bottom-1 left-4 h-1 overflow-hidden rounded-full bg-neutral-300">
          <div
            className="h-full bg-neutral-800"
            style={{ width: `${progress * 100}%` }}
          />
        </div>
      )}
    </div>
  );
}

// Swallows all input while the kiosk is locked. Only the hold area on top of
// the sidebar logo stays interactive to unlock again.
export function KioskOverlay() {
  const locked = useKioskStore((state) => state.locked);

  useEffect(() => {
    if (!locked) return;

    // Drop focus so a focused input can't keep receiving input
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }

    // Keys are also blocked in the main process, wheel covers ctrl+wheel zoom
    const block = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
    };
    const options = { capture: true, passive: false };
    const events = ["keydown", "keyup", "keypress", "wheel"];
    events.forEach((type) => window.addEventListener(type, block, options));
    return () => {
      events.forEach((type) =>
        window.removeEventListener(type, block, options),
      );
    };
  }, [locked]);

  if (!locked) return null;

  return (
    <div
      className="fixed inset-0 touch-none select-none"
      style={{ zIndex: OVERLAY_Z_INDEX }}
      onContextMenu={(event) => event.preventDefault()}
    >
      {/* Same position and size as the logo in SidebarLayout */}
      <KioskHoldArea className="absolute top-0 left-0 h-18 w-48">
        <Icon
          name="lu:Lock"
          className="absolute top-2 right-2 size-4 text-neutral-600"
        />
      </KioskHoldArea>
    </div>
  );
}
