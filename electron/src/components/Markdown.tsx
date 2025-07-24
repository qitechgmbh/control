import React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import { generateHeadingId } from "@/lib/markdown/heading";

type MarkdownProps = {
  text: string;
};

// Extract plain text from React children
function extractTextFromChildren(children: React.ReactNode): string {
  return React.Children.toArray(children)
    .map((child) => {
      if (typeof child === "string") return child;
      if (typeof child === "number") return child.toString();
      if (
        React.isValidElement(child) &&
        typeof child.props === "object" &&
        child.props !== null
      ) {
        return extractTextFromChildren((child.props as any).children || "");
      }
      return "";
    })
    .join("");
}

export function Markdown({ text }: MarkdownProps) {
  /**
   * First heading detection for removing top padding
   *
   * This logic detects if the markdown content starts with a heading and removes
   * the top padding from only that first heading to create better visual spacing
   * between the topbar and content.
   *
   * How it works:
   * 1. Parse the first line of markdown text using regex: /^(#{1,6})\s+(.+)/
   *    - Matches 1-6 hash symbols at the start of the text
   *    - Captures the heading level (number of #) and the heading text
   * 2. Each heading component (h1, h2, h3, h4) calls isFirstHeading() with its text and level
   * 3. Only the heading that exactly matches the first heading's text and level gets isFirst = true
   * 4. The first heading renders without pt-4 (top padding), all others keep their padding
   *
   * Example: For markdown starting with "# Mock Machine Manual"
   * - firstHeadingInfo = { level: 1, text: "Mock Machine Manual", startsWithHeading: true }
   * - Only the h1 with text "Mock Machine Manual" will have no top padding
   * - Subsequent headings like "## Overview" will keep their normal pt-4 padding
   */
  const firstHeadingInfo = React.useMemo(() => {
    const trimmedText = text.trim();
    const firstLineMatch = trimmedText.match(/^(#{1,6})\s+(.+)/);
    if (firstLineMatch) {
      return {
        level: firstLineMatch[1].length,
        text: firstLineMatch[2].trim(),
        startsWithHeading: true,
      };
    }
    return { startsWithHeading: false };
  }, [text]);

  const isFirstHeading = (headingText: string, level: number) => {
    return (
      firstHeadingInfo.startsWithHeading &&
      firstHeadingInfo.text === headingText &&
      firstHeadingInfo.level === level
    );
  };

  return (
    <div>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw]}
        components={{
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h1: ({ node, children, ...props }) => {
            const text = extractTextFromChildren(children);
            const id = generateHeadingId(text);
            const isFirst = isFirstHeading(text, 1);
            return (
              <>
                <h1
                  id={id}
                  className={`${isFirst ? "" : "pt-4"} pb-2 text-2xl font-bold`}
                  {...props}
                >
                  {children}
                </h1>
                <hr className="my-2" />
                <br />
              </>
            );
          },
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h2: ({ node, children, ...props }) => {
            const text = extractTextFromChildren(children);
            const id = generateHeadingId(text);
            const isFirst = isFirstHeading(text, 2);
            return (
              <>
                <h2
                  id={id}
                  className={`${isFirst ? "" : "pt-4"} pb-2 text-xl font-bold`}
                  {...props}
                >
                  {children}
                </h2>
                <hr className="my-2" />
              </>
            );
          },
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h3: ({ node, children, ...props }) => {
            const text = extractTextFromChildren(children);
            const id = generateHeadingId(text);
            const isFirst = isFirstHeading(text, 3);
            return (
              <>
                <h3
                  id={id}
                  className={`${isFirst ? "" : "pt-4"} pb-2 text-lg font-bold`}
                  {...props}
                >
                  {children}
                </h3>
                <hr className="my-2" />
              </>
            );
          },
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h4: ({ node, children, ...props }) => {
            const text = extractTextFromChildren(children);
            const id = generateHeadingId(text);
            const isFirst = isFirstHeading(text, 4);
            return (
              <>
                <h4
                  id={id}
                  className={`${isFirst ? "" : "pt-4"} pb-2 text-lg font-bold`}
                  {...props}
                >
                  {children}
                </h4>
                <hr className="my-2" />
              </>
            );
          },
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          ul: ({ node, ...props }) => (
            <ul
              className="markdown-list mb-6 ml-6 list-outside list-disc"
              {...props}
            />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          li: ({ node, ...props }) => (
            <li className="markdown-list" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          ol: ({ node, ...props }) => (
            <ol
              className="markdown-list mb-6 ml-6 list-outside list-decimal"
              {...props}
            />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          p: ({ node, ...props }) => <p className="text-base" {...props} />,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          a: ({ node, ...props }) => (
            <a
              className="text-blue-500 underline"
              {...props}
              target="_blank"
              rel="noreferrer"
            />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          strong: ({ node, ...props }) => (
            <strong className="[font-size:inherit] font-bold" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          em: ({ node, ...props }) => (
            <em className="[font-size:inherit] italic" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          del: ({ node, ...props }) => (
            <del className="[font-size:inherit] line-through" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          img: ({ node, alt, ...props }) => (
            <img
              className="h-auto max-w-full"
              {...props}
              alt={alt || ""}
              loading="lazy"
            />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          code: ({ node, ...props }) => (
            <code
              className="rounded-sm bg-gray-100 p-0.5 px-2 [font-size:inherit]"
              {...props}
            />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          blockquote: ({ node, ...props }) => (
            <blockquote
              className="border-l-4 border-gray-300 pl-4 text-gray-700 italic"
              {...props}
            />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          pre: ({ node, ...props }) => (
            <pre
              className="my-4 overflow-auto rounded bg-gray-100 p-4"
              {...props}
            />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          table: ({ node, ...props }) => (
            <table className="my-4 w-full table-auto" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          thead: ({ node, ...props }) => <thead className="" {...props} />,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          tbody: ({ node, ...props }) => <tbody className="" {...props} />,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          tr: ({ node, ...props }) => (
            <tr className="border-b border-gray-300" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          th: ({ node, ...props }) => (
            <th className="p-2 text-left font-bold" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          td: ({ node, ...props }) => (
            <td className="p-2 text-left" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          hr: ({ node, ...props }) => <hr className="my-4" {...props} />,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          br: ({ node, ...props }) => <br {...props} />,
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}
