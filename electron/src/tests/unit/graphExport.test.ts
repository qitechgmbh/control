import { describe, expect, it } from "vitest";
import * as XLSX from "xlsx";
import {
  buildExcelWorkbook,
  statistics,
  type ExcelReport,
} from "@/components/graph/export/excelWorkbook";
import {
  currentExportRange,
  exportColumns,
  type GraphExportSource,
} from "@/components/graph/export/historyExport";

const machine = {
  machine_identification: { vendor: 1, machine: 4 },
  serial: 1,
};

function source(partial: Partial<GraphExportSource>): GraphExportSource {
  return {
    machine,
    config: { title: "Graph" },
    series: [],
    visibleRange: () => null,
    ...partial,
  };
}

describe("statistics", () => {
  it("handles long series without overflowing the stack", () => {
    const values = Array.from({ length: 500_000 }, (_, i) => i);
    const stats = statistics(values);
    expect(stats.min).toBe(0);
    expect(stats.max).toBe(499_999);
    expect(stats.mean).toBeCloseTo(249_999.5);
    expect(stats.p50).toBe(250_000);
  });

  it("ignores non-finite values", () => {
    const stats = statistics([1, NaN, 3, Infinity]);
    expect(stats.min).toBe(1);
    expect(stats.max).toBe(3);
    expect(stats.mean).toBe(2);
  });
});

describe("exportColumns", () => {
  it("collects measurements and targets once per path, with units and scale", () => {
    const sources = [
      source({
        config: { title: "Nozzle" },
        unit: "C",
        series: [
          {
            newData: null,
            history: "heating.nozzle.temperature",
            lines: [
              {
                type: "target",
                value: 200,
                color: "#000",
                history: "heating.nozzle.target_temperature",
              },
            ],
          },
        ],
      }),
      source({
        config: { title: "Screw" },
        series: [
          { newData: null, history: "screw.rpm", title: "Speed" },
          {
            newData: null,
            history: "screw.load",
            title: "Load",
            historyScale: 100,
          },
          // already exported by the first graph
          { newData: null, history: "heating.nozzle.temperature" },
          // live only: no database column
          { newData: null, title: "Live" },
        ],
      }),
    ];

    const [columns] = exportColumns(sources);
    expect(columns.measurements).toEqual([
      { name: "heating.nozzle.temperature", label: "Nozzle [°C]", scale: 1 },
      { name: "screw.rpm", label: "Screw – Speed", scale: 1 },
      { name: "screw.load", label: "Screw – Load", scale: 100 },
    ]);
    expect(columns.targets).toEqual([
      {
        name: "heating.nozzle.target_temperature",
        label: "Nozzle [°C] (target)",
        scale: 1,
      },
    ]);
  });

  it("skips graphs without a machine", () => {
    expect(
      exportColumns([
        source({ machine: null, series: [{ newData: null, history: "a" }] }),
      ]),
    ).toEqual([]);
  });
});

describe("currentExportRange", () => {
  it("takes the first graph that has been drawn", () => {
    expect(
      currentExportRange([
        source({}),
        source({ visibleRange: () => ({ min: 10, max: 20 }) }),
      ]),
    ).toEqual({ min: 10, max: 20 });
  });
});

describe("buildExcelWorkbook", () => {
  const t0 = Date.UTC(2026, 10, 23, 12, 0, 0);
  const report: ExcelReport = {
    title: "Extruder",
    range: { min: t0, max: t0 + 3_000 },
    bucketMs: 0,
    series: [
      {
        graphTitle: "Nozzle",
        title: "Nozzle",
        unit: "°C",
        points: { t: [t0, t0 + 1_000, t0 + 2_000], v: [190, 195, 200] },
        target: { t: [t0 - 5_000, t0 + 1_500], v: [180, 200] },
      },
      {
        graphTitle: "Screw",
        title: "Speed",
        unit: "rpm",
        points: { t: [t0 + 1_000], v: [30] },
      },
    ],
    markers: [{ timestamp: t0 + 900, name: "Start" }],
  };

  const workbook = XLSX.read(buildExcelWorkbook(report), {
    type: "buffer",
    cellNF: true,
  });
  const rows = (name: string) =>
    XLSX.utils.sheet_to_json<unknown[]>(workbook.Sheets[name], {
      header: 1,
      raw: true,
      defval: null,
    });

  it("has an overview, the combined sheet and one sheet per series", () => {
    expect(workbook.SheetNames).toEqual([
      "Overview",
      "All values",
      "Nozzle",
      "Speed",
    ]);
  });

  it("writes numbers and dates as typed cells", () => {
    const sheet = workbook.Sheets["Nozzle"];
    expect(sheet.A2.t).toBe("n");
    expect(sheet.A2.z).toBe("yyyy-mm-dd hh:mm:ss.000");
    expect(sheet.B2).toMatchObject({ t: "n", v: 190 });
  });

  it("fills the target column with the step in effect", () => {
    expect(rows("Nozzle").map((r) => r[2])).toEqual(["Target", 180, 180, 200]);
  });

  it("puts every series on one time axis, with markers", () => {
    const all = rows("All values");
    expect(all[0]).toEqual([
      "Time",
      "Seconds",
      "Nozzle [°C]",
      "Speed [rpm]",
      "Marker",
    ]);
    expect(all.slice(1).map((r) => r.slice(1))).toEqual([
      [0, 190, null, null],
      [1, 195, 30, "Start"],
      [2, 200, null, null],
    ]);
  });

  it("summarises each series in the overview", () => {
    const stats = rows("Overview").find((r) => r[1] === "Nozzle");
    expect(stats?.slice(2, 7)).toEqual(["°C", 3, 190, 200, 195]);
  });

  it("names sheets after the graph when series titles repeat", () => {
    const repeated = (graphTitle: string, unit: string) => ({
      graphTitle,
      title: "Nozzle",
      unit,
      points: { t: [t0], v: [1] },
    });
    const book = XLSX.read(
      buildExcelWorkbook({
        ...report,
        series: [
          repeated("Temperatures", "°C"),
          repeated("Power Outputs", "W"),
        ],
      }),
      { type: "buffer" },
    );
    expect(book.SheetNames).toEqual([
      "Overview",
      "All values",
      "Temperatures – Nozzle",
      "Power Outputs – Nozzle",
    ]);
  });

  it("refuses a range without data", () => {
    expect(() =>
      buildExcelWorkbook({
        ...report,
        series: [{ ...report.series[0], points: { t: [], v: [] } }],
      }),
    ).toThrow("No data");
  });
});
