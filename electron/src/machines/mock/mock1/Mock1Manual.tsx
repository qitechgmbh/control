import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import handbookContent from "@/assets/markdown/mock/manual.md?raw";
import duckImage from "@/assets/markdown/mock/duck.jpg";
import { useMarkdownWithImages } from "@/hooks/useMarkdownWithImages";

export function Mock1ManualPage() {
  // Process markdown content with automatic image resolution
  const processedContent = useMarkdownWithImages(handbookContent, {
    "duck.jpg": duckImage,
  });

  return (
    <Page>
      <MarkdownWithToc markdownContent={processedContent} />
    </Page>
  );
}
