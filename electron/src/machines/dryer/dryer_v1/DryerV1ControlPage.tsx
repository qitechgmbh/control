import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { SelectionGroup } from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { dryerV1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerV1SerialRoute } from "@/routes/routes";
import { useDryerV1Namespace } from "./dryerV1Namespace";
import { useMachineMutate } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { Icon } from "@/components/Icon";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { z } from "zod";
import { MATERIAL_PRESETS, recommendedTemp } from "../materialPresets";
import { useDryerV1MaterialStore } from "../materialStore";

function isRunningStatus(status: number): boolean {
  return status !== 1 && status !== 5 && status !== 6;
}

type StatusInfo = {
  label: string;
  icon: string;
};

function getMachineStatusInfo(status: number): StatusInfo {
  switch (status) {
    case 0:
      return { label: "Starting", icon: "lu:Power" };
    case 1:
      return { label: "Standby", icon: "lu:CirclePause" };
    case 2:
      return { label: "Heating", icon: "lu:Flame" };
    case 3:
      return { label: "Starting Up", icon: "lu:Settings" };
    case 4:
      return { label: "Drying", icon: "lu:Wind" };
    case 5:
      return { label: "Cooling", icon: "lu:Snowflake" };
    case 6:
      return { label: "Fan Stop", icon: "lu:Fan" };
    case 7:
      return { label: "Finishing", icon: "lu:Snowflake" };
    case 9:
      return { label: "Switching", icon: "lu:RefreshCw" };
    default:
      return { label: `Status ${status}`, icon: "lu:HelpCircle" };
  }
}

function getAlarmMessage(code: number): string | null {
  switch (code) {
    case 0:
      return null;
    case 1:
      return "Local control active";
    case 2:
      return "Keyboard error";
    case 3:
      return "Sensor break T1";
    case 4:
      return "Sensor break T2";
    case 5:
      return "Sensor break T3";
    case 6:
      return "Sensor break T4";
    case 7:
      return "Sensor break T5";
    case 8:
      return "Sensor break T6";
    case 9:
      return "Process temp exceeded";
    case 11:
      return "Thermal protection regen";
    case 12:
      return "Thermal protection fan";
    case 14:
      return "Fan over-temperature";
    case 15:
      return "Check process fan";
    default:
      return `Alarm ${code}`;
  }
}

function getWarningMessage(code: number): string | null {
  switch (code) {
    case 0:
      return null;
    case 1:
      return "Warning 36";
    case 2:
      return "Filter cleaning required (pressure)";
    case 3:
      return "Filter cleaning required (time)";
    case 4:
      return "High dew point";
    case 5:
      return "Low process temperature";
    case 6:
      return "Slave dryer warning";
    case 7:
      return "MPM temperature reduced";
    case 8:
      return "MPM dryer in standby";
    default:
      return `Warning ${code}`;
  }
}

function formatRemaining(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}min ${s}s`;
}

function encodedToMinutes(encoded: number): number {
  return Math.floor(encoded / 100) * 60 + (encoded % 100);
}

function formatMinutes(totalMins: number): string {
  const h = Math.floor(totalMins / 60);
  const m = totalMins % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

export function DryerV1ControlPage() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const { liveValues, ts_temp_process } =
    useDryerV1Namespace(machineIdentification);
  const v = liveValues?.data;

  const {
    targetTimeMin,
    setTargetTimeMin,
    selectedAbbrev: selectedMaterialAbbrev,
  } = useDryerV1MaterialStore();
  const timerStartRef = useRef<number | null>(null);
  const [remainingSec, setRemainingSec] = useState<number | null>(null);
  const prevStatusRef = useRef<number | undefined>(undefined);
  const autoStopFiredRef = useRef(false);

  const scheduleRef = useRef(v?.schedule);
  scheduleRef.current = v?.schedule;
  const targetTimeMinRef = useRef(targetTimeMin);
  targetTimeMinRef.current = targetTimeMin;

  const scheduledStopMins = useMemo(() => {
    if (!v?.schedule) return null;
    const idx = (new Date().getDay() + 6) % 7; // JS 0=Sun -> convert to 0=Mon
    const today = v.schedule[idx];
    if (!today?.stop_time) return null;
    const stopMins = encodedToMinutes(today.stop_time);
    const nowMins = new Date().getHours() * 60 + new Date().getMinutes();
    return stopMins > nowMins ? stopMins : null;
  }, [v?.schedule]);

  const scheduleControlled = scheduledStopMins !== null;

  useEffect(() => {
    if (v?.status === undefined) return;
    const wasRunning =
      prevStatusRef.current !== undefined &&
      isRunningStatus(prevStatusRef.current);
    const nowRunning = isRunningStatus(v.status);
    if (!wasRunning && nowRunning) {
      timerStartRef.current = Date.now();
      autoStopFiredRef.current = false;
    } else if (wasRunning && !nowRunning) {
      timerStartRef.current = null;
      setRemainingSec(null);
    }
    prevStatusRef.current = v.status;
  }, [v?.status]);

  const handleStartStopRef = useRef<(next: boolean) => void>(() => {});
  useEffect(() => {
    const id = setInterval(() => {
      const schedule = scheduleRef.current;

      if (schedule) {
        const idx = (new Date().getDay() + 6) % 7;
        const today = schedule[idx];
        if (today?.stop_time) {
          const now = new Date();
          const stopSec = encodedToMinutes(today.stop_time) * 60;
          const nowSec =
            now.getHours() * 3600 + now.getMinutes() * 60 + now.getSeconds();
          if (stopSec > nowSec) {
            const remaining = stopSec - nowSec;
            setRemainingSec(remaining);
            if (remaining <= 1 && !autoStopFiredRef.current) {
              autoStopFiredRef.current = true;
              handleStartStopRef.current(false);
            }
            return;
          }
        }
      }

      if (timerStartRef.current !== null) {
        const elapsedSec = (Date.now() - timerStartRef.current) / 1000;
        const remaining = Math.max(
          0,
          targetTimeMinRef.current * 60 - elapsedSec,
        );
        setRemainingSec(remaining);
        if (remaining === 0 && !autoStopFiredRef.current) {
          autoStopFiredRef.current = true;
          timerStartRef.current = null;
          handleStartStopRef.current(false);
        }
      }
    }, 1000);
    return () => clearInterval(id);
  }, []);

  const isRunning = v !== undefined && isRunningStatus(v.status);
  const machineStatusInfo =
    v !== undefined ? getMachineStatusInfo(v.status) : null;

  const {
    value: isRunningOptimistic,
    setOptimistic: setRunningOptimistic,
    setReal: setRunningReal,
  } = useStateOptimistic<boolean>();
  const displayRunning = isRunningOptimistic ?? isRunning;
  const {
    value: displayTargetTemp,
    setOptimistic: setTargetTempOptimistic,
    setReal: setTargetTempReal,
  } = useStateOptimistic<number>();

  useEffect(() => {
    if (v?.status !== undefined) setRunningReal(isRunningStatus(v.status));
  }, [v?.status, setRunningReal]);

  useEffect(() => {
    if (v?.target_temperature !== undefined)
      setTargetTempReal(v.target_temperature);
  }, [v?.target_temperature, setTargetTempReal]);

  useEffect(() => {
    if (!selectedMaterialAbbrev) return;
    const preset = MATERIAL_PRESETS.find(
      (p) => p.abbrev === selectedMaterialAbbrev,
    );
    if (preset) setTargetTempOptimistic(recommendedTemp(preset));
  }, [selectedMaterialAbbrev, setTargetTempOptimistic]);

  const { request: sendMutation, isLoading: mutating } = useMachineMutate(
    z.any(),
  );

  const handleStartStop = (next: boolean) => {
    setRunningOptimistic(next);
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { SetStartStop: next },
    });
  };
  handleStartStopRef.current = handleStartStop;

  const handleTargetTemp = (temp: number) => {
    const rounded = Math.round(Math.max(50, Math.min(180, temp)));
    setTargetTempOptimistic(rounded);
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { SetTargetTemperature: rounded },
    });
  };

  return (
    <Page>
      <ControlGrid columns={3}>
        <ControlCard title="Timer">
          <div className="flex items-center gap-1.5 text-sm text-gray-400">
            <Icon name="lu:Clock" className="size-4" />
            <span>Remaining Time</span>
          </div>
          <div className="text-3xl font-bold">
            {remainingSec !== null ? formatRemaining(remainingSec) : "—"}
          </div>
          {scheduleControlled ? (
            <div className="flex items-center gap-1.5 rounded-lg border border-blue-200 bg-blue-50 px-3 py-2 text-sm font-semibold text-blue-700">
              <Icon name="lu:CalendarClock" className="size-4" />
              <span>Schedule: stops at {formatMinutes(scheduledStopMins!)}</span>
            </div>
          ) : (
            <Label label="Target Time">
              <EditValue
                title="Target Time"
                value={targetTimeMin}
                min={1}
                max={1440}
                step={5}
                renderValue={(val) => `${val.toFixed(0)} min`}
                onChange={setTargetTimeMin}
              />
            </Label>
          )}
        </ControlCard>

        <ControlCard title="Temperature">
          <TimeSeriesValueNumeric
            label="Current Temperature"
            unit="C"
            timeseries={ts_temp_process}
            renderValue={(val) => val.toFixed(1)}
          />
          <Label label="Target Temperature">
            <EditValue
              title="Target Temperature"
              value={displayTargetTemp}
              min={50}
              max={180}
              step={1}
              disabled={v === undefined}
              renderValue={(val) => val.toFixed(0)}
              unit="C"
              onChange={handleTargetTemp}
            />
          </Label>
        </ControlCard>

        <MaterialListeCard machineIdentification={machineIdentification} />

        <ControlCard title="Mode" className="min-h-[280px]">
          <SelectionGroup<"Standby" | "ON">
            value={displayRunning ? "ON" : "Standby"}
            loading={mutating}
            disabled={v === undefined}
            className="grid flex-1 grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              ON: {
                children: "ON",
                icon: "lu:CirclePlay",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={(val) => handleStartStop(val === "ON")}
          />
          {machineStatusInfo && (
            <div className="flex items-center gap-2 rounded-xl border border-gray-200 bg-gray-50 px-3 py-2 text-sm font-semibold text-gray-700">
              <Icon name={machineStatusInfo.icon as any} className="size-4" />
              <span>{machineStatusInfo.label}</span>
            </div>
          )}
          {v?.alarm ? (
            <StatusBadge variant="error">{getAlarmMessage(v.alarm)}</StatusBadge>
          ) : null}
          {v?.warning ? (
            <StatusBadge variant="warning">
              {getWarningMessage(v.warning)}
            </StatusBadge>
          ) : null}
        </ControlCard>

        <FlowRateCard machineIdentification={machineIdentification} />
      </ControlGrid>
    </Page>
  );
}

function MaterialListeCard({
  machineIdentification,
}: {
  machineIdentification: MachineIdentificationUnique;
}) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const {
    favorites,
    recentlyUsed,
    selectedAbbrev,
    throughput,
    selectMaterial,
    clearMaterial,
    toggleFavorite,
  } = useDryerV1MaterialStore();
  const { request: sendMutation } = useMachineMutate(z.any());

  const handleSelect = (abbrev: string) => {
    selectMaterial(abbrev);
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: {
        ApplyMaterialPreset: { abbrev, throughput_kg_per_h: throughput },
      },
    });
    setOpen(false);
    setSearch("");
  };

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return MATERIAL_PRESETS;
    return MATERIAL_PRESETS.filter(
      (p) =>
        p.abbrev.toLowerCase().includes(q) || p.name.toLowerCase().includes(q),
    );
  }, [search]);

  const favPresets = MATERIAL_PRESETS.filter((p) =>
    favorites.includes(p.abbrev),
  );
  const recentPresets = recentlyUsed
    .map((a) => MATERIAL_PRESETS.find((p) => p.abbrev === a))
    .filter(Boolean) as typeof MATERIAL_PRESETS;

  const selectedPreset = selectedAbbrev
    ? MATERIAL_PRESETS.find((p) => p.abbrev === selectedAbbrev)
    : null;

  return (
    <ControlCard title="Material List" height={2}>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <button className="flex w-full items-center justify-between rounded-xl border border-gray-200 bg-gray-50 px-4 py-3 text-left hover:bg-gray-100">
            <span className="flex-1 truncate text-sm font-semibold text-gray-700">
              {selectedPreset
                ? `${selectedPreset.abbrev} — ${selectedPreset.name}`
                : recentPresets.length > 0
                  ? recentPresets
                      .slice(0, 3)
                      .map((p) => p.abbrev)
                      .join(", ")
                  : "Select material..."}
            </span>
            <div className="ml-2 flex items-center gap-1">
              {selectedPreset && (
                <span
                  role="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    clearMaterial();
                  }}
                  className="rounded-full p-0.5 hover:bg-gray-200"
                >
                  <Icon name="lu:X" className="size-4 text-gray-400" />
                </span>
              )}
              <Icon name="lu:ChevronDown" className="size-4 text-gray-400" />
              <Icon name="lu:Search" className="size-4 text-gray-400" />
            </div>
          </button>
        </PopoverTrigger>
        <PopoverContent className="w-80 p-0" align="start">
          <div className="border-b border-gray-100 p-2">
            <div className="relative">
              <Icon
                name="lu:Search"
                className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-gray-400"
              />
              <Input
                placeholder="Search"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="pl-9"
                autoFocus
              />
            </div>
          </div>
          <div className="max-h-80 overflow-y-auto">
            {search === "" ? (
              <>
                {favPresets.length > 0 && (
                  <Section label="Favourites" icon="lu:Star">
                    {favPresets.map((p) => (
                      <MaterialRow
                        key={p.abbrev}
                        abbrev={p.abbrev}
                        name={p.name}
                        isFav
                        isSelected={selectedAbbrev === p.abbrev}
                        onSelect={() => handleSelect(p.abbrev)}
                        onToggleFav={() => toggleFavorite(p.abbrev)}
                      />
                    ))}
                  </Section>
                )}
                {recentPresets.length > 0 && (
                  <Section label="Recently Used">
                    {recentPresets.map((p) => (
                      <MaterialRow
                        key={p.abbrev}
                        abbrev={p.abbrev}
                        name={p.name}
                        isFav={favorites.includes(p.abbrev)}
                        isSelected={selectedAbbrev === p.abbrev}
                        onSelect={() => handleSelect(p.abbrev)}
                        onToggleFav={() => toggleFavorite(p.abbrev)}
                      />
                    ))}
                  </Section>
                )}
              </>
            ) : (
              <Section label={`${filtered.length} Results`}>
                {filtered.map((p) => (
                  <MaterialRow
                    key={p.abbrev}
                    abbrev={p.abbrev}
                    name={p.name}
                    isFav={favorites.includes(p.abbrev)}
                    isSelected={selectedAbbrev === p.abbrev}
                    onSelect={() => handleSelect(p.abbrev)}
                    onToggleFav={() => toggleFavorite(p.abbrev)}
                  />
                ))}
              </Section>
            )}
          </div>
        </PopoverContent>
      </Popover>

      {selectedPreset && (
        <div className="grid grid-cols-2 gap-2 text-xs text-gray-500">
          <span>
            Temp:{" "}
            <b className="text-gray-800">
              {selectedPreset.temp_min === selectedPreset.temp_max
                ? `${selectedPreset.temp_min}°C`
                : `${selectedPreset.temp_min}–${selectedPreset.temp_max}°C`}
            </b>
          </span>
          <span>
            Time:{" "}
            <b className="text-gray-800">
              {selectedPreset.drying_time_min === selectedPreset.drying_time_max
                ? `${selectedPreset.drying_time_min} h`
                : `${selectedPreset.drying_time_min}–${selectedPreset.drying_time_max} h`}
            </b>
          </span>
        </div>
      )}
    </ControlCard>
  );
}

function Section({
  label,
  icon,
  children,
}: {
  label: string;
  icon?: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="flex items-center gap-1 px-3 py-1.5 text-xs font-semibold text-gray-400">
        {icon && <Icon name={icon as any} className="size-3" />}
        {label}
      </div>
      {children}
    </div>
  );
}

function MaterialRow({
  abbrev,
  name,
  isFav,
  isSelected,
  onSelect,
  onToggleFav,
}: {
  abbrev: string;
  name: string;
  isFav: boolean;
  isSelected: boolean;
  onSelect: () => void;
  onToggleFav: () => void;
}) {
  return (
    <div
      className={[
        "flex cursor-pointer items-center gap-2 px-3 py-2 hover:bg-gray-50",
        isSelected ? "bg-gray-100 font-semibold" : "",
      ].join(" ")}
      onClick={onSelect}
    >
      <span className="w-20 shrink-0 font-mono text-sm font-bold text-gray-800">
        {abbrev}
      </span>
      <span className="flex-1 truncate text-sm text-gray-500">{name}</span>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onToggleFav();
        }}
        className="shrink-0"
      >
        <Icon
          name="lu:Star"
          className={[
            "size-4",
            isFav ? "fill-yellow-400 text-yellow-400" : "text-gray-200",
          ].join(" ")}
        />
      </button>
    </div>
  );
}

function FlowRateCard({
  machineIdentification,
}: {
  machineIdentification: MachineIdentificationUnique;
}) {
  const { throughput, selectedAbbrev, setThroughput } =
    useDryerV1MaterialStore();
  const { request: sendMutation } = useMachineMutate(z.any());

  const selectedPreset = selectedAbbrev
    ? MATERIAL_PRESETS.find((p) => p.abbrev === selectedAbbrev)
    : null;

  const handleThroughputChange = (val: number) => {
    setThroughput(val);
    if (selectedAbbrev) {
      sendMutation({
        machine_identification_unique: machineIdentification,
        data: {
          ApplyMaterialPreset: {
            abbrev: selectedAbbrev,
            throughput_kg_per_h: val,
          },
        },
      });
    }
  };

  return (
    <ControlCard title="Flow Rate">
      <Label label="Flow Rate">
        <EditValue
          title="Flow Rate"
          value={throughput}
          min={0.1}
          max={500}
          step={0.5}
          renderValue={(val) => `${val.toFixed(1)} kg/h`}
          onChange={handleThroughputChange}
        />
      </Label>
      {selectedPreset && (
        <div className="rounded-xl bg-blue-50 px-3 py-2 text-xs text-blue-700">
          <div className="mb-1 font-semibold text-blue-500">
            Air Volume Calculation
          </div>
          <div className="flex items-center justify-between">
            <span className="text-blue-600">Spec. Air Vol.</span>
            <span className="font-mono tabular-nums">
              {selectedPreset.specific_air_volume.toFixed(2)} m³/kg
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-blue-600">Throughput</span>
            <span className="font-mono tabular-nums">
              {throughput.toFixed(1)} kg/h
            </span>
          </div>
          <div className="mt-1 flex items-center justify-between border-t border-blue-200 pt-1">
            <span className="font-semibold text-blue-700">= Air Volume</span>
            <span className="font-mono font-bold tabular-nums text-blue-900">
              {(selectedPreset.specific_air_volume * throughput).toFixed(1)} m³/h
            </span>
          </div>
        </div>
      )}
    </ControlCard>
  );
}
