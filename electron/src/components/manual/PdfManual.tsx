import React, {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { getDocument, GlobalWorkerOptions } from "pdfjs-dist";
import type { PDFDocumentProxy } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { cva } from "class-variance-authority";
import { cn } from "@/lib/utils";
import { Icon } from "@/components/Icon";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

GlobalWorkerOptions.workerSrc = workerUrl;

/** Machine manuals shipped under `public/manuals/<machine>/<language>.pdf`. */
export type ManualMachine = "aquapath" | "extruder" | "laser" | "winder";
type ManualLanguage = "en" | "de";

const LANGUAGES: { key: ManualLanguage; label: string }[] = [
  { key: "en", label: "English" },
  { key: "de", label: "Deutsch" },
];
const LANGUAGE_STORAGE_KEY = "manualLanguage";
const ZOOM_STORAGE_KEY = "manualZoom";
/** Zoom levels relative to the width of the page column (1 = fit width). */
const ZOOM_STEPS = [0.5, 0.6, 0.7, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2];
/** Gap kept above a heading after jumping to it. */
const SCROLL_MARGIN = 24;
/** Line below the top of the scroll area that decides the current section. */
const PROBE_OFFSET = 48;
/** Upper bound for the canvas pixel ratio, keeps memory in check on HiDPI panels. */
const MAX_PIXEL_RATIO = 2;
/** Pages closer than this to the viewport get rendered, farther ones are released. */
const RENDER_MARGIN = "1500px 0px";

const A4_RATIO = 842 / 596;

/** A spot in the document: page number plus how far down that page (0..1). */
type Target = { page: number; offset: number };
type OutlineEntry = Target & { title: string; level: number };
type LoadedManual = {
  url: string;
  doc: PDFDocumentProxy | null;
  outline: OutlineEntry[];
  pageRatio: number;
  error: string | null;
};
type Destination = string | unknown[] | null;

function readStoredLanguage(): ManualLanguage {
  try {
    const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY);
    if (stored === "en" || stored === "de") return stored;
  } catch {
    // storage unavailable, fall back to English
  }
  return "en";
}

function readStoredZoom(): number {
  try {
    const stored = Number(localStorage.getItem(ZOOM_STORAGE_KEY));
    if (ZOOM_STEPS.includes(stored)) return stored;
  } catch {
    // storage unavailable, fall back to fit width
  }
  return 1;
}

function store(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    // storage unavailable, the choice just isn't remembered
  }
}

/**
 * Resolves a PDF destination to its page and the height of the target on
 * that page, so headings further down a page are reached directly.
 */
async function resolveTarget(
  doc: PDFDocumentProxy,
  dest: Destination,
): Promise<Target | null> {
  try {
    const explicit =
      typeof dest === "string" ? await doc.getDestination(dest) : dest;
    if (!explicit || explicit.length === 0) return null;
    const ref = explicit[0];
    const page =
      typeof ref === "number"
        ? ref + 1
        : (await doc.getPageIndex(ref as never)) + 1;

    // [ref, {name: "XYZ"}, left, top, zoom] or [ref, {name: "FitH"}, top]
    const kind = (explicit[1] as { name?: string } | undefined)?.name;
    const top =
      kind === "XYZ"
        ? explicit[3]
        : kind === "FitH" || kind === "FitBH"
          ? explicit[2]
          : null;
    if (typeof top !== "number") return { page, offset: 0 };

    const [, y0, , y1] = (await doc.getPage(page)).view;
    const offset = (y1 - top) / (y1 - y0);
    return { page, offset: Math.min(Math.max(offset, 0), 1) };
  } catch {
    return null;
  }
}

/** Flattens the PDF bookmarks, dropping the empty headings Google Docs exports. */
async function loadOutline(doc: PDFDocumentProxy): Promise<OutlineEntry[]> {
  const outline = await doc.getOutline();
  const entries: OutlineEntry[] = [];

  const walk = async (
    items: Awaited<ReturnType<PDFDocumentProxy["getOutline"]>>,
    level: number,
  ) => {
    for (const item of items ?? []) {
      const title = item.title.replace(/\s+/g, " ").trim();
      if (title) {
        const target = await resolveTarget(doc, item.dest as Destination);
        if (target) entries.push({ title, level, ...target });
      }
      await walk(item.items, level + 1);
    }
  };
  await walk(outline, 0);
  return entries;
}

const tocItemVariants = cva(
  "w-full rounded-lg px-2 py-2 text-left text-sm transition-colors duration-150 text-gray-900 dark:text-gray-100",
  {
    variants: {
      isActive: {
        true: "bg-gray-100 dark:bg-gray-800",
        false: "hover:bg-gray-50 dark:hover:bg-gray-900",
      },
      isChapter: {
        true: "font-medium",
        false: "text-gray-700 dark:text-gray-300",
      },
    },
  },
);

type PdfManualProps = {
  machine: ManualMachine;
};

/**
 * Renders a machine manual PDF in the app's own style: pages as cards in the
 * main column, a sticky sidebar with a language switch and the PDF bookmarks
 * as table of contents.
 */
export function PdfManual({ machine }: PdfManualProps) {
  const [language, setLanguage] = useState<ManualLanguage>(readStoredLanguage);
  const [loaded, setLoaded] = useState<LoadedManual | null>(null);
  const [zoom, setZoom] = useState(readStoredZoom);
  // Reading position as page + offset, e.g. 31.5 is halfway down page 31.
  const [position, setPosition] = useState(1);
  const [availableWidth, setAvailableWidth] = useState(0);
  const rootRef = useRef<HTMLDivElement>(null);
  const pagesRef = useRef<HTMLDivElement>(null);
  const positionRef = useRef(1);
  const zoomAnchorRef = useRef<number | null>(null);

  const pageWidth = Math.round(availableWidth * zoom);
  const currentPage = Math.floor(position);

  const url = `manuals/${machine}/${language}.pdf`;
  // Ignore results that belong to a previous machine/language.
  const current = loaded?.url === url ? loaded : null;
  const doc = current?.doc ?? null;
  const outline = current?.outline ?? [];
  const pageRatio = current?.pageRatio ?? A4_RATIO;
  const error = current?.error ?? null;

  // Load the document whenever machine or language changes.
  useEffect(() => {
    let cancelled = false;
    const task = getDocument({ url });

    task.promise
      .then(async (pdf) => {
        const first = await pdf.getPage(1);
        const viewport = first.getViewport({ scale: 1 });
        const entries = await loadOutline(pdf);
        if (cancelled) return;
        setLoaded({
          url,
          doc: pdf,
          outline: entries,
          pageRatio: viewport.height / viewport.width,
          error: null,
        });
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setLoaded({
          url,
          doc: null,
          outline: [],
          pageRatio: A4_RATIO,
          error: e instanceof Error ? e.message : String(e),
        });
      });

    return () => {
      cancelled = true;
      void task.destroy();
    };
  }, [url]);

  // Track the width available for pages.
  useEffect(() => {
    const el = pagesRef.current;
    if (!el) return;
    const observer = new ResizeObserver(([entry]) => {
      setAvailableWidth(Math.floor(entry.contentRect.width));
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Track the reading position at a line just below the top of the scroll area.
  useEffect(() => {
    const root = rootRef.current;
    if (!doc || !root) return;
    const scroller = root.closest<HTMLElement>(".overflow-y-auto");
    const target: HTMLElement | Window = scroller ?? window;

    let frame = 0;
    const update = () => {
      frame = 0;
      const top = scroller ? scroller.getBoundingClientRect().top : 0;
      const probe = top + PROBE_OFFSET;
      const pages = root.querySelectorAll<HTMLElement>("[data-pdf-page]");
      let next = 1;
      for (const el of pages) {
        const rect = el.getBoundingClientRect();
        if (rect.top > probe) break;
        const offset = Math.min((probe - rect.top) / rect.height, 0.9999);
        next = Number(el.dataset.pdfPage) + offset;
      }
      positionRef.current = next;
      setPosition(next);
    };
    const onScroll = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };

    update();
    target.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      target.removeEventListener("scroll", onScroll);
      if (frame) cancelAnimationFrame(frame);
    };
  }, [doc]);

  /** Scrolls so `target` sits `align` pixels below the top of the scroll area. */
  const scrollTo = useCallback(
    (target: Target, align: number, behavior: ScrollBehavior) => {
      const root = rootRef.current;
      const el = root?.querySelector<HTMLElement>(
        `[data-pdf-page="${target.page}"]`,
      );
      if (!root || !el) return;
      const scroller = root.closest<HTMLElement>(".overflow-y-auto");
      const rect = el.getBoundingClientRect();
      const y = rect.top + target.offset * rect.height - align;
      if (scroller) {
        const top =
          scroller.scrollTop + y - scroller.getBoundingClientRect().top;
        scroller.scrollTo({ top, behavior });
      } else {
        window.scrollTo({ top: window.scrollY + y, behavior });
      }
    },
    [],
  );

  const navigate = useCallback(
    (target: Target) => scrollTo(target, SCROLL_MARGIN, "smooth"),
    [scrollTo],
  );

  // After a zoom change, put the previous reading position back on the probe line.
  useLayoutEffect(() => {
    const anchor = zoomAnchorRef.current;
    if (anchor === null) return;
    zoomAnchorRef.current = null;
    const page = Math.floor(anchor);
    scrollTo({ page, offset: anchor - page }, PROBE_OFFSET, "instant");
  }, [pageWidth, scrollTo]);

  const onLanguageChange = (value: string) => {
    if (value !== "en" && value !== "de") return;
    store(LANGUAGE_STORAGE_KEY, value);
    setLanguage(value);
    rootRef.current?.scrollIntoView({ block: "start" });
  };

  const zoomIndex = ZOOM_STEPS.indexOf(zoom);
  const changeZoom = (step: number) => {
    const next = ZOOM_STEPS[zoomIndex + step];
    if (next === undefined) return;
    zoomAnchorRef.current = positionRef.current;
    store(ZOOM_STORAGE_KEY, String(next));
    setZoom(next);
  };

  // The current section is the last heading at or above the reading position.
  const activeIndex = outline.reduce(
    (active, entry, index) =>
      entry.page + entry.offset <= position + 0.0001 ? index : active,
    -1,
  );

  return (
    <div ref={rootRef} className="flex gap-6">
      {/* Pages, scroll sideways when zoomed in past the column width */}
      <div ref={pagesRef} className="min-w-0 flex-1 overflow-x-auto">
        <div
          className="flex flex-col items-center gap-6"
          style={{ minWidth: pageWidth || undefined }}
        >
          {error && (
            <div className="rounded-lg border border-red-500/50 bg-red-500/10 p-4 text-sm text-red-700 dark:text-red-400">
              The manual could not be loaded: {error}
            </div>
          )}
          {!error &&
            !doc &&
            [0, 1].map((i) => (
              <Skeleton
                key={i}
                className="rounded-lg"
                style={{
                  width: pageWidth || "100%",
                  aspectRatio: `1 / ${pageRatio}`,
                }}
              />
            ))}
          {doc &&
            pageWidth > 0 &&
            Array.from({ length: doc.numPages }, (_, i) => (
              <PdfPage
                key={`${url}-${i + 1}`}
                doc={doc}
                pageNumber={i + 1}
                width={pageWidth}
                fallbackRatio={pageRatio}
                onNavigate={navigate}
              />
            ))}
        </div>
      </div>

      {/* Sidebar */}
      <div className="w-64 flex-shrink-0">
        <div
          className="sticky top-6 flex flex-col gap-4"
          style={{ maxHeight: "calc(100vh - 8rem)" }}
        >
          <ToggleGroup
            type="single"
            variant="outline"
            value={language}
            onValueChange={onLanguageChange}
            className="w-full"
          >
            {LANGUAGES.map((l) => (
              <ToggleGroupItem key={l.key} value={l.key} className="flex-1">
                {l.label}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>

          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="icon"
              aria-label="Zoom out"
              disabled={zoomIndex <= 0}
              onClick={() => changeZoom(-1)}
            >
              <Icon name="lu:Minus" />
            </Button>
            <div className="flex-1 text-center text-sm tabular-nums">
              {Math.round(zoom * 100)}%
            </div>
            <Button
              variant="outline"
              size="icon"
              aria-label="Zoom in"
              disabled={zoomIndex >= ZOOM_STEPS.length - 1}
              onClick={() => changeZoom(1)}
            >
              <Icon name="lu:Plus" />
            </Button>
          </div>

          <div className="px-2 text-xs text-gray-500 dark:text-gray-400">
            {doc ? `Page ${currentPage} of ${doc.numPages}` : "Loading…"}
          </div>

          <nav
            className="scrollbar-hidden min-h-0 flex-1 overflow-y-auto"
            aria-label="Table of contents"
          >
            {outline.map((entry, index) => (
              <button
                key={`${index}-${entry.title}`}
                onClick={() => navigate(entry)}
                className={tocItemVariants({
                  isActive: index === activeIndex,
                  isChapter: entry.level === 0,
                })}
                style={{ paddingLeft: `${16 + entry.level * 16}px` }}
              >
                {entry.title}
              </button>
            ))}
          </nav>
        </div>
      </div>
    </div>
  );
}

type LinkArea = {
  left: number;
  top: number;
  width: number;
  height: number;
  dest: Destination;
};

type PdfPageProps = {
  doc: PDFDocumentProxy;
  pageNumber: number;
  width: number;
  fallbackRatio: number;
  onNavigate: (target: Target) => void;
};

/** One page, rendered to a canvas only while it is near the viewport. */
function PdfPage({
  doc,
  pageNumber,
  width,
  fallbackRatio,
  onNavigate,
}: PdfPageProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isNear, setIsNear] = useState(false);
  const [ratio, setRatio] = useState(fallbackRatio);
  const [rendered, setRendered] = useState(false);
  const [links, setLinks] = useState<LinkArea[]>([]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    // Observe against the app's scroll container so the margin applies to it,
    // the viewport margin alone is clipped away by that container.
    const observer = new IntersectionObserver(
      ([entry]) => setIsNear(entry.isIntersecting),
      {
        root: el.closest<HTMLElement>(".overflow-y-auto"),
        rootMargin: RENDER_MARGIN,
      },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!isNear || !canvas) return;

    let cancelled = false;
    let renderTask: ReturnType<
      Awaited<ReturnType<PDFDocumentProxy["getPage"]>>["render"]
    > | null = null;

    (async () => {
      const page = await doc.getPage(pageNumber);
      if (cancelled) return;
      const base = page.getViewport({ scale: 1 });
      const viewport = page.getViewport({ scale: width / base.width });
      const pixelRatio = Math.min(
        window.devicePixelRatio || 1,
        MAX_PIXEL_RATIO,
      );

      canvas.width = Math.floor(viewport.width * pixelRatio);
      canvas.height = Math.floor(viewport.height * pixelRatio);
      setRatio(base.height / base.width);

      renderTask = page.render({
        canvas,
        viewport,
        transform:
          pixelRatio !== 1 ? [pixelRatio, 0, 0, pixelRatio, 0, 0] : undefined,
      });
      await renderTask.promise;
      if (cancelled) return;
      setRendered(true);

      const annotations = await page.getAnnotations({ intent: "display" });
      if (cancelled) return;
      setLinks(
        annotations
          .filter((a) => a.subtype === "Link" && a.dest)
          .map((a) => {
            const [x1, y1] = viewport.convertToViewportPoint(
              a.rect[0],
              a.rect[1],
            ) as number[];
            const [x2, y2] = viewport.convertToViewportPoint(
              a.rect[2],
              a.rect[3],
            ) as number[];
            return {
              left: Math.min(x1, x2),
              top: Math.min(y1, y2),
              width: Math.abs(x2 - x1),
              height: Math.abs(y2 - y1),
              dest: a.dest as Destination,
            };
          }),
      );
    })().catch(() => {
      // cancelled renders reject, nothing to report
    });

    return () => {
      cancelled = true;
      renderTask?.cancel();
      // Release the bitmap of pages that scrolled far away.
      canvas.width = 0;
      canvas.height = 0;
      setRendered(false);
      setLinks([]);
    };
  }, [doc, pageNumber, width, isNear]);

  const height = Math.round(width * ratio);

  return (
    <div
      ref={containerRef}
      data-pdf-page={pageNumber}
      className="relative shrink-0 overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm dark:border-gray-700"
      style={{ width, height }}
    >
      {!rendered && <Skeleton className="absolute inset-0 rounded-none" />}
      <canvas
        ref={canvasRef}
        className={cn(
          "block transition-opacity duration-200",
          rendered ? "opacity-100" : "opacity-0",
        )}
        style={{ width, height }}
      />
      {links.map((link, i) => (
        <button
          key={i}
          aria-label="Go to section"
          className="absolute rounded-sm hover:bg-blue-500/10"
          style={{
            left: link.left,
            top: link.top,
            width: link.width,
            height: link.height,
          }}
          onClick={async () => {
            const target = await resolveTarget(doc, link.dest);
            if (target) onNavigate(target);
          }}
        />
      ))}
    </div>
  );
}
