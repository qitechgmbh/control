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
            onClick={onSwitchToHistorical}
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

          {onExport && (
            <>
              <div className="mx-2 h-8 w-px bg-gray-200"></div>
              <TouchButton
                onClick={onExport}
                variant="outline"
                className="h-auto bg-green-600 px-3 py-3 text-base font-medium text-white hover:bg-green-700"
              >
                Export
              </TouchButton>
            </>
          )}
        </div>
      </div>
    </ControlCard>
  );
}

export function FloatingControlPanel({
  timeWindow,
  isLiveMode,
  onTimeWindowChange,
  onSwitchToLive,
  onSwitchToHistorical,
  onExport,
  timeWindowOptions = DEFAULT_TIME_WINDOW_OPTIONS,
  showFromTimestamp,
  onShowFromChange,
}: ControlProps) {
  const [isExpanded, setIsExpanded] = useState(false);

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
              onClick={onSwitchToHistorical}
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
            {isExpanded && onExport && (
              <div className="h-8 w-px bg-gray-200"></div>
            )}
            {onExport && (
              <TouchButton
                onClick={onExport}
                variant="outline"
                className="h-auto bg-green-600 px-3 py-3 text-base font-medium text-white hover:bg-green-700"
              >
                Export
              </TouchButton>
            )}
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
