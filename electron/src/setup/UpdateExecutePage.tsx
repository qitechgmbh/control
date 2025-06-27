import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import {
  updateExecute,
  cancelCurrentUpdate,
  resetUpdateCancellation,
} from "@/helpers/update_helpers";
import { useUpdateStore } from "@/stores/updateStore";
import { useSearch } from "@tanstack/react-router";
import React, { useEffect, useState } from "react";
import { toast } from "sonner";

export function UpdateExecutePage() {
  const search = useSearch({
    from: "/_sidebar/setup/update/execute",
  });

  const {
    isUpdating,
    logs,
    updateResult,
    startUpdate,
    addLog,
    finishUpdate,
    cancelUpdate,
    clearLogs,
  } = useUpdateStore();

  const [shouldCancel, setShouldCancel] = useState(false);

  // Clear terminal immediately when component mounts
  useEffect(() => {
    clearLogs();
  }, [clearLogs]);

  // Check if user navigated back to an ongoing update
  useEffect(() => {
    if (isUpdating && updateResult === null) {
      // Update is in progress, reset shouldCancel
      setShouldCancel(false);
    }
  }, [isUpdating, updateResult]);

  const handleStartUpdate = async () => {
    // Reset all cancellation state first
    setShouldCancel(false);
    resetUpdateCancellation();

    const source = {
      ...search,
      githubToken: search.githubToken || undefined,
    };

    // Start update (this will reset updateResult and set isUpdating = true)
    startUpdate(source);

    const res = await updateExecute(
      source,
      (log: string) => {
        addLog(log);
      },
      () => shouldCancel,
    );

    finishUpdate(res);

    if (res.success) {
      toast.success("Update applied successfully");
    } else if (res.error !== "Update was cancelled by user") {
      toast.error("Update failed: " + res.error);
    }
  };

  const handleCancelUpdate = () => {
    setShouldCancel(true);
    cancelCurrentUpdate();
    cancelUpdate();
    toast.info("Cancelling update...");
  };

  return (
    <Page>
      <SectionTitle title="Apply Update" />

      <div className="flex gap-4">
        {/* Show Apply Update button when not updating */}
        {!isUpdating && (
          <TouchButton
            className="w-max"
            icon="lu:CircleFadingArrowUp"
            onClick={handleStartUpdate}
          >
            Apply Update
          </TouchButton>
        )}

        {/* Show Cancel button only when update is actively running */}
        {isUpdating && (
          <TouchButton
            className="w-max"
            icon="lu:X"
            onClick={handleCancelUpdate}
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

      {updateResult && !isUpdating && (
        <Alert
          title={updateResult.success ? "Update Successful" : "Update Failed"}
          variant={updateResult.success ? "info" : "error"}
        >
          {updateResult.success
            ? "The update has been applied successfully. The system will reboot shortly."
            : `Update failed: ${updateResult.error}`}
        </Alert>
      )}

      <Terminal
        lines={logs}
        className="h-160"
        exportPrefix="qitech_control_server_update"
      />
    </Page>
  );
}
