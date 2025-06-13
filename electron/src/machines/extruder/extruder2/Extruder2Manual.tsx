import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import extruderManualContent from "@/assets/markdown/extruder/manual.md?raw";

export function ExtruderV2ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={extruderManualContent} />
    </Page>
  );
}
