import { useMemo } from "react";
import {
  processMarkdownImages,
  createImageImports,
} from "@/lib/markdown/images";

/**
 * Hook for processing markdown content with image imports
 * @param markdownContent Raw markdown content
 * @param imageImports Object mapping image filenames to imported URLs
 * @returns Processed markdown content ready for rendering
 */
export function useMarkdownWithImages(
  markdownContent: string,
  imageImports: Record<string, string> = {},
): string {
  return useMemo(() => {
    const processedImageImports = createImageImports(imageImports);
    return processMarkdownImages(markdownContent, processedImageImports);
  }, [markdownContent, imageImports]);
}

/**
 * Type-safe markdown configuration
 */
export interface MarkdownConfig {
  content: string;
  images?: Record<string, string>;
}

/**
 * Hook for processing markdown with configuration object
 */
export function useMarkdownConfig(config: MarkdownConfig): string {
  return useMarkdownWithImages(config.content, config.images);
}
