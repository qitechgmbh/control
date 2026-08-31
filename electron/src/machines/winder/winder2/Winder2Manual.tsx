import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import winderManualContent from "@/assets/markdown/winder/manual.md?raw";

export function Winder2ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={winderManualContent} />
    </Page>
  );
}
