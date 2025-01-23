"use client";

import { MessageResponse } from "@/hooks/useSocketio";
import { useEffect, useState } from "react";

type Props = {
  messageResponse: MessageResponse<unknown>;
};

function formatMilliseconds(milliseconds: number) {
  console.log("ms", milliseconds);
  if (milliseconds < 1000) {
    return `Now`;
  } else if (milliseconds < 60000) {
    return `${(milliseconds / 1000).toFixed(0)}s ago`;
  } else if (milliseconds < 3600000) {
    return `${(milliseconds / 60000).toFixed(0)}:${(
      (milliseconds % 60000) /
      1000
    )
      .toFixed(0)
      .padStart(2, "0")}m ago`;
  } else if (milliseconds < 86400000) {
    return `${(milliseconds / 3600000).toFixed(0)}:${(
      (milliseconds % 3600000) /
      60000
    )
      .toFixed(0)
      .padStart(2, "0")}h ago`;
  } else {
    return `${(milliseconds / 86400000).toFixed(1)}d ago`;
  }
}

export function RefreshIndicator({ messageResponse }: Props) {
  const [now, setNow] = useState<number>(Date.now());

  // every 100ms update the time
  useEffect(() => {
    const interval = setInterval(() => {
      setNow(Date.now());
    }, 100);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex items-center gap-1.5 px-2 p-1 bg-neutral-100 rounded-full w-fit">
      <div
        className={`w-2.5 h-2.5 rounded-full ${
          messageResponse.status === "no_data"
            ? "bg-neutral-400"
            : messageResponse.status === "error"
            ? "bg-red-500"
            : messageResponse.status === "warning"
            ? "bg-yellow-500"
            : "bg-green-500 animate-[pulse_500ms_ease-in-out]"
        }`}
      />
      <span
        className={`text-neutral-500 text-xs ${
          messageResponse.status !== "no_data" &&
          "animate-[colorFadeToGray_250ms_ease-in-out_forwards]"
        }`}
      >
        {messageResponse.status === "no_data"
          ? "Loading"
          : messageResponse.status === "error"
          ? messageResponse.error
          : messageResponse.status === "warning"
          ? messageResponse.warning
          : formatMilliseconds(now - messageResponse.ts)}
      </span>
    </div>
  );
}
