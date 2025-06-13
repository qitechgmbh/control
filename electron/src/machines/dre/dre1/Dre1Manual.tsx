import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import dreManualContent from "@/assets/markdown/dre/manual.md?raw";

export function Dre1ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={dreManualContent} />
    </Page>
  );
}
