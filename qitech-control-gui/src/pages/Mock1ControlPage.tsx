/**
 * Mock1 machine control page
 *
 * Demonstrates:
 * - High-frequency live values with inline sparklines (MiniGraph)
 * - Optimistic state updates for frequency + mode controls
 * - Mode toggle (Standby / Running)
 * - Numeric input with default value indicator
 *
 * Compare to electron/src/machines/mock/mock1/Mock1ControlPage.tsx +
 * useMock.ts (~200 lines combined). This file replaces both.
 */

import { Show, createSignal, createMemo } from "solid-js";
import { useParams } from "@solidjs/router";
import { createMock1Namespace, createMock1SeriesBuffer, VENDOR_QITECH, MACHINE_MOCK } from "../namespaces/mock1";
import type { Mode, SeriesBuffer } from "../namespaces/mock1";
import { mutateMachine } from "../lib/api";
import MiniGraph from "../components/MiniGraph";

export default function Mock1ControlPage() {
  const params = useParams<{ serial: string }>();
  const serial = () => parseInt(params.serial);

  const [state] = createMock1Namespace(serial());
  const [getSeries] = createMock1SeriesBuffer(serial());

  // Optimistic state: prefer local update until server confirms
  const [optimisticFreq1, setOptimisticFreq1] = createSignal<number | null>(null);
  const [optimisticFreq2, setOptimisticFreq2] = createSignal<number | null>(null);
  const [optimisticFreq3, setOptimisticFreq3] = createSignal<number | null>(null);
  const [optimisticMode, setOptimisticMode] = createSignal<Mode | null>(null);

  const freq1 = () => optimisticFreq1() ?? state().frequency1 ?? 1000;
  const freq2 = () => optimisticFreq2() ?? state().frequency2 ?? 2000;
  const freq3 = () => optimisticFreq3() ?? state().frequency3 ?? 5000;
  const mode = () => optimisticMode() ?? state().mode_state?.mode ?? "Standby";

  const isLoaded = createMemo(() => state().frequency1 !== null);

  const machineId = () => ({
    machine_identification: { vendor: VENDOR_QITECH, machine: MACHINE_MOCK },
    serial: serial(),
  });

  async function setFrequency(
    index: 1 | 2 | 3,
    value: number,
    setOptimistic: (v: number) => void,
    resetOptimistic: () => void,
  ) {
    setOptimistic(value);
    try {
      await mutateMachine(machineId(), { [`SetFrequency${index}`]: value });
    } catch (e) {
      console.error(`SetFrequency${index} failed:`, e);
      resetOptimistic();
    }
  }

  async function setMode(newMode: Mode) {
    setOptimisticMode(newMode);
    try {
      await mutateMachine(machineId(), { SetMode: newMode });
    } catch (e) {
      console.error("SetMode failed:", e);
      setOptimisticMode(null);
    }
  }

  return (
    <div class="page">
      <h1>Mock Machine — Control</h1>
      <p class="serial">Serial: #{serial()}</p>

      <Show when={!isLoaded()} fallback={null}>
        <p class="muted">Waiting for machine state…</p>
      </Show>

      <Show when={isLoaded()}>
        <div class="control-grid">

          {/* Live values with sparklines */}
          <div class="control-card live-values-card">
            <div class="control-card-title">Live Values</div>
            <div class="live-values">
              <LiveValueRow
                label="Sum"
                value={state().amplitude_sum}
                getBuffer={() => getSeries().amplitude_sum}
                stroke="#6c8cff"
              />
              <LiveValueRow
                label="Wave 1"
                value={state().amplitude1}
                getBuffer={() => getSeries().amplitude1}
                stroke="#6c8cff"
              />
              <LiveValueRow
                label="Wave 2"
                value={state().amplitude2}
                getBuffer={() => getSeries().amplitude2}
                stroke="#f472b6"
              />
              <LiveValueRow
                label="Wave 3"
                value={state().amplitude3}
                getBuffer={() => getSeries().amplitude3}
                stroke="#34d399"
              />
            </div>
          </div>

          {/* Frequencies */}
          <div class="control-card">
            <div class="control-card-title">Frequencies (mHz)</div>
            <div class="freq-grid">
              <FrequencyInput
                label="Frequency 1"
                value={freq1()}
                defaultValue={state().is_default_state ? null : state().frequency1}
                onChange={(v) => setFrequency(1, v, setOptimisticFreq1, () => setOptimisticFreq1(null))}
              />
              <FrequencyInput
                label="Frequency 2"
                value={freq2()}
                defaultValue={state().is_default_state ? null : state().frequency2}
                onChange={(v) => setFrequency(2, v, setOptimisticFreq2, () => setOptimisticFreq2(null))}
              />
              <FrequencyInput
                label="Frequency 3"
                value={freq3()}
                defaultValue={state().is_default_state ? null : state().frequency3}
                onChange={(v) => setFrequency(3, v, setOptimisticFreq3, () => setOptimisticFreq3(null))}
              />
            </div>
          </div>

          {/* Mode */}
          <div class="control-card">
            <div class="control-card-title">Mode</div>
            <div class="mode-selector">
              <button
                class={`mode-btn ${mode() === "Standby" ? "active" : ""}`}
                onClick={() => setMode("Standby")}
              >
                Standby
              </button>
              <button
                class={`mode-btn ${mode() === "Running" ? "active running" : ""}`}
                onClick={() => setMode("Running")}
              >
                Running
              </button>
            </div>
          </div>

        </div>
      </Show>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function LiveValueRow(props: {
  label: string;
  value: number | null;
  getBuffer: () => SeriesBuffer;
  stroke: string;
}) {
  return (
    <div class="live-value-row-spark">
      <div class="live-value-left">
        <span class="live-value-label">{props.label}</span>
        <span class="live-value-number">
          <Show when={props.value !== null} fallback={<span class="muted">–</span>}>
            {props.value!.toFixed(3)}
          </Show>
        </span>
      </div>
      <div class="live-value-spark">
        <MiniGraph getBuffer={props.getBuffer} stroke={props.stroke} timeWindow={10} />
      </div>
    </div>
  );
}

function FrequencyInput(props: {
  label: string;
  value: number;
  defaultValue: number | null | undefined;
  onChange: (v: number) => void;
}) {
  const [editValue, setEditValue] = createSignal<string | null>(null);

  const displayValue = () => editValue() ?? props.value.toFixed(0);

  function commit() {
    const v = parseFloat(editValue() ?? "");
    if (!isNaN(v) && v >= 0 && v <= 100_000) {
      props.onChange(v);
    }
    setEditValue(null);
  }

  return (
    <div class="freq-input-group">
      <label class="freq-label">{props.label}</label>
      <div class="freq-input-row">
        <input
          class="freq-input"
          type="number"
          min="0"
          max="100000"
          step="100"
          value={displayValue()}
          onInput={(e) => setEditValue(e.currentTarget.value)}
          onBlur={commit}
          onKeyDown={(e) => e.key === "Enter" && commit()}
        />
        <Show when={props.defaultValue != null && props.defaultValue !== props.value}>
          <button
            class="freq-reset-btn"
            title={`Reset to default (${props.defaultValue!.toFixed(0)})`}
            onClick={() => props.onChange(props.defaultValue!)}
          >
            ↺
          </button>
        </Show>
      </div>
    </div>
  );
}
