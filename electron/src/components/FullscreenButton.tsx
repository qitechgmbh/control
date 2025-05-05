"use client";

import React from "react";
import { Button } from "./ui/button";
import { Icon } from "./Icon";
import { useEffectAsync } from "@/lib/useEffectAsync";

export function FullscreenButton() {
  const [isFullscreen, setIsFullscreen] = React.useState(false);

  // We initalize button as fullscreen true if we are on QiTechOS
  useEffectAsync(async () => {
    const envInfo = await window.environment.getInfo();
    if (envInfo.qitechOs) {
      setIsFullscreen(true);
    } else {
      setIsFullscreen(false);
    }
  }, []);

  return (
    <Button
      onClick={() => {
        if (isFullscreen) {
          window.electronWindow.fullscreen(false);
          setIsFullscreen(false);
        } else {
          window.electronWindow.fullscreen(true);
          setIsFullscreen(true);
        }
      }}
      className="px-6 py-7"
      variant="ghost"
    >
      <Icon name={isFullscreen ? "lu:Minimize" : "lu:Maximize"} />
      {isFullscreen ? null : "Fullscreen"}
    </Button>
  );
}
