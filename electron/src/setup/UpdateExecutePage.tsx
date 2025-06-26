import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import { updateExecute, cancelCurrentUpdate } from "@/helpers/update_helpers";
import { useUpdateStore } from "@/stores/updateStore";
import { useSearch, useNavigate } from "@tanstack/react-router";
import React, { useEffect, useState } from "react";
import { toast } from "sonner";

export function UpdateExecutePage() {
  const search = useSearch({
    from: "/_sidebar/setup/update/execute",
  });
  const navigate = useNavigate();

  const {
    isUpdating,
    logs,
    updateResult,
    startUpdate,
    addLog,
    finishUpdate,
    cancelUpdate,
    resetUpdate,
  } = useUpdateStore();

  const [shouldCancel, setShouldCancel] = useState(false);

  // Check if user navigated back to an ongoing update
  useEffect(() => {
    if (isUpdating && updateResult === null) {
      // Update is in progress, reset shouldCancel
      setShouldCancel(false);
    }
  }, [isUpdating, updateResult]);

  const handleStartUpdate = async () => {
    const source = {
      ...search,
      githubToken: search.githubToken || undefined,
    };
    
    startUpdate(source);
    setShouldCancel(false);
    
    const res = await updateExecute(
      source, 
      (log: string) => {
        addLog(log);
      },
      () => shouldCancel
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

  const handleGoBack = () => {
    navigate({ to: "/_sidebar/setup/update/changelog", search });
  };

  const handleReset = () => {
    resetUpdate();
  };

  return (
    <Page>
      <SectionTitle title="Apply Update" />
      
      <div className="flex gap-4">
        {!updateResult && (
          <TouchButton
            className="w-max"
            icon="lu:CircleFadingArrowUp"
            onClick={handleStartUpdate}
            disabled={isUpdating}
            isLoading={isUpdating}
          >
            Apply Update
          </TouchButton>
        )}
        
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
        
        {!isUpdating && !updateResult && (
          <TouchButton
            className="w-max"
            icon="lu:ArrowLeft"
            onClick={handleGoBack}
            variant="outline"
          >
            Back to Changelog
          </TouchButton>
        )}

        {updateResult && (
          <>
            <TouchButton
              className="w-max"
              icon="lu:RotateCcw"
              onClick={handleReset}
              variant="outline"
            >
              Start New Update
            </TouchButton>
            <TouchButton
              className="w-max"
              icon="lu:ArrowLeft"
              onClick={handleGoBack}
              variant="outline"
            >
              Back to Changelog
            </TouchButton>
          </>
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
            : `Update failed: ${updateResult.error}`
          }
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
