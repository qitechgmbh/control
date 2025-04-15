import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import { updatExecute } from "@/helpers/update_helpers";
import { useSearch } from "@tanstack/react-router";
import React, { useState } from "react";
import { toast } from "sonner";

export function UpdateExecutePage() {
  const search = useSearch({
    from: "/_sidebar/setup/update/execute",
  });

  const [isUpdating, setIsUpdating] = React.useState(false);

  const [terminalLines, setTerminalLines] = useState<string[]>([]);

  return (
    <Page>
      <SectionTitle title="Apply Update" />
      <TouchButton
        className="w-max"
        icon="lu:CircleFadingArrowUp"
        onClick={async () => {
          setIsUpdating(true);
          const res = await updatExecute(
            {
              ...search,
              githubToken: search.githubToken || undefined,
            },
            (log: string) => {
              setTerminalLines((prev) => [...prev, log]);
            },
          );
          setIsUpdating(false);
          if (res.success) {
            toast.success("Update applied successfully");
          } else {
            toast.error("Update failed: " + res.error);
          }
        }}
        disabled={isUpdating}
        isLoading={isUpdating}
      >
        Apply Update Now
      </TouchButton>
      <SectionTitle title="Log" />
      <Terminal lines={terminalLines} className="h-128" />
    </Page>
  );
}
