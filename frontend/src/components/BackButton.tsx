"use client";

import { Button } from "./ui/button";
import { ChevronLeft } from "lucide-react";

export function BackButton() {
  return (
    <Button onClick={() => {}} variant="secondary">
      <ChevronLeft size={24} />
      Zurück
    </Button>
  );
}
