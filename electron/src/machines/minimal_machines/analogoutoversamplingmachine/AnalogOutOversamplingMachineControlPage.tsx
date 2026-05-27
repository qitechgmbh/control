import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { EditValue } from "@/control/EditValue";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { TimeSeries } from "@/lib/timeseries";
import React from "react";
import { useAnalogOutOversampling } from "./useAnalogOutOversamplingMachine";
import { ChannelConfig, WaveformType } from "./analogOutOversamplingMachineNamespace";

// ========== Sample bar ==========

function SampleBar({ value, label }: { value: number; label: string }) {
  const pct = ((value + 1) / 2) * 100;
  return (
    <div className="flex flex-col items-center gap-1">
      <div className="relative h-16 w-5 overflow-hidden rounded bg-gray-800">
        <div className="absolute inset-x-0 top-1/2 h-px bg-gray-600" />
        <div
          className={`absolute inset-x-0 ${value >= 0 ? "bg-blue-500" : "bg-orange-500"}`}
          style={
            value >= 0
              ? { bottom: "50%", top: `${100 - pct}%` }
              : { top: "50%", bottom: `${pct}%` }
          }
        />
      </div>
      <span className="text-xs text-gray-400">{label}</span>
    </div>
  );
}

// ========== Voltage time-series card ==========

function VoltageCard({ title, timeseries }: { title: string; timeseries: TimeSeries }) {
  return (
    <ControlCard title={title}>
      <TimeSeriesValueNumeric
        label="Mean output voltage"
        timeseries={timeseries}
        renderValue={(v) => `${v.toFixed(2)} V`}
      />
    </ControlCard>
  );
}

// ========== Per-slot bar visualiser ==========

function SamplesCard({ title, samples }: { title: string; samples: number[] }) {
  return (
    <ControlCard title={title}>
      {samples.length === 0 ? (
        <span className="text-sm text-gray-500">Waiting for data…</span>
      ) : (
        <div className="flex flex-col gap-2">
          <div className="flex gap-2">
            {samples.map((v, i) => (
              <SampleBar key={i} value={v} label={`S${i}`} />
            ))}
          </div>
          <div className="flex gap-2">
            {samples.map((v, i) => (
              <span key={i} className="w-5 text-center text-xs text-gray-500">
                {(v * 10).toFixed(1)}V
              </span>
            ))}
          </div>
        </div>
      )}
    </ControlCard>
  );
}

// ========== Per-channel controls ==========

function ChannelControl({
  index,
  config,
  disabled,
  onWaveform,
  onFrequency,
  onAmplitude,
  onOffset,
}: {
  index: number;
  config: ChannelConfig;
  disabled: boolean;
  onWaveform: (w: WaveformType) => void;
  onFrequency: (hz: number) => void;
  onAmplitude: (a: number) => void;
  onOffset: (o: number) => void;
}) {
  return (
    <ControlCard title={`Channel ${index + 1}`}>
      <div className="flex flex-col gap-6">

        {/* Waveform selector */}
        <Label label="Waveform">
          <SelectionGroup<WaveformType>
            value={config.waveform}
            disabled={disabled}
            loading={false}
            onChange={onWaveform}
            orientation="horizontal"
            className="grid grid-cols-4 gap-1"
            options={{
              Sine:     { children: "Sine",   icon: "lu:AudioWaveform", isActiveClassName: "bg-blue-600" },
              Sawtooth: { children: "Saw",    icon: "lu:TrendingUp",    isActiveClassName: "bg-blue-600" },
              Square:   { children: "Square", icon: "lu:Square",        isActiveClassName: "bg-blue-600" },
              Constant: { children: "DC",     icon: "lu:Minus",         isActiveClassName: "bg-blue-600" },
            }}
          />
        </Label>

        <div className="flex flex-row flex-wrap gap-4">
          {/* Frequency — hidden for Constant */}
          {config.waveform !== "Constant" && (
            <EditValue
              title="Frequency"
              value={config.frequency_hz}
              defaultValue={10}
              min={0.001}
              max={10000}
              minSlider={0.1}
              maxSlider={500}
              step={0.1}
              disabled={disabled}
              renderValue={(v) => v.toFixed(1)}
              onChange={onFrequency}
            />
          )}

          {/* Amplitude */}
          <EditValue
            title="Amplitude"
            value={config.amplitude * 10}
            defaultValue={10}
            min={0}
            max={10}
            step={0.1}
            disabled={disabled}
            renderValue={(v) => v.toFixed(1)}
            onChange={(v) => onAmplitude(v / 10)}
          />

          {/* Offset */}
          <EditValue
            title="Offset"
            value={config.offset * 10}
            defaultValue={0}
            min={-10}
            max={10}
            step={0.1}
            disabled={disabled}
            renderValue={(v) => v.toFixed(1)}
            onChange={(v) => onOffset(v / 10)}
          />
        </div>

      </div>
    </ControlCard>
  );
}

// ========== Page ==========

export function AnalogOutOversamplingControlPage() {
  const {
    state,
    ch1Voltage,
    ch2Voltage,
    ch1Samples,
    ch2Samples,
    setWaveform,
    setFrequency,
    setAmplitude,
    setOffset,
    isDisabled,
  } = useAnalogOutOversampling();

  const ch1 = state?.channels[0];
  const ch2 = state?.channels[1];
  const effectiveSampleRate = state
    ? (state.oversample_factor / state.cycle_time_us) * 1e6
    : null;

  return (
    <Page>
      <ControlGrid columns={2}>

        {/* Device info */}
        <ControlCard title="Device Info" className="col-span-2">
          <div className="flex flex-wrap gap-6">
            <Label label="Oversample factor">
              <Badge className="bg-blue-600 text-sm">
                {state?.oversample_factor ?? "—"}×
              </Badge>
            </Label>
            <Label label="Cycle time">
              <Badge className="bg-gray-600 text-sm">
                {state ? `${state.cycle_time_us} µs` : "—"}
              </Badge>
            </Label>
            <Label label="Effective sample rate">
              <Badge className="bg-gray-600 text-sm">
                {effectiveSampleRate != null
                  ? `${(effectiveSampleRate / 1000).toFixed(1)} kSps`
                  : "—"}
              </Badge>
            </Label>
          </div>
        </ControlCard>

        {/* Channel controls */}
        {ch1 && (
          <ChannelControl
            index={0} config={ch1} disabled={isDisabled}
            onWaveform={(w) => setWaveform(0, w)}
            onFrequency={(hz) => setFrequency(0, hz)}
            onAmplitude={(a) => setAmplitude(0, a)}
            onOffset={(o) => setOffset(0, o)}
          />
        )}
        {ch2 && (
          <ChannelControl
            index={1} config={ch2} disabled={isDisabled}
            onWaveform={(w) => setWaveform(1, w)}
            onFrequency={(hz) => setFrequency(1, hz)}
            onAmplitude={(a) => setAmplitude(1, a)}
            onOffset={(o) => setOffset(1, o)}
          />
        )}

        {/* Voltage time series */}
        <VoltageCard title="Channel 1 — Output Voltage" timeseries={ch1Voltage} />
        <VoltageCard title="Channel 2 — Output Voltage" timeseries={ch2Voltage} />

        {/* Per-slot bar visualisers */}
        <SamplesCard title="Channel 1 — Oversampled Slots" samples={ch1Samples} />
        <SamplesCard title="Channel 2 — Oversampled Slots" samples={ch2Samples} />

      </ControlGrid>
    </Page>
  );
}
