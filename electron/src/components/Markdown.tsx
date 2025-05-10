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
          h1: ({ node, ...props }) => (
            <h1 className="pt-4 pb-2 text-2xl" {...props} />
          ),
          h2: ({ node, ...props }) => (
            <h2 className="pt-4 pb-2 text-xl" {...props} />
          ),
          h3: ({ node, ...props }) => (
            <h3 className="pt-4 pb-2 text-lg" {...props} />
          ),
          h4: ({ node, ...props }) => (
            <h4 className="pt-4 pb-2 text-lg" {...props} />
          ),
          ul: ({ node, ...props }) => (
            <ul className="markdown-list list-inside list-disc" {...props} />
          ),
          li: ({ node, ...props }) => (
            <li className="markdown-list list-inside list-disc" {...props} />
          ),
          ol: ({ node, ...props }) => (
            <ol className="markdown-list list-inside list-decimal" {...props} />
          ),
          p: ({ node, ...props }) => <p className="text-base" {...props} />,
          a: ({ node, ...props }) => (
            <a
              className="text-blue-500 underline"
              {...props}
              target="_blank"
              rel="noreferrer"
            />
          ),
          strong: ({ node, ...props }) => (
            <strong className="font-bold" {...props} />
          ),
          img: ({ node, ...props }) => (
            <img
              className="h-auto max-w-full"
              {...props}
              alt={props.alt || ""}
              loading="lazy"
            />
          ),
          code: ({ node, ...props }) => (
            <code
              className="rounded-full bg-gray-100 p-0.5 px-2 text-sm"
              {...props}
            />
          ),
          blockquote: ({ node, ...props }) => (
            <blockquote
              className="border-l-4 border-gray-300 pl-4 text-gray-700 italic"
              {...props}
            />
          ),
          pre: ({ node, ...props }) => (
            <pre
              className="my-4 overflow-auto rounded bg-gray-100 p-4"
              {...props}
            />
          ),
          table: ({ node, ...props }) => (
            <table className="my-4 w-full table-auto" {...props} />
          ),
          thead: ({ node, ...props }) => <thead className="" {...props} />,
          tbody: ({ node, ...props }) => <tbody className="" {...props} />,
          tr: ({ node, ...props }) => (
            <tr className="border-b border-gray-300" {...props} />
          ),
          th: ({ node, ...props }) => (
            <th className="p-2 text-left font-bold" {...props} />
          ),
          td: ({ node, ...props }) => (
            <td className="p-2 text-left" {...props} />
          ),
          hr: ({ node, ...props }) => <hr className="my-4" {...props} />,
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}
