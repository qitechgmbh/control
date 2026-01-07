import React from "react";
import { Markdown } from "./Markdown";
import { MarkdownToc } from "./MarkdownToc";

type MarkdownWithTocProps = {
  /**
   * The markdown content to render
   */
  markdownContent: string;
  /**
   * Additional CSS classes for the table of contents container
   */
  tocClassName?: string;
  /**
   * Width of the table of contents sidebar (in Tailwind units)
   * @default "w-64"
   */
  tocWidth?: string;
  /**
   * Gap between the TOC and main content (in Tailwind units)
   * @default "gap-6"
   */
  gap?: string;
  /**
   * Sticky positioning offset from top (in Tailwind units)
   * @default "top-4"
   */
  stickyTop?: string;
};

/**
 * A combined component that renders markdown content alongside a table of contents
 *
 * This component provides a two-column layout with:
 * - Left main area: Rendered markdown content
 * - Right sidebar: Sticky table of contents for navigation
 *
 * The table of contents automatically generates from headings in the markdown
 * and provides clickable navigation to scroll to specific sections.
 */
export function MarkdownWithToc({
  markdownContent,
  tocClassName = "",
  tocWidth = "w-64",
  gap = "gap-6",
  stickyTop = "top-6",
}: MarkdownWithTocProps) {
  return (
    <div className={`flex ${gap}`}>
      {/* Main Content */}
      <div className="min-w-0 flex-1">
        <Markdown text={markdownContent} />
      </div>

      {/* Table of Contents Sidebar */}
      <div className={`${tocWidth} flex-shrink-0`}>
        <div
          className={`sticky ${stickyTop}`}
          style={{ maxHeight: "calc(100vh - 6rem)" }}
        >
          {/* Table of Contents with hidden scrollbar */}
          <div
            className="scrollbar-hidden overflow-y-auto"
            style={{ maxHeight: "calc(100vh - 8rem)" }}
          >
            <MarkdownToc
              markdownContent={markdownContent}
              className={tocClassName}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
