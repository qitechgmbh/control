import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import laserManualContent from "@/assets/markdown/laser/manual.md?raw";

export function Laser1ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={laserManualContent} />
    </Page>
  );
}
