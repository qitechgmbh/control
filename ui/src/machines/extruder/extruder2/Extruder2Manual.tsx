import { Page } from "@ui/components/Page";
import { MarkdownWithToc } from "@ui/components/MarkdownWithToc";
import React from "react";
import extruderManualContent from "@ui/assets/markdown/extruder/manual.md?raw";

export function ExtruderV2ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={extruderManualContent} />
    </Page>
  );
}
