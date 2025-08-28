import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import { useLogsStore } from "@/stores/logsStore";
import { rebootHmi, restartBackend } from "@/helpers/troubleshoot_helpers";
import { useLocalLogStreaming } from "@/hooks/useLocalLogStreaming";
import React, { useEffect, useState } from "react";
import { toast } from "sonner";

export function TroubleshootPage() {
  const [isRebootLoading, setIsRebootLoading] = useState(false);
  const [isRestartLoading, setIsRestartLoading] = useState(false);

  const { getLogsBySource } = useLogsStore();
  const { isStreaming, startStreaming, stopStreaming, error } = useLocalLogStreaming();

  // Start log streaming when component mounts, stop when unmounts
  useEffect(() => {
    startStreaming();
    
    // Cleanup function will stop streaming when component unmounts
    return () => {
      stopStreaming();
    };
  }, [startStreaming, stopStreaming]);

  // Show error toast if log streaming fails
  useEffect(() => {
    if (error) {
      toast.error(`Log streaming error: ${error}`);
    }
  }, [error]);

  // Get backend logs for display
  const backendLogs = getLogsBySource("qitech-control-server");
  const logLines = backendLogs.map((log) => log.raw);

  const handleRebootHmi = async () => {
    setIsRebootLoading(true);
    try {
      const result = await rebootHmi();
      if (result.success) {
        toast.success("HMI Panel reboot initiated");
      } else {
        toast.error(`Failed to reboot HMI: ${result.error}`);
      }
    } catch (error) {
      toast.error(`Failed to reboot HMI: ${error}`);
    } finally {
      setIsRebootLoading(false);
    }
  };

  const handleRestartBackend = async () => {
    setIsRestartLoading(true);
    try {
      const result = await restartBackend();
      if (result.success) {
        toast.success("Backend service restart initiated");
      } else {
        toast.error(`Failed to restart backend: ${result.error}`);
      }
    } catch (error) {
      toast.error(`Failed to restart backend: ${error}`);
    } finally {
      setIsRestartLoading(false);
    }
  };

  return (
    <Page>
      <SectionTitle title="System Troubleshoot" />

      <div className="flex gap-4">
        <TouchButton
          variant="destructive"
          icon="lu:Power"
          isLoading={isRebootLoading}
          onClick={handleRebootHmi}
          className="w-max"
        >
          Reboot HMI Panel
        </TouchButton>

        <TouchButton
          variant="outline"
          icon="lu:RotateCcw"
          isLoading={isRestartLoading}
          onClick={handleRestartBackend}
          className="w-max"
        >
          Restart Backend Process
        </TouchButton>
      </div>

      <Alert title="Troubleshoot Actions Info" variant="warning">
        These actions will temporarily interrupt system operations. The HMI
        reboot will restart the entire panel, while the backend restart will
        only restart the control service. Use with caution during production.
      </Alert>

      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Backend Service Logs</h2>
        <div className="flex items-center gap-2 text-sm text-neutral-500">
          <div className={`h-2 w-2 rounded-full ${isStreaming ? 'bg-green-500' : 'bg-red-500'}`} />
          {isStreaming ? 'Live streaming' : 'Not streaming'}
        </div>
      </div>

      <Terminal
        lines={logLines}
        autoScroll={true}
        className="h-160"
        title="qitech-control-server"
        exportPrefix="qitech_control_server_journald"
      />
    </Page>
  );
}
