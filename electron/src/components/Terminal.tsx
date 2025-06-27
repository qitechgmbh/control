import { cva } from "class-variance-authority";
import React, { 
  useCallback, 
  useEffect, 
  useMemo, 
  useRef, 
  useState 
} from "react";
import { Icon } from "./Icon";

type Props = {
  lines: string[];
  autoScroll?: boolean; // Optional prop to control if terminal should auto-scroll
  className?: string; // Optional prop to control terminal height
  title?: string;
  exportPrefix?: string; // Optional prefix for exported log files
  maxLines?: number; // Maximum number of lines to render (for performance)
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

// Parse ANSI color codes in text - Memoized for performance
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

// Cache for parsed color codes to avoid re-parsing
const parseCache = new Map<string, { text: string; className: string }[]>();

// Optimized parse function with caching
const parseColorCodesCached = (text: string) => {
  if (parseCache.has(text)) {
    return parseCache.get(text)!;
  }
  
  const result = parseColorCodes(text);
  
  // Limit cache size to prevent memory leaks
  if (parseCache.size > 1000) {
    const firstKey = parseCache.keys().next().value;
    if (firstKey) {
      parseCache.delete(firstKey);
    }
  }
  
  parseCache.set(text, result);
  return result;
};

// Function to strip ANSI color codes for plain text copy
const stripColorCodes = (text: string): string => {
  // eslint-disable-next-line no-control-regex
  return text.replace(/\x1b\[\d+m/g, "");
};

// Memoized line component for better performance
const TerminalLine = React.memo(({ 
  line, 
  index 
}: { 
  line: string; 
  index: number;
}) => {
  const colorParts = useMemo(() => parseColorCodesCached(line), [line]);

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

TerminalLine.displayName = 'TerminalLine';

export function Terminal({
  lines,
  autoScroll = true,
  className,
  title = "Terminal",
  exportPrefix,
  maxLines = 1000,
}: Props) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const [isScrolledToBottom, setIsScrolledToBottom] = useState(true);
  const [copySuccess, setCopySuccess] = useState(false);
  const [exportSuccess, setExportSuccess] = useState(false);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(0);

  // Throttle scroll updates for performance
  const scrollThrottle = useRef<NodeJS.Timeout | null>(null);

  // Limit lines for performance - keep most recent lines
  const displayLines = useMemo(() => {
    if (lines.length <= maxLines) return lines;
    return lines.slice(-maxLines);
  }, [lines, maxLines]);

  // Calculate visible range for virtualization
  const lineHeight = 20; // Approximate line height in pixels
  
  const visibleRange = useMemo(() => {
    const visibleStart = Math.max(0, Math.floor(scrollTop / lineHeight) - 10);
    const visibleEnd = Math.min(
      displayLines.length,
      visibleStart + Math.ceil(containerHeight / lineHeight) + 20
    );

    return { start: visibleStart, end: visibleEnd };
  }, [displayLines.length, containerHeight, scrollTop]);

  // Memoize visible lines to prevent unnecessary re-parsing
  const visibleLines = useMemo(() => {
    return displayLines.slice(visibleRange.start, visibleRange.end);
  }, [displayLines, visibleRange]);

  // Update container dimensions
  const updateDimensions = useCallback(() => {
    const terminal = terminalRef.current;
    if (terminal) {
      setContainerHeight(terminal.clientHeight);
    }
  }, []);

  useEffect(() => {
    updateDimensions();
    const resizeObserver = new ResizeObserver(updateDimensions);
    if (terminalRef.current) {
      resizeObserver.observe(terminalRef.current);
    }
    return () => {
      resizeObserver.disconnect();
      // Clean up scroll throttle on unmount
      if (scrollThrottle.current) {
        clearTimeout(scrollThrottle.current);
      }
    };
  }, [updateDimensions]);

  // Handle scrolling - Optimized to use display lines
  useEffect(() => {
    const terminal = terminalRef.current;
    if (!terminal) return;

    // If auto-scroll is enabled and user was at bottom, scroll to bottom when lines change
    if (autoScroll && isScrolledToBottom) {
      terminal.scrollTop = terminal.scrollHeight;
    }
  }, [displayLines, autoScroll, isScrolledToBottom]);

  // Handle scroll events to detect if user is at bottom - Throttled for performance
  const handleScroll = useCallback(() => {
    const terminal = terminalRef.current;
    if (!terminal) return;

    // Clear existing throttle
    if (scrollThrottle.current) {
      clearTimeout(scrollThrottle.current);
    }

    scrollThrottle.current = setTimeout(() => {
      const currentScrollTop = terminal.scrollTop;
      setScrollTop(currentScrollTop);

      const isAtBottom =
        Math.abs(
          terminal.scrollHeight - terminal.clientHeight - currentScrollTop,
        ) < 10; // Small threshold to account for rounding errors

      setIsScrolledToBottom(isAtBottom);
    }, 16); // ~60fps throttling
  }, []);

  // Handle copy to clipboard - Optimized to use display lines
  const handleCopy = useCallback(async () => {
    // Strip ANSI color codes and join lines - use original lines, not just displayed
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
  }, [lines]);

  // Handle export to file - Optimized to use all lines
  const handleExport = useCallback(() => {
    if (!exportPrefix) return;

    // Strip ANSI color codes and join lines - use original lines, not just displayed
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
  }, [exportPrefix, lines]);

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
          Add commentMore actions
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
        {/* Virtualized rendering */}
        <div style={{ height: displayLines.length * lineHeight }}>
          <div 
            style={{ 
              transform: `translateY(${visibleRange.start * lineHeight}px)`,
              position: 'relative'
            }}
          >
            {visibleLines.map((line, index) => (
              <TerminalLine 
                key={visibleRange.start + index}
                line={line}
                index={visibleRange.start + index}
              />
            ))}
          </div>
        </div>
      </div>

      {/* Status bar */}
      <div className="flex items-center justify-between bg-neutral-800 px-4 py-1 text-xs text-neutral-400">
        <div>
          {lines.length} lines 
          {lines.length !== displayLines.length && (
            <span className="text-yellow-400">
              {" "}(showing last {displayLines.length})
            </span>
          )}
          <span className="text-blue-400">
            {" "}• virtualized
          </span>
        </div>
        <div>
          {isScrolledToBottom ? "At bottom" : "Scrolled up"} |
          {autoScroll ? " Auto-scroll enabled" : " Auto-scroll disabled"}
        </div>
      </div>
    </div>
  );
}
