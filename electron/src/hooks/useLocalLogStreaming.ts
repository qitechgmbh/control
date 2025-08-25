import { useEffect, useState, useCallback } from "react";
import { useLogsStore } from "@/stores/logsStore";
import {
  startLogStream,
  stopLogStream,
  setupLogDataListener,
  cleanupLogDataListener,
  isTroubleshootAvailable,
} from "@/helpers/troubleshoot_helpers";

/**
 * Hook for local log streaming that can be controlled on-demand
 * This replaces the global log streaming for use on specific pages
 */
export function useLocalLogStreaming() {
  const { isStreaming, addLogEntry, setStreaming } = useLogsStore();
  const [isLocalStreaming, setIsLocalStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Setup log data listener
  const setupListener = useCallback(() => {
    const handleLogData = (logData: string) => {
      // Parse systemd journal log format
      const lines = logData.split("\n").filter((line) => line.trim());

      lines.forEach((line) => {
        if (line.trim()) {
          // Simple log level detection
          let level: "info" | "warn" | "error" | "debug" = "info";
          if (line.includes("ERROR") || line.includes("error")) level = "error";
          else if (line.includes("WARN") || line.includes("warn"))
            level = "warn";
          else if (line.includes("DEBUG") || line.includes("debug"))
            level = "debug";

          addLogEntry({
            source: "qitech-control-server",
            level,
            message: line.trim(),
            raw: line,
          });
        }
      });
    };

    setupLogDataListener(handleLogData);
  }, [addLogEntry]);

  // Start log streaming
  const startStreaming = useCallback(async () => {
    if (!isTroubleshootAvailable()) {
      setError("Troubleshoot context not available");
      return false;
    }

    if (isLocalStreaming || isStreaming) {
      return true; // Already streaming
    }

    try {
      setError(null);
      const result = await startLogStream();
      
      if (result.success) {
        setIsLocalStreaming(true);
        setStreaming(true);
        setupListener();
        return true;
      } else {
        setError(result.error || "Failed to start log stream");
        return false;
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.warn("Failed to start log stream:", err);
      return false;
    }
  }, [isLocalStreaming, isStreaming, setStreaming, setupListener]);

  // Stop log streaming
  const stopStreaming = useCallback(async () => {
    if (!isLocalStreaming && !isStreaming) {
      return true; // Already stopped
    }

    try {
      setError(null);
      
      if (isTroubleshootAvailable()) {
        const result = await stopLogStream();
        if (!result.success) {
          setError(result.error || "Failed to stop log stream");
        }
      }
      
      setIsLocalStreaming(false);
      setStreaming(false);
      cleanupLogDataListener();
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.warn("Failed to stop log stream:", err);
      return false;
    }
  }, [isLocalStreaming, isStreaming, setStreaming]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (isLocalStreaming && isTroubleshootAvailable()) {
        stopStreaming();
      }
    };
  }, [isLocalStreaming, stopStreaming]);

  return {
    isStreaming: isLocalStreaming || isStreaming,
    startStreaming,
    stopStreaming,
    error,
  };
}