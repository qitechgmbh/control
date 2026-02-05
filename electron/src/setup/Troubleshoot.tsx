import { Alert } from "@/components/Alert";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { Terminal } from "@/components/Terminal";
import { TouchButton } from "@/components/touch/TouchButton";
import React, { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

type LogLineCache = {
    numLines: number;
    getLine: (index: number) => string | undefined;
    loadLines: (start: number, end: number) => void;
}

function useLogLineCache(): LogLineCache {
  const [numLines, setNumLines] = useState(200);

  const [cachedLines, setCashedLines] = useState<string[]>([]);
  const [firstCachedLine, setFirstCachedLine] = useState(0);

  useEffect(() => {
    loadLinesIntoCache(numLines, numLines - 100);
    return window.troubleshoot.subscribeLogLine(setNumLines);
  }, []);

  const loadLinesIntoCache = async (start: number, end: number) => {
      const lines = await window.troubleshoot.getLogLines(start - 100, end - start + 100);
      setFirstCachedLine(start);
      setCashedLines(lines);
  }

  const getLine = useCallback((index: number) => {
      console.log(index, firstCachedLine, cachedLines.length);
      if (index < firstCachedLine || firstCachedLine + cachedLines.length <= index) {
          return undefined;
      }

      return cachedLines[index - firstCachedLine];
  }, [firstCachedLine, cachedLines]);


  return { numLines, getLine, loadLines: loadLinesIntoCache };
}

export function TroubleshootPage() {
  const [isRebootLoading, setIsRebootLoading] = useState(false);
  const [isRestartLoading, setIsRestartLoading] = useState(false);
  const { numLines, getLine, loadLines } = useLogLineCache();

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
        numLines={numLines}
        getLine={getLine}
        loadLines={loadLines}
        autoScroll={true}
        className="h-160"
        title="qitech-control-server"
        exportPrefix="qitech_control_server_journald"
      />
    </Page>
  );
}
