import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import { updateExecute, updateCancel } from "@/helpers/update_helpers";
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
    const res = await updateExecute(
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

  const handleCancel = async () => {
    if (!isUpdating) return;

    try {
      const res = await updateCancel();
      if (res.success) {
        toast.info("Update cancelled successfully");
        setIsUpdating(false);
      } else {
        toast.error("Failed to cancel update: " + res.error);
      }
    } catch (error: any) {
      toast.error("Failed to cancel update: " + error.message);
    }
  };

  return (
    <Page>
      <SectionTitle title="Apply Update" />
      <div className="flex flex-row gap-4">
        <TouchButton
          className="w-max"
          icon="lu:CircleFadingArrowUp"
          onClick={handleClick}
          disabled={isUpdating}
          isLoading={isUpdating}
        >
          Apply Update
        </TouchButton>
        {isUpdating && (
          <TouchButton
            className="w-max"
            icon="lu:X"
            onClick={handleCancel}
            variant="destructive"
          >
            Cancel Update
          </TouchButton>
        )}
      </div>
      <Alert title="Update Procedure Info" variant="info">
        Please stay connected to the internet and leave the power on. The update
        procuedure takes a couple of minutes and reboots the machine afterwards.
      </Alert>
      <Terminal
        lines={terminalLines}
        className="h-160"
        exportPrefix="qitech_control_server_update"
      />
    </Page>
  );
}
