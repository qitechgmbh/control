import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import mockManualContent from "@root/docs/machines/manuals/mock.md?raw";

export function Mock1ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={mockManualContent} isManual={true} />
    </Page>
  );
}
