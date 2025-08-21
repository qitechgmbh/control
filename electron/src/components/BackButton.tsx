"use client";

import React from "react";
import { Button } from "control-ui/components/ui/button";
import { useRouter } from "@tanstack/react-router";
import { Icon } from "control-ui/components/Icon";

export function BackButton() {
  const router = useRouter();
  return (
    <Button
      onClick={() => {
        router.history.back();
      }}
      className="h-full bg-neutral-100 text-black"
      variant="ghost"
    >
      <Icon name="lu:ChevronLeft" />
      Back
    </Button>
  );
}
