import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import laserManualContent from "@root/docs/machines/manuals/laser.md?raw";

export function Laser1ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={laserManualContent} isManual={true} />
    </Page>
  );
}
