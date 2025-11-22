import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import laserManualContent from "@/assets/markdown/laser/manual.md?raw";

export function XtremZebraManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={laserManualContent} />
    </Page>
  );
}
