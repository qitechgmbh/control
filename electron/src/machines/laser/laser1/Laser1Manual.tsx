import { Page } from "@/components/Page";
import { MarkdownWithToc } from "@/components/MarkdownWithToc";
import React from "react";
import laserManualContent from "@/assets/markdown/laser/manual.md?raw";

console.log("LASER CONTENT:", laserManualContent);
console.log("LASER TYPE:", typeof laserManualContent);

export function Laser1ManualPage() {
  return (
    <Page>
      <MarkdownWithToc markdownContent={laserManualContent} />
    </Page>
  );
}
