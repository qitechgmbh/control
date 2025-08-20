import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useLocalLogStreaming } from "@/hooks/useLocalLogStreaming";
import { useLogsStore } from "@/stores/logsStore";

// Mock the troubleshoot helpers
vi.mock("@/helpers/troubleshoot_helpers", () => ({
  startLogStream: vi.fn(),
  stopLogStream: vi.fn(),
  setupLogDataListener: vi.fn(),
  cleanupLogDataListener: vi.fn(),
  isTroubleshootAvailable: vi.fn(),
}));

// Mock the logs store
vi.mock("@/stores/logsStore", () => ({
  useLogsStore: vi.fn(),
}));

describe("useLocalLogStreaming", () => {
  const mockSetStreaming = vi.fn();
  const mockAddLogEntry = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    
    // Setup default mock returns
    (useLogsStore as any).mockReturnValue({
      isStreaming: false,
      setStreaming: mockSetStreaming,
      addLogEntry: mockAddLogEntry,
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("should initialize with correct default state", () => {
    const { result } = renderHook(() => useLocalLogStreaming());

    expect(result.current.isStreaming).toBe(false);
    expect(result.current.error).toBe(null);
    expect(typeof result.current.startStreaming).toBe("function");
    expect(typeof result.current.stopStreaming).toBe("function");
  });

  it("should handle successful stream start", async () => {
    const { startLogStream, isTroubleshootAvailable } = await import("@/helpers/troubleshoot_helpers");
    
    (isTroubleshootAvailable as any).mockReturnValue(true);
    (startLogStream as any).mockResolvedValue({ success: true });

    const { result } = renderHook(() => useLocalLogStreaming());

    await act(async () => {
      const success = await result.current.startStreaming();
      expect(success).toBe(true);
    });

    expect(startLogStream).toHaveBeenCalledOnce();
    expect(mockSetStreaming).toHaveBeenCalledWith(true);
  });

  it("should handle failed stream start", async () => {
    const { startLogStream, isTroubleshootAvailable } = await import("@/helpers/troubleshoot_helpers");
    
    (isTroubleshootAvailable as any).mockReturnValue(true);
    (startLogStream as any).mockResolvedValue({ success: false, error: "Test error" });

    const { result } = renderHook(() => useLocalLogStreaming());

    await act(async () => {
      const success = await result.current.startStreaming();
      expect(success).toBe(false);
    });

    expect(result.current.error).toBe("Test error");
  });

  it("should handle unavailable troubleshoot context", async () => {
    const { isTroubleshootAvailable } = await import("@/helpers/troubleshoot_helpers");
    
    (isTroubleshootAvailable as any).mockReturnValue(false);

    const { result } = renderHook(() => useLocalLogStreaming());

    await act(async () => {
      const success = await result.current.startStreaming();
      expect(success).toBe(false);
    });

    expect(result.current.error).toBe("Troubleshoot context not available");
  });

  it("should handle successful stream stop", async () => {
    const { stopLogStream, isTroubleshootAvailable } = await import("@/helpers/troubleshoot_helpers");
    
    (isTroubleshootAvailable as any).mockReturnValue(true);
    (stopLogStream as any).mockResolvedValue({ success: true });

    // Mock initial streaming state
    (useLogsStore as any).mockReturnValue({
      isStreaming: true,
      setStreaming: mockSetStreaming,
      addLogEntry: mockAddLogEntry,
    });

    const { result } = renderHook(() => useLocalLogStreaming());

    await act(async () => {
      const success = await result.current.stopStreaming();
      expect(success).toBe(true);
    });

    expect(stopLogStream).toHaveBeenCalledOnce();
    expect(mockSetStreaming).toHaveBeenCalledWith(false);
  });
});