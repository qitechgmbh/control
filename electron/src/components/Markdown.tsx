import React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

type MarkdownProps = {
  text: string;
};

export function Markdown({ text }: MarkdownProps) {
  return (
    <div>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h1: ({ node, ...props }) => (
            <h1 className="pt-4 pb-2 text-2xl" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h2: ({ node, ...props }) => (
            <h2 className="pt-4 pb-2 text-xl" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h3: ({ node, ...props }) => (
            <h3 className="pt-4 pb-2 text-lg" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          h4: ({ node, ...props }) => (
            <h4 className="pt-4 pb-2 text-lg" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          ul: ({ node, ...props }) => (
            <ul className="markdown-list list-inside list-disc" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          li: ({ node, ...props }) => (
            <li className="markdown-list list-inside list-disc" {...props} />
          ),
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          ol: ({ node, ...props }) => (
            <ol className="markdown-list list-inside list-decimal" {...props} />
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
            <strong className="font-bold" {...props} />
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
              className="rounded-full bg-gray-100 p-0.5 px-2 text-sm"
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
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}
