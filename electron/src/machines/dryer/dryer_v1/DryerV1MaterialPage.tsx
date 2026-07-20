import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { Icon } from "@/components/Icon";
import { Input } from "@/components/ui/input";
import { dryerV1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerV1SerialRoute } from "@/routes/routes";
import { useMachineMutate } from "@/client/useClient";
import { MATERIAL_PRESETS } from "../materialPresets";
import { useDryerV1MaterialStore } from "../materialStore";
import React, { useMemo, useState } from "react";
import { z } from "zod";

export function DryerV1MaterialPage() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const [search, setSearch] = useState("");
  const {
    favorites,
    selectedAbbrev,
    throughput,
    selectMaterial,
    toggleFavorite,
  } = useDryerV1MaterialStore();
  const { request: sendMutation } = useMachineMutate(z.any());

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    const list = !q
      ? MATERIAL_PRESETS
      : MATERIAL_PRESETS.filter(
          (p) =>
            p.abbrev.toLowerCase().includes(q) ||
            p.name.toLowerCase().includes(q),
        );
    return [...list].sort((a, b) => a.abbrev.localeCompare(b.abbrev));
  }, [search]);

  const handleSelect = (abbrev: string) => {
    selectMaterial(abbrev);
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: {
        ApplyMaterialPreset: { abbrev, throughput_kg_per_h: throughput },
      },
    });
  };

  return (
    <Page>
      <ControlCard title="Material List">
        <div className="relative">
          <Icon
            name="lu:Search"
            className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-gray-400"
          />
          <Input
            placeholder="Search material..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>

        <div className="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
          {filtered.map((p) => {
            const isSelected = selectedAbbrev === p.abbrev;
            const isFav = favorites.includes(p.abbrev);
            return (
              <div
                key={p.abbrev}
                onClick={() => handleSelect(p.abbrev)}
                className={[
                  "flex cursor-pointer flex-col gap-1 rounded-xl border p-3 hover:bg-gray-50",
                  isSelected
                    ? "border-blue-400 bg-blue-50"
                    : "border-gray-200",
                ].join(" ")}
              >
                <div className="flex items-center justify-between">
                  <span className="font-mono text-sm font-bold text-gray-800">
                    {p.abbrev}
                  </span>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      toggleFavorite(p.abbrev);
                    }}
                  >
                    <Icon
                      name="lu:Star"
                      className={[
                        "size-4",
                        isFav
                          ? "fill-yellow-400 text-yellow-400"
                          : "text-gray-200",
                      ].join(" ")}
                    />
                  </button>
                </div>
                <span className="truncate text-sm text-gray-500">
                  {p.name}
                </span>
                <div className="flex items-center justify-between text-xs text-gray-400">
                  <span>
                    {p.temp_min === p.temp_max
                      ? `${p.temp_min}°C`
                      : `${p.temp_min}–${p.temp_max}°C`}
                  </span>
                  <span>
                    {p.drying_time_min === p.drying_time_max
                      ? `${p.drying_time_min} h`
                      : `${p.drying_time_min}–${p.drying_time_max} h`}
                  </span>
                  <span>{p.specific_air_volume.toFixed(2)} m³/kg</span>
                </div>
              </div>
            );
          })}
        </div>
      </ControlCard>
    </Page>
  );
}
