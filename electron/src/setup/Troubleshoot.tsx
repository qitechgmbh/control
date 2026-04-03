import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { TouchButton } from "@/components/touch/TouchButton";
import React, { useState } from "react";
import { toast } from "sonner";

export function TroubleshootPage() {
  const [isRebootLoading, setIsRebootLoading] = useState(false);
  const [isRestartLoading, setIsRestartLoading] = useState(false);
  const [isExportLoading, setIsExportLoading] = useState(false);

  const handleRebootHmi = async () => {
    setIsRebootLoading(true);
    try {
      await window.troubleshoot.rebootHmi();
    } catch (error) {
      toast.error(`Failed to reboot HMI: ${error}`);
    } finally {
      setIsRebootLoading(false);
    }
  };

  const handleRestartBackend = async () => {
    setIsRestartLoading(true);
    try {
      await window.troubleshoot.restartBackend();
    } catch (error) {
      toast.error(`Failed to restart backend: ${error}`);
    } finally {
      setIsRestartLoading(false);
    }
  };

  const handleExportLogs = async () => {
    setIsExportLoading(true);
    try {
      await window.troubleshoot.exportLogs();
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
          icon="lu:FileDown"
          isLoading={isExportLoading}
          onClick={handleExportLogs}
          className="w-max"
        >
          Export Backend Service Logs
        </TouchButton>
      </div>
    </Page>
  );
}
