import React from "react";
import { Button } from "@ui/components/ui/button";
import { toggleTheme } from "@ui/helpers/theme_helpers";
import { Icon } from "./Icon";

export default function ToggleTheme() {
  return (
    <Button onClick={toggleTheme} size="icon">
      <Icon name="lu:Moon" />
    </Button>
  );
}
