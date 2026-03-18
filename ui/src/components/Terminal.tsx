import { cva } from "class-variance-authority";
import React, { useEffect, useRef, useState } from "react";
import { Icon } from "./Icon";
import {
  List,
  CellMeasurer,
  CellMeasurerCache,
  ListRowRenderer,
  AutoSizer,
} from "react-virtualized";

// Create cache (outside the component so it persists across renders)
const cache = new CellMeasurerCache({
  defaultHeight: 20,
  fixedWidth: true,
});

type Props = {
  lines: string[];
  autoScroll?: boolean;
  className?: string;
  title?: string;
  exportPrefix?: string;
};

const terminalStyle = cva([
  "flex flex-col overflow-hidden rounded-md border border-neutral-700 font-mono text-sm",
]);

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
  "40": "bg-gray-900",
  "41": "bg-red-600",
  "42": "bg-green-600",
  "43": "bg-yellow-600",
  "44": "bg-blue-600",
  "45": "bg-purple-600",
  "46": "bg-cyan-600",
  "47": "bg-gray-200",
};

const parseColorCodes = (text: string) => {
  /* eslint-disable-next-line no-control-regex */
  const parts = text.split(/(\x1b\[\d+m)/g);
  let currentClass = "";
  const result: { text: string; className: string }[] = [];
  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];
    if (part.startsWith("\x1b[")) {
      const code = part.slice(2, -1);
      currentClass = colorMap[code] || "";
    } else if (part) {
      result.push({ text: part, className: currentClass });
    }
  }
  return result;
};

const stripColorCodes = (text: string): string =>
  // eslint-disable-next-line no-control-regex
  text.replace(/\x1b\[\d+m/g, "");

export function Terminal({
  lines,
  autoScroll = true,
  className,
  title = "Terminal",
  exportPrefix,
}: Props) {
  const listRef = useRef<List>(null);
  const [copySuccess, setCopySuccess] = useState(false);
  const [exportSuccess, setExportSuccess] = useState(false);

  // Auto-scroll to bottom when new lines arrive
  useEffect(() => {
    if (autoScroll && listRef.current) {
      listRef.current.scrollToRow(lines.length - 1);
    }
  }, [lines, autoScroll]);

  const handleCopy = async () => {
    const plainText = lines.map(stripColorCodes).join("\n");
    try {
      await navigator.clipboard.writeText(plainText);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  const handleExport = () => {
    if (!exportPrefix) return;
    const plainText = lines.map(stripColorCodes).join("\n");
    const timestamp = new Date()
      .toISOString()
      .replace(/[:.]/g, "-")
      .slice(0, -5);
    const filename = `${exportPrefix}_${timestamp}.log`;
    const blob = new Blob([plainText], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    setExportSuccess(true);
    setTimeout(() => setExportSuccess(false), 2000);
  };

  const rowRenderer: ListRowRenderer = ({ index, key, parent, style }) => {
    const colorParts = parseColorCodes(lines[index] || "");

    return (
      <CellMeasurer
        key={key}
        cache={cache}
        parent={parent}
        columnIndex={0}
        rowIndex={index}
      >
        <div
          style={style}
          className="bg-black px-2 py-1 whitespace-pre-wrap text-white"
        >
          {colorParts.length > 0
            ? colorParts.map((part, partIndex) => (
                <span
                  key={partIndex}
                  className={part.className || "text-white"}
                >
                  {part.text || " "}
                </span>
              ))
            : lines[index] || " "}
        </div>
      </CellMeasurer>
    );
  };

  return (
    <div className={terminalStyle({ className })}>
      {/* Header */}
      <div className="flex items-center justify-between border-b border-neutral-700 bg-neutral-800 px-4 py-2">
        <div className="text-xs text-neutral-400">{title}</div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleCopy}
            className="flex items-center text-xs text-neutral-400 transition-colors hover:text-neutral-200"
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

      {/* Virtualized List */}
      <div className="flex-grow overflow-y-auto bg-neutral-900 p-4 text-neutral-300">
        <AutoSizer>
          {({ width, height }) => (
            <List
              ref={listRef}
              width={width}
              height={height}
              deferredMeasurementCache={cache}
              rowHeight={cache.rowHeight}
              rowCount={lines.length}
              rowRenderer={rowRenderer}
              overscanRowCount={5}
            />
          )}
        </AutoSizer>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between bg-neutral-800 px-4 py-1 text-xs text-neutral-400">
        <div>{lines.length} lines</div>
        <div>{autoScroll ? "Auto-scroll enabled" : "Auto-scroll disabled"}</div>
      </div>
    </div>
  );
}
