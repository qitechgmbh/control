import React from "react";
import { Button } from "@/components/ui/button";
import { toggleTheme } from "@/helpers/theme_helpers";
import { Icon } from "./Icon";

export default function ToggleTheme() {
  return (
    <Button onClick={toggleTheme} size="icon">
      <Icon name="lu:Moon" />
    </Button>
  );
}
