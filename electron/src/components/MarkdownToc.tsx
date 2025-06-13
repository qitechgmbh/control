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
        false: "",
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
    // Update active headings
    const updateActiveHeadings = () => {
      const headingElements = allHeadingIds
        .map((id) => document.getElementById(id))
        .filter(Boolean) as HTMLElement[];

      if (headingElements.length === 0) {
        setActiveIds(new Set());
        return;
      }

      const visibleHeadings = headingElements.filter((element) => {
        const rect = element.getBoundingClientRect();
        const viewportHeight = window.innerHeight;
        // Consider element visible if it's in the top 95% of viewport
        return rect.top >= 0 && rect.top <= viewportHeight * 0.95;
      });

      const activeHeadingIds = new Set<string>();

      if (visibleHeadings.length > 0) {
        // Add all visible headings
        visibleHeadings.forEach((h) => activeHeadingIds.add(h.id));

        // Find the heading that comes before the first visible one
        const firstVisibleId = visibleHeadings[0].id;
        const firstVisibleIndex = allHeadingIds.indexOf(firstVisibleId);

        // If there's a heading before the first visible one, add it too
        if (firstVisibleIndex > 0) {
          const previousHeadingId = allHeadingIds[firstVisibleIndex - 1];
          activeHeadingIds.add(previousHeadingId);
        }
      }

      setActiveIds(activeHeadingIds);
    };

    // Set initial active headings immediately
    updateActiveHeadings();

    // Scroll listener for instant responsiveness
    const handleScroll = () => {
      updateActiveHeadings();
    };

    // Add scroll listener to the scroll container
    const scrollContainer =
      document.querySelector("[data-scroll-container]") ||
      document.querySelector(".overflow-y-auto") ||
      window;
    scrollContainer.addEventListener("scroll", handleScroll, { passive: true });

    return () => {
      scrollContainer.removeEventListener("scroll", handleScroll);
    };
  }, [allHeadingIds]);

  // Additional effect to ensure immediate highlighting on mount and content changes
  useEffect(() => {
    // Use requestAnimationFrame to ensure DOM is fully rendered
    const checkVisible = () => {
      const headingElements = allHeadingIds
        .map((id) => document.getElementById(id))
        .filter(Boolean) as HTMLElement[];

      if (headingElements.length > 0) {
        const visibleHeadings = headingElements.filter((element) => {
          const rect = element.getBoundingClientRect();
          const viewportHeight = window.innerHeight;
          return rect.top >= 0 && rect.top <= viewportHeight * 0.95;
        });

        if (visibleHeadings.length > 0) {
          setActiveIds(new Set(visibleHeadings.map((h) => h.id)));
        }
      }
    };

    requestAnimationFrame(checkVisible);
  }, [markdownContent, allHeadingIds]);

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
      const title = match[2].trim();
      const id = generateHeadingId(title);
      headings.push({ level, title, id });
    }
  });

  return buildTocTree(headings);
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
