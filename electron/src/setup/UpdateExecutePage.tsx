import { Alert } from "@/components/Alert";
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

  const handleClick = async () => {
    setIsUpdating(true);
    setTerminalLines([]);
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
  };

  return (
    <Page>
      <SectionTitle title="Apply Update" />
      <TouchButton
        className="w-max"
        icon="lu:CircleFadingArrowUp"
        onClick={handleClick}
        disabled={isUpdating}
        isLoading={isUpdating}
      >
        Apply Update
      </TouchButton>
      <Alert title="Update Procedure Info" variant="info">
        Please stay connected to the internet and leave the power on. The update
        procuedure takes a couple of minutes and reboots the machine afterwards.
      </Alert>
      <Terminal lines={terminalLines} className="h-160" />
    </Page>
  );
}
