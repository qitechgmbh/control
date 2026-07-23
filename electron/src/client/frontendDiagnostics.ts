const STORAGE_KEY = "qitech.frontendDiagnostics";
const MAX_ENTRIES = 300;
const EVENT_LOOP_POLL_MS = 250;
const EVENT_LOOP_STALL_THRESHOLD_MS = 500;
const EVENT_LOOP_STALL_LOG_COOLDOWN_MS = 2_000;

type Details = Record<string, unknown>;

export type FrontendDiagnosticEntry = {
  at: string;
  uptimeMs: number;
  name: string;
  details?: Details;
};

interface PerformanceMemory {
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
}

function readEntries(): FrontendDiagnosticEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function frontendDiagnostic(name: string, details?: Details): void {
  const entry: FrontendDiagnosticEntry = {
    at: new Date().toISOString(),
    uptimeMs: Math.round(performance.now()),
    name,
    details,
  };

  const entries = [...readEntries(), entry].slice(-MAX_ENTRIES);
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // Keep diagnostics non-invasive; never let logging affect machine UI.
  }

  if (name.includes("disconnect") || name.includes("error")) {
    console.warn("[frontend-diagnostics]", entry);
  } else {
    console.debug("[frontend-diagnostics]", entry);
  }
}

export function installFrontendDiagnostics(): () => void {
  frontendDiagnostic("renderer.boot", {
    userAgent: navigator.userAgent,
    href: window.location.href,
  });

  const api = {
    clear: () => localStorage.removeItem(STORAGE_KEY),
    read: readEntries,
  };
  Object.assign(window, { qitechFrontendDiagnostics: api });

  let expected = performance.now() + EVENT_LOOP_POLL_MS;
  let lastStallLog = 0;
  const eventLoopInterval = window.setInterval(() => {
    const now = performance.now();
    const delayMs = now - expected;
    expected = now + EVENT_LOOP_POLL_MS;

    if (
      delayMs < EVENT_LOOP_STALL_THRESHOLD_MS ||
      now - lastStallLog < EVENT_LOOP_STALL_LOG_COOLDOWN_MS
    ) {
      return;
    }

    lastStallLog = now;
    const perf = performance as Performance & { memory?: PerformanceMemory };
    frontendDiagnostic("renderer.event_loop_stall", {
      delayMs: Math.round(delayMs),
      href: window.location.href,
      heapUsedMb: perf.memory
        ? Math.round(perf.memory.usedJSHeapSize / 1_048_576)
        : null,
      heapLimitMb: perf.memory
        ? Math.round(perf.memory.jsHeapSizeLimit / 1_048_576)
        : null,
    });
  }, EVENT_LOOP_POLL_MS);

  return () => {
    window.clearInterval(eventLoopInterval);
    frontendDiagnostic("renderer.unmount");
  };
}
