import React, {
  useEffect,
  useLayoutEffect,
  useState,
  useCallback,
  useMemo,
} from "react";
import { cva } from "class-variance-authority";
import { cn } from "@/lib/utils";
import { generateHeadingId } from "@/lib/markdown/heading";

interface TocItem {
  id: string;
  title: string;
  level: number;
  children: TocItem[];
}

interface MarkdownTocProps {
  markdownContent: string;
  className?: string;
}

// CVA variants for TOC container
const tocContainerVariants = cva("relative flex items-center", {
  variants: {
    isActive: {
      true: "bg-gray-100 dark:bg-gray-800",
      false: "",
    },
    roundedCorners: {
      none: "",
      top: "rounded-t-lg",
      bottom: "rounded-b-lg",
      all: "rounded-lg",
    },
  },
  defaultVariants: {
    isActive: false,
    roundedCorners: "none",
  },
});

// CVA variants for TOC button
const tocButtonVariants = cva(
  "w-full px-2 py-2 text-left text-sm transition-all duration-150 text-gray-900 dark:text-gray-100",
  {
    variants: {
      isActive: {
        true: "pr-4",
        false: "pr-4", // keep same padding to avoid layout shift
      },
      depth: {
        0: "",
        1: "",
        2: "",
        3: "",
        4: "",
        5: "",
      },
    },
    defaultVariants: {
      isActive: false,
      depth: 0,
    },
  },
);

/**
 * Table of Contents component that automatically generates navigation
 * from markdown headings with active section highlighting
 */
export function MarkdownToc({
  markdownContent,
  className = "",
}: MarkdownTocProps) {
  const tocItems = useMemo(
    () => parseMarkdownHeadings(markdownContent),
    [markdownContent],
  );
  const [activeIds, setActiveIds] = useState<Set<string>>(new Set());

  // Get all heading IDs in visual order for active range detection
  const allHeadingIds = useMemo(() => {
    const flattenHeadings = (items: TocItem[]): string[] => {
      return items.flatMap((item) => [
        item.id,
        ...flattenHeadings(item.children),
      ]);
    };
    return flattenHeadings(tocItems);
  }, [tocItems]);

  // Find first and last active items for continuous highlighting
  const { firstActiveId, lastActiveId } = useMemo(() => {
    const activeItemsInOrder = allHeadingIds.filter((id) => activeIds.has(id));
    return {
      firstActiveId: activeItemsInOrder[0] || null,
      lastActiveId: activeItemsInOrder[activeItemsInOrder.length - 1] || null,
    };
  }, [allHeadingIds, activeIds]);

  // Handle TOC item click navigation
  const handleItemClick = useCallback((id: string) => {
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: "smooth" });
    }
  }, []);

  // Set up intersection observer for active heading detection
  useLayoutEffect(() => {
    const updateActiveHeadings = () => {
      const headingElements = allHeadingIds
        .map((id) => document.getElementById(id))
        .filter(Boolean) as HTMLElement[];

      if (headingElements.length === 0) {
        setActiveIds(new Set());
        return;
      }

      const viewportHeight = window.innerHeight;
      const visibleHeadings = headingElements.filter((element) => {
        const rect = element.getBoundingClientRect();
        return rect.bottom > 0 && rect.top < viewportHeight;
      });

      const activeHeadingIds = new Set<string>();

      if (visibleHeadings.length > 0) {
        visibleHeadings.forEach((h) => activeHeadingIds.add(h.id));
        // Add previous heading if available
        const firstVisibleIndex = allHeadingIds.indexOf(visibleHeadings[0].id);
        if (firstVisibleIndex > 0) {
          activeHeadingIds.add(allHeadingIds[firstVisibleIndex - 1]);
        }
      } else {
        // Fallback: find last heading above viewport
        const lastAbove = headingElements
          .filter((el) => el.getBoundingClientRect().top <= 0)
          .pop();
        if (lastAbove) {
          activeHeadingIds.add(lastAbove.id);
        } else {
          activeHeadingIds.add(headingElements[0].id);
        }
      }

      setActiveIds(activeHeadingIds);
    };

    updateActiveHeadings();

    const scrollContainer =
      document.querySelector("[data-scroll-container]") ||
      document.querySelector(".overflow-y-auto") ||
      window;

    scrollContainer.addEventListener("scroll", updateActiveHeadings, {
      passive: true,
    });
    return () =>
      scrollContainer.removeEventListener("scroll", updateActiveHeadings);
  }, [allHeadingIds]);

  // Additional effect to ensure immediate highlighting on mount and content changes
  useEffect(() => {
    const headingElements = allHeadingIds
      .map((id) => document.getElementById(id))
      .filter(Boolean) as HTMLElement[];

    if (headingElements.length > 0) {
      const viewportHeight = window.innerHeight;
      const visibleHeadings = headingElements.filter((element) => {
        const rect = element.getBoundingClientRect();
        return rect.bottom > 0 && rect.top < viewportHeight;
      });

      if (visibleHeadings.length > 0) {
        setActiveIds(new Set(visibleHeadings.map((h) => h.id)));
      } else {
        const lastAbove = headingElements
          .filter((el) => el.getBoundingClientRect().top <= 0)
          .pop();
        setActiveIds(new Set([lastAbove?.id || headingElements[0].id]));
      }
    }
  }, [markdownContent, allHeadingIds]);

  // Auto-scroll the TOC so the active heading stays comfortably in view within the TOC scroll container
  useEffect(() => {
    const targetId = firstActiveId ?? lastActiveId;
    if (!targetId) return;

    const btn = document.querySelector(
      `button[data-toc-id="${targetId}"]`,
    ) as HTMLElement | null;
    if (!btn) return;

    // Prefer an explicitly marked container, else fall back to nearest overflow container
    const container =
      (btn.closest("[data-toc-scroll-container]") as HTMLElement | null) ||
      (btn.closest(".overflow-y-auto") as HTMLElement | null);

    if (!container) return;

    const cRect = container.getBoundingClientRect();
    const bRect = btn.getBoundingClientRect();

    // Compute the button's top in container's scroll coordinates
    const btnTopInContainer = bRect.top - cRect.top + container.scrollTop;
    const btnBottomInContainer = btnTopInContainer + bRect.height;

    // Keep the active item within a comfortable band (middle 40%)
    const comfortOffset = Math.max(
      32,
      Math.round(container.clientHeight * 0.3),
    );
    const upperComfort = container.scrollTop + comfortOffset;
    const lowerComfort =
      container.scrollTop + container.clientHeight - comfortOffset;

    let newTop: number | null = null;

    if (btnTopInContainer < upperComfort) {
      // Active item is too close to the top, move it down a bit (towards 30% from top)
      newTop = btnTopInContainer - comfortOffset;
    } else if (btnBottomInContainer > lowerComfort) {
      // Active item is too close to the bottom, move it up (towards 70% from top)
      newTop = btnBottomInContainer - (container.clientHeight - comfortOffset);
    }

    if (newTop !== null) {
      // Clamp scroll position to valid range
      const maxTop = Math.max(
        0,
        container.scrollHeight - container.clientHeight,
      );
      const clampedTop = Math.min(Math.max(newTop, 0), maxTop);
      container.scrollTo({ top: clampedTop, behavior: "auto" });
    }
  }, [firstActiveId, lastActiveId]);

  // Helper function to determine rounded corner variant
  const getRoundedCorners = useCallback(
    (isActive: boolean, isFirst: boolean, isLast: boolean) => {
      if (!isActive) return "none" as const;

      if (isFirst && isLast) return "all" as const;
      if (isFirst) return "top" as const;
      if (isLast) return "bottom" as const;
      return "none" as const;
    },
    [],
  );

  // Helper function to get depth variant (capped at 5 for CVA)
  const getDepthVariant = useCallback((depth: number) => {
    return Math.min(depth, 5) as 0 | 1 | 2 | 3 | 4 | 5;
  }, []);

  // Render individual TOC items with proper styling and nesting
  const renderTocItems = useCallback(
    (items: TocItem[], depth = 0) => {
      return (
        <div className="relative">
          {items.map((item) => {
            const isActive = activeIds.has(item.id);
            const leftPadding = depth === 0 ? 16 : depth * 16 + 8;
            const isFirstActive = item.id === firstActiveId;
            const isLastActive = item.id === lastActiveId;
            const roundedCorners = getRoundedCorners(
              isActive,
              isFirstActive,
              isLastActive,
            );
            const depthVariant = getDepthVariant(depth);

            return (
              <div key={item.id} className="relative">
                <div
                  className={tocContainerVariants({
                    isActive,
                    roundedCorners,
                  })}
                >
                  <button
                    onClick={() => handleItemClick(item.id)}
                    className={tocButtonVariants({
                      isActive,
                      depth: depthVariant,
                    })}
                    style={{ paddingLeft: `${leftPadding}px` }}
                    data-toc-id={item.id}
                  >
                    {item.title}
                  </button>
                </div>

                {item.children.length > 0 && (
                  <div className="relative">
                    {renderTocItems(item.children, depth + 1)}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      );
    },
    [
      activeIds,
      firstActiveId,
      lastActiveId,
      handleItemClick,
      getRoundedCorners,
      getDepthVariant,
    ],
  );

  if (tocItems.length === 0) {
    return (
      <div
        className={cn("text-sm text-gray-500 dark:text-gray-400", className)}
      >
        No headings found
      </div>
    );
  }

  return (
    <nav
      className={cn("toc w-full text-xs", className)}
      aria-label="Table of contents"
    >
      <div>{renderTocItems(tocItems)}</div>
    </nav>
  );
}

function parseMarkdownHeadings(markdownContent: string): TocItem[] {
  const lines = markdownContent.split("\n");
  const headings: Array<{ level: number; title: string; id: string }> = [];

  lines.forEach((line) => {
    const match = line.match(/^(#{1,6})\s+(.+)$/);
    if (match) {
      const level = match[1].length;
      const rawTitle = match[2].trim();
      // Strip markdown formatting for TOC display
      const title = stripMarkdownInlineFormatting(rawTitle);
      const id = generateHeadingId(rawTitle); // Use raw title for ID generation
      headings.push({ level, title, id });
    }
  });

  return buildTocTree(headings);
}

/**
 * Strip markdown inline formatting from text for TOC display
 * Removes: **bold**, *italic*, `code`, ~~strikethrough~~, links, etc.
 */
function stripMarkdownInlineFormatting(text: string): string {
  return (
    text
      // Remove bold (**text** or __text__)
      .replace(/\*\*(.*?)\*\*/g, "$1")
      .replace(/__(.*?)__/g, "$1")
      // Remove italic (*text* or _text_)
      .replace(/\*(.*?)\*/g, "$1")
      .replace(/_(.*?)_/g, "$1")
      // Remove strikethrough (~~text~~)
      .replace(/~~(.*?)~~/g, "$1")
      // Remove inline code (`text`)
      .replace(/`([^`]+)`/g, "$1")
      // Remove links [text](url) -> text
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      // Remove reference-style links [text][ref] -> text
      .replace(/\[([^\]]+)\]\[[^\]]*\]/g, "$1")
      // Remove images ![alt](url) -> alt
      .replace(/!\[([^\]]*)\]\([^)]+\)/g, "$1")
      // Clean up any remaining markdown characters
      .replace(/[*_`~]/g, "")
      .trim()
  );
}

function buildTocTree(
  headings: Array<{ level: number; title: string; id: string }>,
): TocItem[] {
  const root: TocItem[] = [];
  const stack: TocItem[] = [];

  headings.forEach(({ level, title, id }) => {
    const item: TocItem = {
      id,
      title,
      level,
      children: [],
    };

    while (stack.length > 0 && stack[stack.length - 1].level >= level) {
      stack.pop();
    }

    if (stack.length === 0) {
      root.push(item);
    } else {
      stack[stack.length - 1].children.push(item);
    }

    stack.push(item);
  });

  return root;
}
