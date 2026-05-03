import React, { useState } from "react";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/Icon";
import { ControlCard } from "@/control/ControlCard";
import { TimeInput } from "@/components/time/TimeInput";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ControlProps } from "./types";
import { DEFAULT_TIME_WINDOW_OPTIONS } from "./constants";

export function GraphControls({
  timeWindow,
  isLiveMode,
  onTimeWindowChange,
  onSwitchToLive,
  onSwitchToHistorical,
  onExport,
  onAddMarker,
  onManageMarkers,
  timeWindowOptions = DEFAULT_TIME_WINDOW_OPTIONS,
  showFromTimestamp,
  onShowFromChange,
}: ControlProps) {
  const getSelectedTimeWindowLabel = () => {
    // If Show from Time is set, display "Show All" for the time window
    if (showFromTimestamp) {
      return "Show All";
    }
    const option = timeWindowOptions.find((opt) => opt.value === timeWindow);
    return option ? option.label : "1m";
  };

  return (
    <ControlCard className="ml-auto w-fit py-4">
      <div className="flex items-center justify-end">
        <div className="flex items-center gap-3">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <TouchButton
                variant="outline"
                className={`h-auto border-gray-300 bg-white px-3 py-3 text-base text-gray-900 hover:bg-gray-50 ${
                  showFromTimestamp ? "cursor-not-allowed opacity-50" : ""
                }`}
                disabled={!!showFromTimestamp}
              >
                {getSelectedTimeWindowLabel()}
                <Icon name="lu:ChevronDown" className="ml-2 size-4" />
              </TouchButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuLabel className="text-base font-medium">
                Time Window
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              {timeWindowOptions.map((option) => (
                <DropdownMenuItem
                  key={option.value}
                  onClick={() => onTimeWindowChange(option.value)}
                  className={`min-h-[48px] px-4 py-3 text-base ${
                    timeWindow === option.value ? "bg-blue-50" : ""
                  }`}
                >
                  {option.label}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Show From Time Input - only visible in historical mode */}
          {!isLiveMode && onShowFromChange && (
            <TimeInput
              timestamp={showFromTimestamp}
              onTimeChange={onShowFromChange}
              onClear={() => onShowFromChange(null)}
            />
          )}

          <TouchButton
            onClick={() => onSwitchToHistorical("button")}
            variant="outline"
            className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
              !isLiveMode
                ? "bg-black text-white"
                : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
            }`}
          >
            Historical
          </TouchButton>
          <TouchButton
            onClick={onSwitchToLive}
            variant="outline"
            className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
              isLiveMode
                ? "bg-black text-white"
                : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
            }`}
          >
            Live
          </TouchButton>

          {(onExport || onAddMarker || onManageMarkers) && (
            <>
              <div className="mx-2 h-8 w-px bg-gray-200"></div>
              {onAddMarker && (
                <TouchButton
                  onClick={onAddMarker}
                  variant="outline"
                  className="h-auto bg-blue-600 px-3 py-3 text-base font-medium text-white hover:bg-blue-700"
                >
                  Add Marker
                </TouchButton>
              )}
              {onManageMarkers && (
                <TouchButton
                  onClick={onManageMarkers}
                  variant="outline"
                  className="h-auto border-red-200 bg-red-50 px-3 py-3 text-base font-medium text-red-700 hover:bg-red-100"
                >
                  Remove Marker
                </TouchButton>
              )}
              {onExport && <ExportButton onExport={onExport} />}
            </>
          )}
        </div>
      </div>
    </ControlCard>
  );
}

function ExportButton({ onExport }: { onExport: () => void | Promise<void> }) {
  const [isExporting, setIsExporting] = useState(false);

  const handleClick = async () => {
    if (isExporting) return;
    setIsExporting(true);
    // Yield to the event loop so React can re-render and paint the spinner
    // before the heavy synchronous XLSX work blocks the thread.
    await new Promise<void>((resolve) => setTimeout(resolve, 50));
    try {
      await onExport();
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <TouchButton
      onClick={handleClick}
      disabled={isExporting}
      variant="outline"
      className="h-auto bg-green-600 px-3 py-3 text-base font-medium text-white hover:bg-green-700 disabled:opacity-70"
    >
      {isExporting ? (
        <>
          <Icon name="lu:Loader" className="mr-2 size-4 animate-spin" />
          Exporting...
        </>
      ) : (
        "Export"
      )}
    </TouchButton>
  );
}

export function FloatingControlPanel({
  timeWindow,
  isLiveMode,
  onTimeWindowChange,
  onSwitchToLive,
  onSwitchToHistorical,
  onExport,
  onAddMarker,
  onManageMarkers,
  timeWindowOptions = DEFAULT_TIME_WINDOW_OPTIONS,
  showFromTimestamp,
  onShowFromChange,
}: ControlProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const getSelectedTimeWindowLabel = () => {
    // If Show from Time is set, display "Show All" for the time window
    if (showFromTimestamp) {
      return "Show All";
    }
    const option = timeWindowOptions.find((opt) => opt.value === timeWindow);
    return option ? option.label : "1m";
  };

  return (
    <div className="fixed right-10 bottom-6 z-50">
      <ControlCard className="overflow-hidden px-4 py-4 transition-all duration-300 ease-in-out">
        <div
          className={`flex items-center ${isExpanded ? "gap-3" : "justify-center"}`}
        >
          <div
            className={`flex items-center gap-3 transition-all duration-300 ease-in-out ${
              isExpanded
                ? "max-w-none translate-x-0 opacity-100"
                : "w-0 max-w-0 overflow-hidden opacity-0"
            }`}
          >
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <TouchButton
                  variant="outline"
                  className={`h-auto border-gray-300 bg-white px-3 py-3 text-base text-gray-900 hover:bg-gray-50 ${
                    showFromTimestamp ? "cursor-not-allowed opacity-50" : ""
                  }`}
                  disabled={!!showFromTimestamp}
                >
                  {getSelectedTimeWindowLabel()}
                  <Icon name="lu:ChevronDown" className="ml-2 size-4" />
                </TouchButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuLabel className="text-base font-medium">
                  Time Window
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                {timeWindowOptions.map((option) => (
                  <DropdownMenuItem
                    key={option.value}
                    onClick={() => onTimeWindowChange(option.value)}
                    className={`min-h-[48px] px-4 py-3 text-base ${
                      timeWindow === option.value ? "bg-blue-50" : ""
                    }`}
                  >
                    {option.label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Show From Time Input - only visible in historical mode */}
            {!isLiveMode && onShowFromChange && (
              <TimeInput
                timestamp={showFromTimestamp}
                onTimeChange={onShowFromChange}
                onClear={() => onShowFromChange(null)}
              />
            )}

            <TouchButton
              onClick={() => onSwitchToHistorical("button")}
              variant="outline"
              className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
                !isLiveMode
                  ? "bg-black text-white"
                  : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
              }`}
            >
              Historical
            </TouchButton>
            <TouchButton
              onClick={onSwitchToLive}
              variant="outline"
              className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
                isLiveMode
                  ? "bg-black text-white"
                  : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
              }`}
            >
              Live
            </TouchButton>
            {isExpanded && (onExport || onAddMarker || onManageMarkers) && (
              <div className="h-8 w-px bg-gray-200"></div>
            )}
            {isExpanded && onAddMarker && (
              <TouchButton
                onClick={onAddMarker}
                variant="outline"
                className="h-auto bg-blue-600 px-3 py-3 text-base font-medium text-white hover:bg-blue-700"
              >
                Add Marker
              </TouchButton>
            )}
            {isExpanded && onManageMarkers && (
              <TouchButton
                onClick={onManageMarkers}
                variant="outline"
                className="h-auto border-red-200 bg-red-50 px-3 py-3 text-base font-medium text-red-700 hover:bg-red-100"
              >
                Remove Marker
              </TouchButton>
            )}
            {isExpanded && onExport && <ExportButton onExport={onExport} />}
          </div>

          {isExpanded && <div className="h-8 w-px bg-gray-200"></div>}

          <TouchButton
            onClick={() => setIsExpanded(!isExpanded)}
            variant="outline"
            className="h-auto flex-shrink-0 bg-black p-3 text-white hover:bg-gray-100"
            icon={isExpanded ? "lu:ChevronRight" : "lu:ChevronLeft"}
          />
        </div>
      </ControlCard>
    </div>
  );
}
