import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { Terminal } from "@/components/Terminal";

// Mock the Icon component
vi.mock("@/components/Icon", () => ({
  Icon: ({ name }: { name: string }) => <span data-testid={`icon-${name}`} />,
}));

describe("Terminal", () => {
  const mockLines = [
    "2025-01-31 10:00:00 INFO: Application started",
    "2025-01-31 10:00:01 DEBUG: Loading configuration",
    "2025-01-31 10:00:02 WARN: Configuration file not found, using defaults",
    "2025-01-31 10:00:03 ERROR: Failed to connect to database",
  ];

  it("renders terminal with default props", () => {
    render(<Terminal lines={mockLines} />);
    
    expect(screen.getByText("Terminal")).toBeInTheDocument();
    expect(screen.getByText("4 / 4 lines")).toBeInTheDocument();
  });

  it("renders terminal with custom title", () => {
    render(<Terminal lines={mockLines} title="Custom Terminal" />);
    
    expect(screen.getByText("Custom Terminal")).toBeInTheDocument();
  });

  it("shows performance warning when lines exceed maxLines", () => {
    const manyLines = Array.from({ length: 6000 }, (_, i) => `Line ${i + 1}`);
    render(<Terminal lines={manyLines} maxLines={5000} />);
    
    expect(screen.getByText("(showing last 5000 for performance)")).toBeInTheDocument();
    expect(screen.getByText("5000 / 6000 lines")).toBeInTheDocument();
  });

  it("renders copy and export buttons", () => {
    render(<Terminal lines={mockLines} exportPrefix="test" />);
    
    expect(screen.getByText("Copy")).toBeInTheDocument();
    expect(screen.getByText("Export")).toBeInTheDocument();
  });

  it("does not show export button when no exportPrefix is provided", () => {
    render(<Terminal lines={mockLines} />);
    
    expect(screen.getByText("Copy")).toBeInTheDocument();
    expect(screen.queryByText("Export")).not.toBeInTheDocument();
  });

  it("displays auto-scroll status", () => {
    render(<Terminal lines={mockLines} autoScroll={true} />);
    
    expect(screen.getByText(/Auto-scroll enabled/)).toBeInTheDocument();
  });

  it("displays disabled auto-scroll status", () => {
    render(<Terminal lines={mockLines} autoScroll={false} />);
    
    expect(screen.getByText(/Auto-scroll disabled/)).toBeInTheDocument();
  });

  it("renders all provided lines when under maxLines limit", () => {
    const testLines = ["Line 1", "Line 2", "Line 3"];
    render(<Terminal lines={testLines} maxLines={1000} />);
    
    testLines.forEach(line => {
      expect(screen.getByText(line)).toBeInTheDocument();
    });
  });

  it("handles empty lines array", () => {
    render(<Terminal lines={[]} />);
    
    expect(screen.getByText("0 / 0 lines")).toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = render(<Terminal lines={mockLines} className="custom-height" />);
    
    expect(container.firstChild).toHaveClass("custom-height");
  });
});