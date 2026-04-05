import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import { UpdateProgressBar } from "@/components/UpdateProgressBar";
import { Icon } from "@/components/Icon";
import { useSearch } from "@tanstack/react-router";
import React from "react";
import { useUpdate } from "@/lib/update/useUpdate";

export function UpdateExecutePage() {
  const updateInfo = useSearch({
    from: "/_sidebar/setup/update/execute",
  });

  const {
    isUpdating,
    terminalLines,
    currentUpdateInfo,
    steps,
    overallProgress,
    start,
    cancel,
  } = useUpdate();

  return (
    <Page>
      <SectionTitle title="Apply Update" />

      <div className="flex flex-row items-center gap-4">
        <TouchButton
          className="w-max"
          icon="lu:CircleFadingArrowUp"
          onClick={() => start(updateInfo)}
          disabled={isUpdating}
          isLoading={isUpdating}
        >
          Apply Update
        </TouchButton>
        {isUpdating && (
          <TouchButton
            className="w-max"
            icon="lu:X"
            onClick={cancel}
            variant="destructive"
          >
            Cancel Update
          </TouchButton>
        )}
        <div className="ml-auto flex flex-col gap-2">
          <div role="alert" className="flex w-max gap-2 rounded-xl border border-amber-400 bg-amber-500 px-4 py-2.5 text-white shadow-xl backdrop-blur-sm transition-all duration-300">
            <Icon name="lu:Info" className="h-5 w-5 text-blue-100" />
            <span className="text-base leading-snug text-blue-50">
              Make sure machine is <strong>not</strong> in use when updating!
            </span>
          </div>
          {isUpdating && (
            <div role="alert" className="flex w-max gap-2 rounded-xl border border-blue-400 bg-blue-500 px-4 py-2.5 text-white shadow-xl backdrop-blur-sm transition-all duration-300">
              <Icon name="lu:Info" className="h-5 w-5 text-blue-100" />
              <span className="text-base leading-snug text-blue-50">
                Updates typically take approximately <strong>5 minutes</strong>
              </span>
            </div>
          )}
        </div>
      </div>
      {currentUpdateInfo && (
        <Alert title="Update Information" variant="info">
          <div className="space-y-3">
            <div className="space-y-1 text-sm">
              {currentUpdateInfo.tag && (
                <div>
                  <span className="font-medium">Tag:</span>{" "}
                  <span className="font-mono">{currentUpdateInfo.tag}</span>
                </div>
              )}
              {currentUpdateInfo.branch && (
                <div>
                  <span className="font-medium">Branch:</span>{" "}
                  <span className="font-mono">{currentUpdateInfo.branch}</span>
                </div>
              )}
              {currentUpdateInfo.commit && (
                <div>
                  <span className="font-medium">Commit:</span>{" "}
                  <span className="font-mono">
                    {currentUpdateInfo.commit.substring(0, 8)}
                  </span>
                </div>
              )}
            </div>
            <div className="text-sm">
              Please stay connected to the internet and leave the power on. The
              machine will reboot after the update completes.
            </div>
          </div>
        </Alert>
      )}

      {/* Progress Bar */}
      {isUpdating && (
        <UpdateProgressBar
          steps={steps}
          overallProgress={overallProgress}
          className="mb-4"
        />
      )}

      <Terminal
        lines={terminalLines}
        className="h-160"
        exportPrefix="qitech_control_server_update"
      />
    </Page>
  );
}
