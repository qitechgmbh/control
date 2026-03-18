"use client";

import React from "react";
import { Button } from "./ui/button";
import { Icon } from "./Icon";
import { getBridge } from "@ui/bridge";
import { useEffectAsync } from "@ui/lib/useEffectAsync";

export function FullscreenButton() {
  const [isFullscreen, setIsFullscreen] = React.useState(false);

  // We initalize button as fullscreen true if we are on QiTechOS
  useEffectAsync(async () => {
    const envInfo = await getBridge().environment.getInfo();
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
          getBridge().window.fullscreen(false);
          setIsFullscreen(false);
        } else {
          getBridge().window.fullscreen(true);
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
