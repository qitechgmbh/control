import { Page } from "@/components/Page";
import { Icon } from "@/components/Icon";
import { Input } from "@/components/ui/input";
import React, { useMemo, useState } from "react";
import { MATERIAL_PRESETS } from "./materialPresets";
import { useDryerMaterialStore } from "./dryerMaterialStore";
import { useDryerV1 } from "./useDryerV1";

function formatTempRange(min: number, max: number): string {
  return min === max ? `${min}` : `${min}–${max}`;
}

function formatTimeRange(min: number, max: number): string {
  return min === max ? `${min}` : `${min}–${max}`;
}

export function DryerV1MaterialPage() {
  const { applyMaterialPreset } = useDryerV1();

  const [search, setSearch] = useState("");
  const [applyingAbbrev, setApplyingAbbrev] = useState<string | null>(null);
  const { favorites, selectedAbbrev, throughput, toggleFavorite, selectMaterial } =
    useDryerMaterialStore();

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return MATERIAL_PRESETS;
    return MATERIAL_PRESETS.filter(
      (p) =>
        p.abbrev.toLowerCase().includes(q) ||
        p.name.toLowerCase().includes(q),
    );
  }, [search]);

  const handleSelect = (abbrev: string) => {
    selectMaterial(abbrev);
    setApplyingAbbrev(abbrev);
    applyMaterialPreset(abbrev, throughput);
    setApplyingAbbrev(null);
  };

  return (
    <Page>
      <div className="flex h-full flex-col gap-4 p-4">
        {/* Toolbar */}
        <div className="flex items-center gap-3">
          <div className="relative flex-1">
            <Icon
              name="lu:Search"
              className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-gray-400"
            />
            <Input
              placeholder="Search material..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-11 pl-9 text-base"
            />
          </div>
          <span className="text-sm text-gray-400">
            {filtered.length} / {MATERIAL_PRESETS.length}
          </span>
        </div>

        {/* Table */}
        <div className="flex-1 overflow-auto rounded-2xl border border-gray-200 bg-white shadow-sm">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-gray-50 text-xs font-semibold uppercase tracking-wide text-gray-500">
              <tr>
                <th className="px-4 py-3 text-left">Abbr.</th>
                <th className="px-4 py-3 text-left">Material</th>
                <th className="px-4 py-3 text-right">
                  Density
                  <span className="ml-1 font-normal normal-case text-gray-400">
                    g/cm³
                  </span>
                </th>
                <th className="px-4 py-3 text-right">
                  Drying Temp.
                  <span className="ml-1 font-normal normal-case text-gray-400">
                    °C
                  </span>
                </th>
                <th className="px-4 py-3 text-right">
                  Drying Time
                  <span className="ml-1 font-normal normal-case text-gray-400">
                    h
                  </span>
                </th>
                <th className="px-4 py-3 text-right">
                  Spec. Air Vol.
                  <span className="ml-1 font-normal normal-case text-gray-400">
                    m³/kg
                  </span>
                </th>
                <th className="px-4 py-3 text-center">Favourite</th>
                <th className="px-4 py-3 text-center">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {filtered.map((p) => {
                const isSelected = selectedAbbrev === p.abbrev;
                const isFav = favorites.includes(p.abbrev);
                return (
                  <tr
                    key={p.abbrev}
                    className={
                      isSelected ? "bg-gray-900 text-white" : "hover:bg-gray-50"
                    }
                  >
                    <td className="px-4 py-3">
                      <span
                        className={[
                          "font-mono font-bold",
                          isSelected ? "text-white" : "text-gray-800",
                        ].join(" ")}
                      >
                        {p.abbrev}
                      </span>
                    </td>
                    <td
                      className={[
                        "max-w-xs truncate px-4 py-3",
                        isSelected ? "text-gray-300" : "text-gray-600",
                      ].join(" ")}
                    >
                      {p.name}
                    </td>
                    <td
                      className={[
                        "px-4 py-3 text-right tabular-nums",
                        isSelected ? "text-gray-300" : "text-gray-500",
                      ].join(" ")}
                    >
                      {p.bulk_density.toFixed(2)}
                    </td>
                    <td
                      className={[
                        "px-4 py-3 text-right tabular-nums",
                        isSelected ? "text-gray-300" : "text-gray-500",
                      ].join(" ")}
                    >
                      {formatTempRange(p.temp_min, p.temp_max)}
                    </td>
                    <td
                      className={[
                        "px-4 py-3 text-right tabular-nums",
                        isSelected ? "text-gray-300" : "text-gray-500",
                      ].join(" ")}
                    >
                      {formatTimeRange(p.drying_time_min, p.drying_time_max)}
                    </td>
                    <td
                      className={[
                        "px-4 py-3 text-right tabular-nums",
                        isSelected ? "text-gray-300" : "text-gray-500",
                      ].join(" ")}
                    >
                      {p.specific_air_volume.toFixed(2)}
                    </td>
                    <td className="px-4 py-3 text-center">
                      <button
                        onClick={() => toggleFavorite(p.abbrev)}
                        className="transition-transform hover:scale-110"
                        aria-label={isFav ? "Remove from favorites" : "Add to favorites"}
                      >
                        <Icon
                          name={isFav ? "lu:Star" : "lu:Star"}
                          className={[
                            "size-5",
                            isFav
                              ? isSelected
                                ? "fill-yellow-300 text-yellow-300"
                                : "fill-yellow-400 text-yellow-400"
                              : isSelected
                                ? "text-gray-500"
                                : "text-gray-300",
                          ].join(" ")}
                        />
                      </button>
                    </td>
                    <td className="px-4 py-3 text-center">
                      <button
                        onClick={() => handleSelect(p.abbrev)}
                        disabled={applyingAbbrev === p.abbrev}
                        className={[
                          "rounded-lg px-4 py-1.5 text-sm font-semibold transition-colors",
                          isSelected
                            ? "bg-white text-gray-900"
                            : "bg-gray-900 text-white hover:bg-gray-700",
                        ].join(" ")}
                      >
                        {applyingAbbrev === p.abbrev ? "..." : "Select"}
                      </button>
                    </td>
                  </tr>
                );
              })}
              {filtered.length === 0 && (
                <tr>
                  <td
                    colSpan={8}
                    className="py-12 text-center text-sm text-gray-400"
                  >
                    No material found
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </Page>
  );
}
