import React from "react";
import { Markdown } from "./Markdown";

type ManualMarkdownProps = {
  /**
   * The markdown content to render
   */
  markdownContent: string;
};

/**
 * A specialized Markdown component for rendering machine manuals.
 * 
 * This component transforms image paths in markdown to work correctly in the Electron app.
 * Manual markdown files are stored in `docs/machines/manuals/` with relative image paths
 * pointing to `../../../../electron/public/images/manuals/[machine-name]/...`
 * 
 * This component transforms those paths to absolute paths `/images/manuals/[machine-name]/...`
 * that work correctly in the Electron app's context.
 * 
 * Path transformation examples:
 * - `../../../../electron/public/images/manuals/winder/image.png` → `/images/manuals/winder/image.png`
 * - `images/manuals/winder/image.png` → `/images/manuals/winder/image.png`
 */
export function ManualMarkdown({ markdownContent }: ManualMarkdownProps) {
  const transformedContent = React.useMemo(() => {
    // Transform image paths to work in Electron context
    // Pattern 1: ../../../../electron/public/images/manuals/... → /images/manuals/...
    let content = markdownContent.replace(
      /!\[([^\]]*)\]\(\.\.\/\.\.\/\.\.\/\.\.\/electron\/public\/(images\/manuals\/[^)]+)\)/g,
      "![$1](/$2)"
    );
    
    // Pattern 2: ../../../electron/public/images/manuals/... → /images/manuals/... (backward compatibility)
    content = content.replace(
      /!\[([^\]]*)\]\(\.\.\/\.\.\/\.\.\/electron\/public\/(images\/manuals\/[^)]+)\)/g,
      "![$1](/$2)"
    );
    
    // Pattern 3: images/manuals/... → /images/manuals/... (for backward compatibility)
    content = content.replace(
      /!\[([^\]]*)\]\(images\/manuals\/([^)]+)\)/g,
      "![$1](/images/manuals/$2)"
    );
    
    // Pattern 4: Handle <img> tags with src attribute (new path)
    content = content.replace(
      /<img\s+([^>]*)src="\.\.\/\.\.\/\.\.\/\.\.\/electron\/public\/(images\/manuals\/[^"]+)"/g,
      '<img $1src="/$2"'
    );
    
    // Pattern 5: Handle <img> tags with src attribute (old path)
    content = content.replace(
      /<img\s+([^>]*)src="\.\.\/\.\.\/\.\.\/electron\/public\/(images\/manuals\/[^"]+)"/g,
      '<img $1src="/$2"'
    );
    
    content = content.replace(
      /<img\s+([^>]*)src="images\/manuals\/([^"]+)"/g,
      '<img $1src="/images/manuals/$2"'
    );
    
    return content;
  }, [markdownContent]);

  return <Markdown text={transformedContent} />;
}
