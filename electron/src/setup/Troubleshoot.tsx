import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { TouchButton } from "@/components/touch/TouchButton";
import { ExportResultDialog } from "@/components/ExportResultDialog";
import { mainNamespaceStore } from "@/client/mainNamespace";
import {
  rebootHmi,
  restartBackend,
  restartBackendIntoPreop,
  exportLogs,
} from "@/helpers/troubleshoot_helpers";
import { useExportDialog } from "@/hooks/useExportDialog";
import React, { useState } from "react";
import { toast } from "sonner";

export function TroubleshootPage() {
  const [isRebootLoading, setIsRebootLoading] = useState(false);
  const [isRestartLoading, setIsRestartLoading] = useState(false);
  const [isRestartPreopLoading, setIsRestartPreopLoading] = useState(false);
  const [isExportLoading, setIsExportLoading] = useState(false);
  const { notifyResult, dialogProps } = useExportDialog();

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

  const handleRestartBackendIntoPreop = async () => {
    setIsRestartPreopLoading(true);
    try {
      const result = await restartBackendIntoPreop();
      if (result.success) {
        mainNamespaceStore.setState({ isIntentionalPreop: true });
        toast.success("Backend restarted into Preop mode");
      } else {
        toast.error(`Failed to restart into Preop: ${result.error}`);
      }
    } catch (error) {
      toast.error(`Failed to restart into Preop: ${error}`);
    } finally {
      setIsRestartPreopLoading(false);
    }
  };

  const handleExportLogs = async () => {
    setIsExportLoading(true);
    try {
      const result = await exportLogs();
      notifyResult(result);
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
          icon="lu:RotateCcw"
          isLoading={isRestartPreopLoading}
          onClick={handleRestartBackendIntoPreop}
          className="w-max"
        >
          Restart Backend Process Into Preop
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

      <ExportResultDialog {...dialogProps} />
    </Page>
  );
}
