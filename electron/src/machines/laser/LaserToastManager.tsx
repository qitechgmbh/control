import React, { useEffect, useRef } from "react";
import { useLaser1 } from "./laser1/useLaser1";
import { toast } from "sonner";

export function LaserToastManager() {
  const { state } = useLaser1();
  const toastRef = useRef<string | number | null>(null);

  useEffect(() => {
    if (!state?.laser_state) return;

    if (state.laser_state.in_tolerance === false && !toastRef.current) {
      toastRef.current = toast(
        <div className="bg-red-500 text-white p-4 rounded-lg shadow-lg flex flex-col gap-1 w-80">
          <strong>Warning!</strong>
          <span>Diameter out of tolerance!</span>
          <button
            className="self-end font-bold mt-2 hover:text-gray-200"
            onClick={() => {
              toast.dismiss(toastRef.current!);
              toastRef.current = null;
            }}
          >
            ×
          </button>
        </div>,
        { duration: Infinity, position: "top-center", style: { background: "transparent", padding: 0, boxShadow: "none", border: "none" } }
      );
    }
  }, [state?.laser_state?.in_tolerance]);

  return null;
}

