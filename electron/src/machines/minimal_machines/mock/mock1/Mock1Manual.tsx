import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import mockManualContent from "@/assets/markdown/mock/manual.md?raw";

export function Mock1ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={mockManualContent} />
    </Page>
  );
}
