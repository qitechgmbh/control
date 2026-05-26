import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { TouchSlider } from "@/components/touch/TouchSlider";
import { Input } from "@/components/ui/input";
import React from "react";
import { useAnalogOutOversampling } from "./useAnalogOutOversamplingMachine";
import { ChannelConfig, WaveformType } from "./useAnalogOutOversamplingMachineNamespace";

function SampleBar({
  value,
  label,
}: {
  value: number; // -1.0 .. 1.0
  label: string;
}) {
  const pct = ((value + 1) / 2) * 100; // map to 0-100%
  return (
    <div className="flex flex-col items-center gap-1">
      <div className="relative h-16 w-5 overflow-hidden rounded bg-gray-800">
        {/* centre line */}
        <div className="absolute inset-x-0 top-1/2 h-px bg-gray-600" />
        {/* fill from centre */}
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

function ChannelLiveValues({
  samples,
  label,
}: {
  samples: number[];
  label: string;
}) {
  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium text-gray-300">{label}</span>
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
  );
}

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
      <div className="flex flex-col gap-5">

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
              Sine: {
                children: "Sine",
                icon: "lu:AudioWaveform",
                isActiveClassName: "bg-blue-600",
              },
              Sawtooth: {
                children: "Saw",
                icon: "lu:TrendingUp",
                isActiveClassName: "bg-blue-600",
              },
              Square: {
                children: "Square",
                icon: "lu:Square",
                isActiveClassName: "bg-blue-600",
              },
              Constant: {
                children: "DC",
                icon: "lu:Minus",
                isActiveClassName: "bg-blue-600",
              },
            }}
          />
        </Label>

        {/* Frequency */}
        {config.waveform !== "Constant" && (
          <Label label={`Frequency: ${config.frequency_hz.toFixed(1)} Hz`}>
            <div className="flex gap-2">
              <TouchSlider
                disabled={disabled}
                min={0.1}
                max={500}
                step={0.1}
                value={[config.frequency_hz]}
                onValueChange={([v]) => onFrequency(v)}
                className="flex-1"
              />
              <Input
                type="number"
                disabled={disabled}
                className="w-24 text-right"
                value={config.frequency_hz}
                min={0.001}
                max={10000}
                step={0.1}
                onChange={(e) => {
                  const v = parseFloat(e.target.value);
                  if (!isNaN(v) && v > 0) onFrequency(v);
                }}
              />
            </div>
          </Label>
        )}

        {/* Amplitude */}
        <Label
          label={`Amplitude: ${(config.amplitude * 10).toFixed(1)} V`}
        >
          <TouchSlider
            disabled={disabled}
            min={0}
            max={1}
            step={0.01}
            value={[config.amplitude]}
            onValueChange={([v]) => onAmplitude(v)}
          />
        </Label>

        {/* Offset */}
        <Label
          label={`Offset: ${(config.offset * 10).toFixed(1)} V`}
        >
          <TouchSlider
            disabled={disabled}
            min={-1}
            max={1}
            step={0.01}
            value={[config.offset]}
            onValueChange={([v]) => onOffset(v)}
          />
        </Label>
      </div>
    </ControlCard>
  );
}

export function AnalogOutOversamplingControlPage() {
  const {
    state,
    liveValues,
    setWaveform,
    setFrequency,
    setAmplitude,
    setOffset,
    isDisabled,
  } = useAnalogOutOversampling();

  const ch1 = state?.channels[0];
  const ch2 = state?.channels[1];

  return (
    <Page>
      <ControlGrid columns={2}>

        {/* Info strip */}
        <ControlCard title="Device Info" className="col-span-2">
          <div className="flex gap-6">
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
                {state
                  ? `${((state.oversample_factor / state.cycle_time_us) * 1e6 / 1000).toFixed(1)} kSps`
                  : "—"}
              </Badge>
            </Label>
          </div>
        </ControlCard>

        {/* Channel controls */}
        {ch1 && (
          <ChannelControl
            index={0}
            config={ch1}
            disabled={isDisabled}
            onWaveform={(w) => setWaveform(0, w)}
            onFrequency={(hz) => setFrequency(0, hz)}
            onAmplitude={(a) => setAmplitude(0, a)}
            onOffset={(o) => setOffset(0, o)}
          />
        )}
        {ch2 && (
          <ChannelControl
            index={1}
            config={ch2}
            disabled={isDisabled}
            onWaveform={(w) => setWaveform(1, w)}
            onFrequency={(hz) => setFrequency(1, hz)}
            onAmplitude={(a) => setAmplitude(1, a)}
            onOffset={(o) => setOffset(1, o)}
          />
        )}

        {/* Live values */}
        <ControlCard title="Live Samples" className="col-span-2">
          {liveValues ? (
            <div className="grid grid-cols-2 gap-8">
              <ChannelLiveValues
                samples={liveValues.ch1_samples}
                label="Channel 1"
              />
              <ChannelLiveValues
                samples={liveValues.ch2_samples}
                label="Channel 2"
              />
            </div>
          ) : (
            <span className="text-sm text-gray-500">Waiting for data…</span>
          )}
        </ControlCard>

      </ControlGrid>
    </Page>
  );
}
