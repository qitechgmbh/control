import { LogEntry } from "@/stores/logsStore";

/**
 * Filters and manages log comments for export
 */
export class CommentManager {
  static filterRelevant(
    logs: LogEntry[],
    startTime: number,
    endTime: number,
  ): LogEntry[] {
    return logs.filter(
      (log) =>
        log.timestamp.getTime() >= startTime &&
        log.timestamp.getTime() <= endTime &&
        log.level === "info" &&
        log.message.toLowerCase().includes("comment"),
    );
  }

  static findAtTimestamp(
    comments: LogEntry[],
    timestamp: number,
    tolerance: number = 1000,
  ): LogEntry | undefined {
    return comments.find(
      (log) => Math.abs(log.timestamp.getTime() - timestamp) < tolerance,
    );
  }
}
