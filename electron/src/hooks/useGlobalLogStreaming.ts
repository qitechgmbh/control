import { useEffect } from "react";
import { useLogsStore } from "@/stores/logsStore";
import {
  startLogStream,
  stopLogStream,
  setupLogDataListener,
  isTroubleshootAvailable,
} from "@/helpers/troubleshoot_helpers";

/**
 * Hook to manage global log streaming
 * This should be used once at the app level to start log streaming
 */
export function useGlobalLogStreaming() {
  const { isStreaming, addLogEntry, setStreaming } = useLogsStore();

  useEffect(() => {
    // Only proceed if troubleshoot context is available
    if (!isTroubleshootAvailable()) {
      return;
    }

    if (!isStreaming) {
      startLogStreamHelper();
    }

    // Setup log data listener
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

    // Setup log data listener using helper
    setupLogDataListener(handleLogData);

    return () => {
      // Cleanup on unmount
      if (isStreaming && isTroubleshootAvailable()) {
        stopLogStream();
        setStreaming(false);
      }
    };
  }, [isStreaming, addLogEntry, setStreaming]);

  const startLogStreamHelper = async () => {
    try {
      const result = await startLogStream();
      if (result.success) {
        setStreaming(true);
      } else {
        console.warn("Failed to start log stream:", result.error);
      }
    } catch (error) {
      console.warn("Failed to start log stream:", error);
    }
  };

  return { isStreaming };
}
