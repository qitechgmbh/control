/**
 * Machines overview page — lists all detected machines with navigation links.
 * Uses the main namespace signal directly; no intermediate store or hook.
 */

import { For, Show, createMemo } from "solid-js";
import { A } from "@solidjs/router";
import type { MachineObj } from "../namespaces/main";
import type { MainState } from "../namespaces/main";

type Props = {
  mainState: () => MainState;
};

function machineSlug(m: MachineObj): string {
  const { vendor, machine } = m.machine_identification_unique.machine_identification;
  // Map vendor_machine → URL slug (matches routes in App.tsx)
  const knownMachines: Record<string, string> = {
    "1_51": "testmachine",    // 0x0033
    "1_2": "winder2",         // 0x0002
    "1_4": "extruder2",       // 0x0004
    "1_6": "laser1",          // 0x0006
    "1_7": "mock1",           // 0x0007
    "1_8": "buffer1",         // 0x0008
    "1_9": "aquapath1",       // 0x0009
    "1_10": "wago_power1",    // 0x000A
    "1_22": "extruder3",      // 0x0016
  };
  return knownMachines[`${vendor}_${machine}`] ?? `unknown-${vendor}-${machine}`;
}

export default function MachinesPage(props: Props) {
  const machines = createMemo(() => props.mainState().machines);

  return (
    <div class="page">
      <h1>Machines</h1>
      <Show when={machines() === null} fallback={
        <Show
          when={(machines()?.length ?? 0) > 0}
          fallback={<p class="muted">No machines detected.</p>}
        >
          <div class="machine-list">
            <For each={machines()!}>
              {(m) => {
                const slug = machineSlug(m);
                const serial = m.machine_identification_unique.serial;
                return (
                  <div class={`machine-card ${m.error ? "error" : "ok"}`}>
                    <div class="machine-info">
                      <strong>{slug}</strong>
                      <span class="serial">#{serial}</span>
                      <Show when={m.error}>
                        <span class="error-badge">{m.error}</span>
                      </Show>
                    </div>
                    <div class="machine-links">
                      <A href={`/machines/${slug}/${serial}`}>Overview</A>
                      <A href={`/machines/${slug}/${serial}/control`}>Control</A>
                    </div>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>
      }>
        <p class="muted">Connecting to server…</p>
      </Show>
    </div>
  );
}
