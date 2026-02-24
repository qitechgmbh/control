import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { TouchButton } from "@/components/touch/TouchButton";
import {
  rebootHmi,
  restartBackend,
  exportLogs,
} from "@/helpers/troubleshoot_helpers";
import React, { useState } from "react";
import { toast } from "sonner";

export function TroubleshootPage() {
  const [isRebootLoading, setIsRebootLoading] = useState(false);
  const [isRestartLoading, setIsRestartLoading] = useState(false);
  const [isExportLoading, setIsExportLoading] = useState(false);

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

  const handleExportLogs = async () => {
    setIsExportLoading(true);
    try {
      const result = await exportLogs();
      if (result.success) {
        toast.success("Export Logs initiated");
      } else {
        toast.error(`Failed to export Logs: ${result.error}`);
      }
    } catch (error) {
      toast.error(`Failed to export Logs: ${error}`);
    } finally {
      setIsExportLoading(false);
    }
  };

  return (
    <Page>
      <SectionTitle title="System Troubleshoot" />

      <Alert title="Troubleshoot Actions Info" variant="warning">
        These actions will temporarily interrupt system operations. The HMI
        reboot will restart the entire panel, while the backend restart will
        only restart the control service. Use with caution during production.
      </Alert>

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

        <TouchButton
          variant="outline"
          icon="lu:Power"
          isLoading={isExportLoading}
          onClick={exportLogs}
          className="w-max"
        >
          Export Backend Service Logs
        </TouchButton>
      </div>
    </Page>
  );
}
