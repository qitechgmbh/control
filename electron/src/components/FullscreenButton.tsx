"use client";

import React from "react";
import { Button } from "./ui/button";
import { Icon } from "./Icon";

export function FullscreenButton() {
  const [isFullscreen, setIsFullscreen] = React.useState(false);
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
