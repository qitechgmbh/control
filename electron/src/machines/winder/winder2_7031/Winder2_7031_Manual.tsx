import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import winderManualContent from "@/assets/markdown/winder/manual.md?raw";

export function Winder2_7031ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={winderManualContent} />
    </Page>
  );
}
