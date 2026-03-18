import { Page } from "@ui/components/Page";
import { MarkdownWithToc } from "@ui/components/MarkdownWithToc";
import React from "react";
import winderManualContent from "@ui/assets/markdown/winder/manual.md?raw";

export function Winder2ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={winderManualContent} />
    </Page>
  );
}
