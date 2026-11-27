import React, { useState } from "react";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/icons/Icon";
import { ControlCard } from "@/control/ControlCard";
import { TimeInput } from "@/components/inputs/TimeInput";
import { ExportResultDialog } from "@/components/feedback/ExportResultDialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { TimeWindow, TimeWindowOption } from "../types";
import { DEFAULT_TIME_WINDOW_OPTIONS } from "../constants";
import { type GraphSync, viewEndTimestamp } from "../useGraphSync";
import { MarkerControls } from "../markers/MarkerControls";
import { ExportButton } from "./ExportDialog";

type GraphControlPanelProps = {
  syncHook: GraphSync;
  timeWindowOptions?: TimeWindowOption[];
};

/**
 * Floating panel at the bottom right of a graph page: time window, live/historical,
 * "show from", markers and export. Collapses to a single button.
 */
export function GraphControlPanel({
  syncHook,
  timeWindowOptions = DEFAULT_TIME_WINDOW_OPTIONS,
}: GraphControlPanelProps) {
  const [isExpanded, setIsExpanded] = useState(true);
  const { view } = syncHook;

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
            <TimeWindowMenu
              value={view.timeWindow}
              options={timeWindowOptions}
              // "Show from" replaces the window.
              disabled={view.showFromTimestamp !== null}
              onChange={syncHook.setTimeWindow}
            />

            {!view.isLiveMode && (
              <TimeInput
                timestamp={view.showFromTimestamp}
                onTimeChange={syncHook.setShowFrom}
                onClear={() => syncHook.setShowFrom(null)}
              />
            )}

            <ModeButton
              active={!view.isLiveMode}
              onClick={syncHook.switchToHistorical}
            >
              Historical
            </ModeButton>
            <ModeButton
              active={view.isLiveMode}
              onClick={syncHook.switchToLive}
            >
              Live
            </ModeButton>

            {isExpanded && (
              <>
                <div className="h-8 w-px bg-gray-200" />
                <MarkerControls defaultTimestamp={viewEndTimestamp(view)} />
                <ExportButton syncHook={syncHook} />
              </>
            )}
          </div>

          {isExpanded && <div className="h-8 w-px bg-gray-200" />}

          <TouchButton
            onClick={() => setIsExpanded(!isExpanded)}
            variant="outline"
            className="h-auto flex-shrink-0 bg-black p-3 text-white hover:bg-gray-100"
            icon={isExpanded ? "lu:ChevronRight" : "lu:ChevronLeft"}
          />
        </div>
      </ControlCard>
      <ExportResultDialog {...syncHook.exportDialogProps} />
    </div>
  );
}

function TimeWindowMenu({
  value,
  options,
  disabled,
  onChange,
}: {
  value: TimeWindow;
  options: TimeWindowOption[];
  disabled: boolean;
  onChange: (value: TimeWindow) => void;
}) {
  // Disabled while "show from" replaces the window.
  const label = disabled
    ? "Show From"
    : (options.find((opt) => opt.value === value)?.label ?? "1m");

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <TouchButton
          variant="outline"
          className={`h-auto border-gray-300 bg-white px-3 py-3 text-base text-gray-900 hover:bg-gray-50 ${
            disabled ? "cursor-not-allowed opacity-50" : ""
          }`}
          disabled={disabled}
        >
          {label}
          <Icon name="lu:ChevronDown" className="ml-2 size-4" />
        </TouchButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel className="text-base font-medium">
          Time Window
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        {options.map((option) => (
          <DropdownMenuItem
            key={option.value}
            onClick={() => onChange(option.value)}
            className={`min-h-[48px] px-4 py-3 text-base ${
              value === option.value ? "bg-blue-50" : ""
            }`}
          >
            {option.label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ModeButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <TouchButton
      onClick={onClick}
      variant="outline"
      className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
        active
          ? "bg-black text-white"
          : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
      }`}
    >
      {children}
    </TouchButton>
  );
}
