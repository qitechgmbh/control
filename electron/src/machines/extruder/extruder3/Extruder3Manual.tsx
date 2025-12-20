import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import extruderManualContent from "@root/docs/machines/manuals/extruder.md?raw";

export function ExtruderV3ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={extruderManualContent} />
    </Page>
  );
}
