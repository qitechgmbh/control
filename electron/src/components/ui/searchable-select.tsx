import * as React from "react";
import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverTrigger,
  PopoverContent,
} from "@/components/ui/popover";
import { Icon } from "@/components/Icon";

export type SearchableSelectGroup<T> = {
  label: string;
  options: T[];
};

type SearchableSelectProps<T> = {
  /** Groups of options to display. Each group gets a header label. */
  groups: SearchableSelectGroup<T>[];
  /** Current selected value (the option value, not the label). */
  value: string;
  /** Called when an option is selected. Receives the option value as string. */
  onChange: (value: string) => void;
  /** Placeholder shown when nothing is selected. */
  placeholder?: string;
  /** Search input placeholder. */
  searchPlaceholder?: string;
  /** Extract the display label from an option. */
  getOptionLabel: (option: T) => string;
  /** Extract the string value from an option. */
  getOptionValue: (option: T) => string;
  /** Message shown when search yields no results. */
  emptyMessage?: string;
  /** Optional class for the trigger button. */
  className?: string;
  /** If true, the select is disabled. */
  disabled?: boolean;
};

export function SearchableSelect<T>({
  groups,
  value,
  onChange,
  placeholder = "Select...",
  searchPlaceholder = "Search...",
  getOptionLabel,
  getOptionValue,
  emptyMessage = "No results found.",
  className,
  disabled = false,
}: SearchableSelectProps<T>) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Find the selected option label for display
  const selectedLabel = useMemo(() => {
    if (!value) return "";
    for (const group of groups) {
      const found = group.options.find((o) => getOptionValue(o) === value);
      if (found) return getOptionLabel(found);
    }
    return "";
  }, [value, groups, getOptionLabel, getOptionValue]);

  // Flatten filtered options for keyboard navigation
  const filteredOptions = useMemo(() => {
    if (!search.trim()) {
      return groups.flatMap((g) => g.options);
    }
    const q = search.toLowerCase();
    return groups.flatMap((g) =>
      g.options.filter((o) => getOptionLabel(o).toLowerCase().includes(q)),
    );
  }, [groups, search, getOptionLabel]);

  // Reset active index when filtered options change
  useEffect(() => {
    setActiveIndex(0);
  }, [filteredOptions.length]);

  // Focus search input when opened
  useEffect(() => {
    if (open) {
      setTimeout(() => searchInputRef.current?.focus(), 0);
      setSearch("");
    }
  }, [open]);

  const selectOption = useCallback(
    (optionValue: string) => {
      onChange(optionValue);
      setOpen(false);
      setSearch("");
    },
    [onChange],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setActiveIndex((prev) =>
            prev < filteredOptions.length - 1 ? prev + 1 : prev,
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setActiveIndex((prev) => (prev > 0 ? prev - 1 : 0));
          break;
        case "Enter":
          e.preventDefault();
          if (filteredOptions[activeIndex]) {
            selectOption(getOptionValue(filteredOptions[activeIndex]));
          }
          break;
        case "Escape":
          e.preventDefault();
          setOpen(false);
          break;
      }
    },
    [filteredOptions, activeIndex, selectOption, getOptionValue],
  );

  // Scroll active item into view
  useEffect(() => {
    if (listRef.current) {
      const activeEl = listRef.current.querySelector(
        `[data-option-index="${activeIndex}"]`,
      );
      activeEl?.scrollIntoView({ block: "nearest" });
    }
  }, [activeIndex]);

  // Filter groups: only show groups that have matching options
  const visibleGroups = useMemo(() => {
    if (!search.trim()) return groups;
    const q = search.toLowerCase();
    return groups
      .map((g) => ({
        ...g,
        options: g.options.filter((o) =>
          getOptionLabel(o).toLowerCase().includes(q),
        ),
      }))
      .filter((g) => g.options.length > 0);
  }, [groups, search, getOptionLabel]);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          disabled={disabled}
          className={cn(
            "flex h-12 w-full min-w-48 items-center justify-between rounded-md border px-3 py-1 text-base shadow-xs",
            "border-input bg-transparent",
            "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:outline-none",
            "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
            !selectedLabel && "text-muted-foreground",
            className,
          )}
        >
          <span className="truncate">{selectedLabel || placeholder}</span>
          <Icon
            name="lu:ChevronsUpDown"
            className="ml-2 h-4 w-4 shrink-0 opacity-50"
          />
        </button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[--radix-popover-trigger-width] p-0"
        align="start"
        onKeyDown={handleKeyDown}
      >
        <div className="flex items-center border-b px-3">
          <Icon name="lu:Search" className="mr-2 h-4 w-4 shrink-0 opacity-50" />
          <Input
            ref={searchInputRef}
            placeholder={searchPlaceholder}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="h-10 border-0 bg-transparent px-0 text-base shadow-none focus-visible:ring-0"
          />
        </div>
        <div ref={listRef} className="max-h-64 overflow-auto p-1">
          {visibleGroups.length === 0 ? (
            <div className="text-muted-foreground px-2 py-6 text-center text-sm">
              {emptyMessage}
            </div>
          ) : (
            visibleGroups.map((group) => (
              <div key={group.label}>
                <div className="text-muted-foreground px-2 py-1.5 text-xs font-semibold">
                  {group.label}
                </div>
                {group.options.map((option) => {
                  const optionValue = getOptionValue(option);
                  const optionLabel = getOptionLabel(option);
                  const isSelected = value === optionValue;
                  // Find the flat index for keyboard nav
                  const flatIndex = filteredOptions.findIndex(
                    (o) => getOptionValue(o) === optionValue,
                  );
                  return (
                    <div
                      key={optionValue}
                      data-option-index={flatIndex}
                      role="option"
                      aria-selected={isSelected}
                      onClick={() => selectOption(optionValue)}
                      className={cn(
                        "relative flex cursor-pointer items-center rounded-sm px-2 py-2 text-base",
                        "hover:bg-accent hover:text-accent-foreground",
                        activeIndex === flatIndex &&
                          "bg-accent text-accent-foreground",
                        isSelected && "font-medium",
                      )}
                    >
                      {isSelected && (
                        <Icon
                          name="lu:Check"
                          className="mr-2 h-4 w-4 shrink-0"
                        />
                      )}
                      <span className={cn(!isSelected && "ml-6")}>
                        {optionLabel}
                      </span>
                    </div>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
