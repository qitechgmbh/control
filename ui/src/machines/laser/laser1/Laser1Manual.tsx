import { Page } from "@ui/components/Page";
import { MarkdownWithToc } from "@ui/components/MarkdownWithToc";
import React from "react";
import laserManualContent from "@ui/assets/markdown/laser/manual.md?raw";

export function Laser1ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={laserManualContent} />
    </Page>
  );
}
