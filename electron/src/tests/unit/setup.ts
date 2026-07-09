import "@testing-library/jest-dom";

// jsdom has no ResizeObserver; Chart.js's `responsive: true` option needs one
// to bind its own auto-resize handling. Electron's real Chromium renderer
// has this natively — this stub exists only to satisfy the test environment.
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class ResizeObserver {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  };
}
