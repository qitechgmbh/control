import { useLogsStore } from "@/stores/logsStore";

/**
 * Utility functions for working with logs
 */
export const useLogsUtils = () => {
  const { entries, getLogsBySource } = useLogsStore();

  const getLogsByLevel = (level: "info" | "warn" | "error" | "debug") => {
    return entries.filter((entry) => entry.level === level);
  };

  const getRecentLogs = (count: number = 100) => {
    return entries.slice(-count);
  };

  const getLogsByTimeRange = (startTime: Date, endTime: Date) => {
    return entries.filter(
      (entry) => entry.timestamp >= startTime && entry.timestamp <= endTime,
    );
  };

  const searchLogs = (searchTerm: string) => {
    const term = searchTerm.toLowerCase();
    return entries.filter(
      (entry) =>
        entry.message.toLowerCase().includes(term) ||
        entry.raw.toLowerCase().includes(term),
    );
  };

  const getErrorLogs = () => getLogsByLevel("error");
  const getWarningLogs = () => getLogsByLevel("warn");

  return {
    getLogsBySource,
    getLogsByLevel,
    getRecentLogs,
    getLogsByTimeRange,
    searchLogs,
    getErrorLogs,
    getWarningLogs,
    totalLogs: entries.length,
    errorCount: getErrorLogs().length,
    warningCount: getWarningLogs().length,
  };
};
