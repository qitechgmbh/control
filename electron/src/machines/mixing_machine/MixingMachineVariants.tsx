import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { TouchSlider } from "@/components/touch/TouchSlider";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { StatusBadge } from "@/control/StatusBadge";
import React from "react";
import { MachineOverview } from "./MixingMachinePreview";
import {
  ConnectedExtruderCard,
  ConnectedExtruderPanel,
} from "./MixerExtruderLink";
import { useMixerDemo } from "./useMixerDemo";

function mixHexColors(colorA: string, colorB: string, ratioA: number) {
  const parse = (hex: string) => [
    Number.parseInt(hex.slice(1, 3), 16),
    Number.parseInt(hex.slice(3, 5), 16),
    Number.parseInt(hex.slice(5, 7), 16),
  ];
  const a = parse(colorA);
  const b = parse(colorB);
  const weightA = ratioA / 100;
  return `#${a
    .map((channel, index) =>
      Math.round(channel * weightA + b[index] * (1 - weightA))
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
}

function MachineStatusBadges({
  demo,
}: {
  demo: ReturnType<typeof useMixerDemo>;
}) {
  const phaseBadge =
    demo.phase === "running" ? (
      <StatusBadge variant="success">Mixing</StatusBadge>
    ) : demo.phase === "starting" ? (
      <StatusBadge variant="success">Starting</StatusBadge>
    ) : demo.phase === "purging" ? (
      <StatusBadge variant="success">Clearing</StatusBadge>
    ) : demo.canStart ? (
      <StatusBadge variant="success">Ready</StatusBadge>
    ) : (
      <StatusBadge variant="error">Not ready</StatusBadge>
    );

  return (
    <div className="flex flex-wrap justify-end gap-2">
      {phaseBadge}
      {demo.mixerFault && (
        <StatusBadge variant="error">Mixer fault</StatusBadge>
      )}
      {demo.hopperAEmpty && (
        <StatusBadge variant="error">Hopper A empty</StatusBadge>
      )}
      {demo.hopperBEmpty && (
        <StatusBadge variant="error">Hopper B empty</StatusBadge>
      )}
      {demo.hopperALow && !demo.hopperAEmpty && (
        <StatusBadge variant="warning">Hopper A low</StatusBadge>
      )}
      {demo.hopperBLow && !demo.hopperBEmpty && (
        <StatusBadge variant="warning">Hopper B low</StatusBadge>
      )}
      {demo.extruderLinkState === "no-demand" && (
        <StatusBadge variant="warning">No extruder demand</StatusBadge>
      )}
      {demo.extruderLinkState === "fault" && (
        <StatusBadge variant="error">Extruder fault</StatusBadge>
      )}
    </div>
  );
}

function Overview({
  demo,
  className,
}: {
  demo: ReturnType<typeof useMixerDemo>;
  className?: string;
}) {
  return (
    <div className={className}>
      <MachineOverview
        phase={demo.phase}
        ratioA={demo.ratioA}
        feedRate={demo.feedRate}
        hopperAEmpty={demo.hopperAEmpty}
        hopperBEmpty={demo.hopperBEmpty}
        hopperALow={demo.hopperALow}
        hopperBLow={demo.hopperBLow}
      />
    </div>
  );
}

function RunButton({ demo }: { demo: ReturnType<typeof useMixerDemo> }) {
  return demo.running || demo.busy ? (
    <TouchButton variant="destructive" icon="lu:OctagonX" onClick={demo.stop}>
      {demo.phase === "purging" ? "Clearing Mixer…" : "Stop Mixing"}
    </TouchButton>
  ) : (
    <TouchButton
      icon="lu:Play"
      disabled={!demo.canStart}
      onClick={demo.start}
      className="bg-green-600 text-white"
    >
      Start Mixing
    </TouchButton>
  );
}

export function MixingMachineVariant2() {
  const demo = useMixerDemo();
  return (
    <Page className="mm-page">
      <div className="flex items-start justify-between gap-4">
        <div className="grid gap-4">
          <h1 className="text-3xl font-bold">Material Mixer</h1>
        </div>
        <MachineStatusBadges demo={demo} />
      </div>

      <div className="grid grid-cols-1 gap-4 2xl:grid-cols-[minmax(0,1.55fr)_minmax(360px,0.65fr)]">
        <ControlCard title="Machine">
          <Overview demo={demo} className="mm-operator-overview" />
        </ControlCard>

        <div>
          <ControlCard title="Run" className="h-full">
            <div className="grid grid-cols-2 gap-3">
              <div className="rounded-xl border border-gray-200 bg-gray-50 p-4">
                <span className="text-sm text-gray-500">Hopper A</span>
                <strong className="mt-1 block font-mono text-4xl">
                  {demo.ratioA}%
                </strong>
              </div>
              <div className="rounded-xl border border-gray-200 bg-gray-50 p-4 text-right">
                <span className="text-sm text-gray-500">Hopper B</span>
                <strong className="mt-1 block font-mono text-4xl">
                  {demo.ratioB}%
                </strong>
              </div>
            </div>
            <TouchSlider
              min={0}
              max={100}
              step={1}
              value={[demo.ratioA]}
              disabled={demo.running || demo.busy}
              minLabel="All B"
              maxLabel="All A"
              onValueChange={([value]) => demo.setRatioA(value)}
            />
            <Label label="Total Feed Rate">
              <div className="flex items-center gap-3">
                <TouchButton
                  variant="outline"
                  disabled={demo.running || demo.busy}
                  onClick={() =>
                    demo.setFeedRate(Math.max(1, demo.feedRate - 0.5))
                  }
                >
                  −
                </TouchButton>
                <div className="flex-1 rounded-xl border border-gray-200 bg-gray-50 p-3 text-center">
                  <strong className="font-mono text-3xl">
                    {demo.feedRate}
                  </strong>
                  <span className="ml-2 text-sm text-gray-500">kg/h</span>
                </div>
                <TouchButton
                  variant="outline"
                  disabled={demo.running || demo.busy}
                  onClick={() =>
                    demo.setFeedRate(Math.min(25, demo.feedRate + 0.5))
                  }
                >
                  +
                </TouchButton>
              </div>
            </Label>
            <RunButton demo={demo} />
            <p className="text-xs text-amber-700">
              Ratio is estimated until feeder calibration is available.
            </p>
            <div className="border-t border-gray-200 pt-4">
              <h3 className="mb-3 text-sm font-semibold">Connected Extruder</h3>
              <ConnectedExtruderPanel
                state={demo.extruderLinkState}
                onStateChange={demo.setExtruderLinkState}
              />
            </div>
            <div className="border-t border-gray-200 pt-4">
              <div className="mb-2 flex items-center justify-between">
                <span className="text-sm font-semibold">Errors</span>
                {(demo.hopperAEmpty ||
                  demo.hopperBEmpty ||
                  demo.hopperALow ||
                  demo.hopperBLow ||
                  demo.mixerFault) && (
                  <button
                    className="text-xs font-semibold text-blue-700"
                    onClick={demo.reset}
                  >
                    Clear all
                  </button>
                )}
              </div>
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-3 2xl:grid-cols-1">
                {[
                  {
                    label: "Hopper A low",
                    active: demo.hopperALow,
                    warning: true,
                    toggle: () => demo.setHopperALow(!demo.hopperALow),
                  },
                  {
                    label: "Hopper A empty",
                    active: demo.hopperAEmpty,
                    warning: false,
                    toggle: () => demo.setHopperAEmpty(!demo.hopperAEmpty),
                  },
                  {
                    label: "Hopper B low",
                    active: demo.hopperBLow,
                    warning: true,
                    toggle: () => demo.setHopperBLow(!demo.hopperBLow),
                  },
                  {
                    label: "Hopper B empty",
                    active: demo.hopperBEmpty,
                    warning: false,
                    toggle: () => demo.setHopperBEmpty(!demo.hopperBEmpty),
                  },
                  {
                    label: "Mixer fault",
                    active: demo.mixerFault,
                    warning: false,
                    toggle: () => demo.setMixerFault(!demo.mixerFault),
                  },
                ].map((error) => (
                  <button
                    key={error.label}
                    className={`min-h-10 rounded-lg border px-3 text-left text-sm font-medium ${
                      error.active
                        ? error.warning
                          ? "border-amber-300 bg-amber-50 text-amber-700"
                          : "border-red-300 bg-red-50 text-red-700"
                        : "border-gray-200 bg-gray-50 text-gray-700"
                    }`}
                    onClick={error.toggle}
                  >
                    {error.label}
                  </button>
                ))}
              </div>
            </div>
          </ControlCard>
        </div>
      </div>
    </Page>
  );
}

export function MixingMachineVariant3() {
  const demo = useMixerDemo();
  const [colorA, setColorA] = React.useState("#2563eb");
  const [colorB, setColorB] = React.useState("#9ca3af");
  const estimatedColor = mixHexColors(colorA, colorB, demo.ratioA);

  return (
    <Page className="mm-page">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold">Material Mixer</h1>
        </div>
        <MachineStatusBadges demo={demo} />
      </div>

      <ControlCard title="Overview">
        <MachineOverview
          phase={demo.phase}
          ratioA={demo.ratioA}
          feedRate={demo.feedRate}
          hopperAEmpty={demo.hopperAEmpty}
          hopperBEmpty={demo.hopperBEmpty}
          hopperALow={demo.hopperALow}
          hopperBLow={demo.hopperBLow}
          materialColorA={colorA}
          materialColorB={colorB}
          showMaterialColors
        />
      </ControlCard>

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-2 2xl:grid-cols-4">
        <ControlCard title="Material Colors">
          {[
            {
              label: "Hopper A",
              color: colorA,
              setColor: setColorA,
            },
            {
              label: "Hopper B",
              color: colorB,
              setColor: setColorB,
            },
          ].map((input) => (
            <label
              key={input.label}
              className="flex items-center gap-4 rounded-xl border border-gray-200 bg-gray-50 p-3"
            >
              <input
                type="color"
                value={input.color}
                disabled={demo.running || demo.busy}
                onChange={(event) => input.setColor(event.target.value)}
                className="mm-color-input"
                aria-label={`${input.label} color`}
              />
              <span className="flex-1">
                <strong className="block">{input.label}</strong>
                <small className="text-gray-500">Selected color</small>
              </span>
              <code className="text-sm text-gray-500">
                {input.color.toUpperCase()}
              </code>
            </label>
          ))}
          <div className="border-t border-gray-200 pt-4">
            <div className="mb-2 flex items-center justify-between">
              <strong className="text-sm">Estimated Blend Color</strong>
              <code className="text-sm font-semibold">
                {estimatedColor.toUpperCase()}
              </code>
            </div>
            <div
              className="mm-result-color"
              style={{ backgroundColor: estimatedColor }}
              aria-label={`Estimated blend color ${estimatedColor}`}
            />
            <p className="mt-3 text-xs text-amber-800">
              Screen estimate only; actual plastic color depends on material,
              pigment, contamination, and processing conditions.
            </p>
          </div>
        </ControlCard>

        <ControlCard title="Blend">
          <Label label="Material Ratio">
            <div className="mb-2 flex justify-between text-sm">
              <strong>Hopper A · {demo.ratioA}%</strong>
              <strong>Hopper B · {demo.ratioB}%</strong>
            </div>
            <TouchSlider
              min={0}
              max={100}
              step={1}
              value={[demo.ratioA]}
              disabled={demo.running || demo.busy}
              minLabel="All B"
              maxLabel="All A"
              onValueChange={([value]) => demo.setRatioA(value)}
            />
          </Label>
          <Label label="Total Feed Rate">
            <div className="flex items-center gap-3">
              <TouchButton
                variant="outline"
                disabled={demo.running || demo.busy}
                onClick={() =>
                  demo.setFeedRate(Math.max(1, demo.feedRate - 0.5))
                }
              >
                −
              </TouchButton>
              <div className="flex-1 rounded-xl border border-gray-200 bg-gray-50 p-3 text-center">
                <strong className="font-mono text-3xl">{demo.feedRate}</strong>
                <span className="ml-2 text-sm text-gray-500">kg/h</span>
              </div>
              <TouchButton
                variant="outline"
                disabled={demo.running || demo.busy}
                onClick={() =>
                  demo.setFeedRate(Math.min(25, demo.feedRate + 0.5))
                }
              >
                +
              </TouchButton>
            </div>
          </Label>
          <RunButton demo={demo} />
        </ControlCard>

        <ControlCard title="Errors">
          <div className="grid grid-cols-1 gap-2">
            {[
              {
                label: "Hopper A low",
                active: demo.hopperALow,
                warning: true,
                toggle: () => demo.setHopperALow(!demo.hopperALow),
              },
              {
                label: "Hopper A empty",
                active: demo.hopperAEmpty,
                warning: false,
                toggle: () => demo.setHopperAEmpty(!demo.hopperAEmpty),
              },
              {
                label: "Hopper B low",
                active: demo.hopperBLow,
                warning: true,
                toggle: () => demo.setHopperBLow(!demo.hopperBLow),
              },
              {
                label: "Hopper B empty",
                active: demo.hopperBEmpty,
                warning: false,
                toggle: () => demo.setHopperBEmpty(!demo.hopperBEmpty),
              },
              {
                label: "Mixer fault",
                active: demo.mixerFault,
                warning: false,
                toggle: () => demo.setMixerFault(!demo.mixerFault),
              },
            ].map((signal) => (
              <button
                key={signal.label}
                className={`min-h-11 rounded-lg border px-3 text-left text-sm font-medium ${
                  signal.active
                    ? signal.warning
                      ? "border-amber-300 bg-amber-50 text-amber-700"
                      : "border-red-300 bg-red-50 text-red-700"
                    : "border-gray-200 bg-gray-50 text-gray-700"
                }`}
                onClick={signal.toggle}
              >
                {signal.label}
              </button>
            ))}
          </div>
          <TouchButton
            variant="outline"
            icon="lu:RotateCcw"
            onClick={demo.reset}
          >
            Clear Simulation
          </TouchButton>
        </ControlCard>

        <ConnectedExtruderCard
          state={demo.extruderLinkState}
          onStateChange={demo.setExtruderLinkState}
        />
      </div>
    </Page>
  );
}
