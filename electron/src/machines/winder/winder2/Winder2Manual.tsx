import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import winderManualContent from "@root/docs/machines/manuals/winder.md?raw";

export function Winder2ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={winderManualContent} isManual={true} />
    </Page>
  );
}
