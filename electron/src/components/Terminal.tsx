import { cva } from "class-variance-authority";
import React, { useEffect, useRef, useState, useMemo, useCallback } from "react";
import { Icon } from "./Icon";

type Props = {
  lines: string[];
  autoScroll?: boolean; // Optional prop to control if terminal should auto-scroll
  className?: string; // Optional prop to control terminal height
  title?: string;
  exportPrefix?: string; // Optional prefix for exported log files
  maxLines?: number; // Maximum number of lines to display for performance
};

const terminalStyle = cva([
  "flex flex-col overflow-hidden rounded-md border border-neutral-700 font-mono text-sm",
]);

// Color mapping for ANSI color codes
const colorMap: Record<string, string> = {
  "30": "text-gray-900",
  "31": "text-red-600",
  "32": "text-green-600",
  "33": "text-yellow-600",
  "34": "text-blue-600",
  "35": "text-purple-600",
  "36": "text-cyan-600",
  "37": "text-gray-200",
  "90": "text-gray-500",
  "91": "text-red-400",
  "92": "text-green-400",
  "93": "text-yellow-400",
  "94": "text-blue-400",
  "95": "text-purple-400",
  "96": "text-cyan-400",
  "97": "text-white",
  // Background colors
  "40": "bg-gray-900",
  "41": "bg-red-600",
  "42": "bg-green-600",
  "43": "bg-yellow-600",
  "44": "bg-blue-600",
  "45": "bg-purple-600",
  "46": "bg-cyan-600",
  "47": "bg-gray-200",
};

// Parse ANSI color codes in text - memoized for performance
const parseColorCodes = (text: string) => {
  // Split by ANSI escape sequences
  // eslint-disable-next-line no-control-regex
  const parts = text.split(/(\x1b\[\d+m)/g);

  let currentClass = "";
  const result: { text: string; className: string }[] = [];

  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];

    if (part.startsWith("\x1b[")) {
      // This is a color code
      const code = part.slice(2, -1); // Extract the number from \x1b[XXm
      currentClass = colorMap[code] || "";
    } else if (part) {
      // This is text content
      result.push({ text: part, className: currentClass });
    }
  }

  return result;
};

// Function to strip ANSI color codes for plain text copy
const stripColorCodes = (text: string): string => {
  // eslint-disable-next-line no-control-regex
  return text.replace(/\x1b\[\d+m/g, "");
};

// Memoized line component for better performance
const TerminalLine = React.memo(({ line, index }: { line: string; index: number }) => {
  const colorParts = useMemo(() => parseColorCodes(line), [line]);

  return (
    <div key={index} className="whitespace-pre-wrap">
      {colorParts.length > 0
        ? colorParts.map((part, partIndex) => (
            <span key={partIndex} className={part.className}>
              {part.text || " "}
            </span>
          ))
        : line || " "}
    </div>
  );
});

TerminalLine.displayName = "TerminalLine";

export function Terminal({
  lines,
  autoScroll = true,
  className,
  title = "Terminal",
  exportPrefix,
  maxLines = 5000, // Default maximum lines for performance
}: Props) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const [isScrolledToBottom, setIsScrolledToBottom] = useState(true);
  const [copySuccess, setCopySuccess] = useState(false);
  const [exportSuccess, setExportSuccess] = useState(false);

  // Limit lines for performance - show only the most recent lines
  const displayLines = useMemo(() => {
    if (lines.length <= maxLines) return lines;
    return lines.slice(-maxLines);
  }, [lines, maxLines]);

  // Throttled scroll to bottom function to improve performance
  const scrollToBottomThrottled = useCallback(() => {
    const terminal = terminalRef.current;
    if (terminal && autoScroll && isScrolledToBottom) {
      // Use requestAnimationFrame for smooth scrolling
      requestAnimationFrame(() => {
        terminal.scrollTop = terminal.scrollHeight;
      });
    }
  }, [autoScroll, isScrolledToBottom]);

  // Handle scrolling with throttling
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      scrollToBottomThrottled();
    }, 100);

    return () => clearTimeout(timeoutId);
  }, [displayLines, scrollToBottomThrottled]);

  // Handle scroll events to detect if user is at bottom
  const handleScroll = useCallback(() => {
    const terminal = terminalRef.current;
    if (!terminal) return;

    const isAtBottom =
      Math.abs(
        terminal.scrollHeight - terminal.clientHeight - terminal.scrollTop,
      ) < 10; // Small threshold to account for rounding errors

    setIsScrolledToBottom(isAtBottom);
  }, []);

  // Handle copy to clipboard
  const handleCopy = async () => {
    // Strip ANSI color codes and join lines
    const plainText = lines.map((line) => stripColorCodes(line)).join("\n");

    try {
      await navigator.clipboard.writeText(plainText);
      setCopySuccess(true);

      // Reset copy success message after 2 seconds
      setTimeout(() => {
        setCopySuccess(false);
      }, 2000);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  // Handle export to file
  const handleExport = () => {
    if (!exportPrefix) return;

    // Strip ANSI color codes and join lines
    const plainText = lines.map((line) => stripColorCodes(line)).join("\n");

    // Create timestamp for filename
    const timestamp = new Date()
      .toISOString()
      .replace(/[:.]/g, "-")
      .slice(0, -5);
    const filename = `${exportPrefix}_${timestamp}.log`;

    // Create blob and download
    const blob = new Blob([plainText], { type: "text/plain" });
    const url = URL.createObjectURL(blob);

    // Create temporary link element and trigger download
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    // Clean up
    URL.revokeObjectURL(url);

    // Show success feedback
    setExportSuccess(true);
    setTimeout(() => {
      setExportSuccess(false);
    }, 2000);
  };

  return (
    <div className={terminalStyle({ className })}>
      {/* Terminal header */}
      <div className="flex items-center justify-between border-b border-neutral-700 bg-neutral-800 px-4 py-2">
        <div className="text-xs text-neutral-400">{title}</div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleCopy}
            className="flex items-center text-xs text-neutral-400 transition-colors hover:text-neutral-200"
            title="Copy to clipboard"
          >
            {copySuccess ? (
              <>
                <Icon name="lu:ClipboardCheck" className="mr-2 size-4" />
                Copied!
              </>
            ) : (
              <>
                <Icon name="lu:Clipboard" className="mr-2 size-4" />
                Copy
              </>
            )}
          </button>
          {exportPrefix && (
            <button
              onClick={handleExport}
              className="flex items-center text-xs text-neutral-400 transition-colors hover:text-neutral-200"
              title="Export logs to file"
            >
              {exportSuccess ? (
                <>
                  <Icon name="lu:Check" className="mr-2 size-4" />
                  Exported!
                </>
              ) : (
                <>
                  <Icon name="lu:Save" className="mr-2 size-4" />
                  Export
                </>
              )}
            </button>
          )}
        </div>
      </div>

      {/* Terminal content with custom scrollbar styling */}
      <div
        ref={terminalRef}
        onScroll={handleScroll}
        className={`scrollbar-thin scrollbar-thumb-neutral-600 scrollbar-track-transparent flex-grow overflow-y-auto bg-neutral-900 p-4 text-neutral-300`}
        style={{
          scrollbarWidth: "thin",
          scrollbarColor: "rgb(82 82 91) transparent",
        }}
      >
        <style>{`
          /* For Webkit browsers (Chrome, Safari) */
          .scrollbar-thin::-webkit-scrollbar {
            width: 6px;
          }

          .scrollbar-thin::-webkit-scrollbar-track {
            background: transparent;
          }

          .scrollbar-thin::-webkit-scrollbar-thumb {
            background-color: rgb(82 82 91);
            border-radius: 3px;
          }

          /* For Firefox */
          .scrollbar-thin {
            scrollbar-width: thin;
            scrollbar-color: rgb(82 82 91) transparent;
          }
        `}</style>
        {displayLines.map((line, index) => (
          <TerminalLine key={`${index}-${line.substring(0, 50)}`} line={line} index={index} />
        ))}
      </div>

      {/* Status bar */}
      <div className="flex items-center justify-between bg-neutral-800 px-4 py-1 text-xs text-neutral-400">
        <div>
          {displayLines.length} / {lines.length} lines
          {lines.length > maxLines && (
            <span className="ml-2 text-yellow-400">
              (showing last {maxLines} for performance)
            </span>
          )}
        </div>
        <div>
          {isScrolledToBottom ? "At bottom" : "Scrolled up"} |
          {autoScroll ? " Auto-scroll enabled" : " Auto-scroll disabled"}
        </div>
      </div>
    </div>
  );
}
