import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import { useLogsStore } from "@/stores/logsStore";
import { rebootHmi, restartBackend } from "@/helpers/troubleshoot_helpers";
import React, { useState } from "react";
import { toast } from "sonner";

export function TroubleshootPage() {
  const [isRebootLoading, setIsRebootLoading] = useState(false);
  const [isRestartLoading, setIsRestartLoading] = useState(false);
  const { getLogsBySource, clearLogs } = useLogsStore();

  // Perhaps we just need to clear the logs ?
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

      <h2 className="text-lg font-semibold">Backend Service Logs</h2>

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
