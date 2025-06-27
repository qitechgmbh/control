/**
 * Utility for processing markdown content with automatic image resolution
 */

/**
 * Processes markdown content by replacing relative image paths with imported asset URLs
 * @param markdownContent The raw markdown content
 * @param imageImports Object mapping relative paths to imported image URLs
 * @returns Processed markdown content with resolved image URLs
 */
export function processMarkdownImages(
  markdownContent: string,
  imageImports: Record<string, string>,
): string {
  let processedContent = markdownContent;

  // Replace each image path with its corresponding import
  Object.entries(imageImports).forEach(([relativePath, importedUrl]) => {
    // Handle different markdown image syntaxes
    const patterns = [
      new RegExp(`!\\[([^\\]]*)\\]\\(${escapeRegExp(relativePath)}\\)`, "g"),
      new RegExp(`<img[^>]*src="${escapeRegExp(relativePath)}"[^>]*>`, "g"),
    ];

    patterns.forEach((pattern) => {
      processedContent = processedContent.replace(pattern, (match) => {
        if (match.startsWith("![")) {
          // Standard markdown image syntax
          const altTextMatch = match.match(/!\[([^\]]*)\]/);
          const altText = altTextMatch ? altTextMatch[1] : "";
          return `![${altText}](${importedUrl})`;
        } else {
          // HTML img tag syntax
          return match.replace(relativePath, importedUrl);
        }
      });
    });
  });

  return processedContent;
}

/**
 * Escapes special regex characters in a string
 */
function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Convenience function for creating image import maps
 * Automatically handles common relative path formats
 */
export function createImageImports(
  imports: Record<string, string>,
): Record<string, string> {
  const imageImports: Record<string, string> = {};

  Object.entries(imports).forEach(([key, value]) => {
    // Add the original key
    imageImports[key] = value;

    // Also add variations for different relative path formats
    if (key.startsWith("./")) {
      imageImports[key.substring(2)] = value; // Remove './' prefix
    } else if (!key.startsWith("./")) {
      imageImports[`./${key}`] = value; // Add './' prefix
    }
  });

  return imageImports;
}
