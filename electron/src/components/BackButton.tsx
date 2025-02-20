"use client";

import React from "react";
import { Button } from "./ui/button";
import { ChevronLeft } from "lucide-react";
import { useRouter } from "@tanstack/react-router";

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
      <ChevronLeft size={24} />
      Zurück
    </Button>
  );
}
