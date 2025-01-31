"use client";

import { useRouter } from "next/navigation";
import { Button } from "./ui/button";
import { ChevronLeft } from "lucide-react";

type Props = {};

export function BackButton() {
  const router = useRouter();
  return (
    <Button
      onClick={() => {
        router.back();
      }}
      variant="secondary"
    >
      <ChevronLeft size={24} />
      Zurück
    </Button>
  );
}
