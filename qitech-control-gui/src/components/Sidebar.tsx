import { A } from "@solidjs/router";
import { For, Show, createMemo } from "solid-js";
import type { MainState, MachineObj } from "../namespaces/main";

type Props = {
  mainState: () => MainState;
};

function machineLabel(m: MachineObj): string {
  const { vendor, machine } = m.machine_identification_unique.machine_identification;
  const labels: Record<string, string> = {
    "1_51": "Test Machine",   // 0x0033
    "1_2": "Winder v2",       // 0x0002
    "1_4": "Extruder v2",     // 0x0004
    "1_6": "Laser v1",        // 0x0006
    "1_7": "Mock",            // 0x0007
    "1_8": "Buffer v1",       // 0x0008
    "1_9": "Aquapath v1",     // 0x0009
    "1_10": "Wago Power",     // 0x000A
    "1_22": "Extruder v3",    // 0x0016
  };
  return labels[`${vendor}_${machine}`] ?? `Machine ${vendor}/${machine}`;
}

function machineSlug(m: MachineObj): string {
  const { vendor, machine } = m.machine_identification_unique.machine_identification;
  const slugs: Record<string, string> = {
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
  return slugs[`${vendor}_${machine}`] ?? `unknown-${vendor}-${machine}`;
}

// Slugs that have a graph page implemented
const SLUGS_WITH_GRAPH = new Set(["mock1"]);

export default function Sidebar(props: Props) {
  const machines = createMemo(() => props.mainState().machines ?? []);

  return (
    <nav class="sidebar">
      <div class="sidebar-brand">
        <span>QiTech Control</span>
      </div>

      <div class="sidebar-section">
        <A href="/" class="sidebar-link" end>
          Machines
        </A>
      </div>

      <Show when={machines().length > 0}>
        <div class="sidebar-section">
          <div class="sidebar-section-label">Active Machines</div>
          <For each={machines()}>
            {(m) => {
              const slug = machineSlug(m);
              const serial = m.machine_identification_unique.serial;
              return (
                <div class="sidebar-machine">
                  <A href={`/machines/${slug}/${serial}`} class="sidebar-link" end>
                    {machineLabel(m)} <span class="sidebar-serial">#{serial}</span>
                  </A>
                  <A href={`/machines/${slug}/${serial}/control`} class="sidebar-sublink">
                    Control
                  </A>
                  <Show when={SLUGS_WITH_GRAPH.has(slug)}>
                    <A href={`/machines/${slug}/${serial}/graph`} class="sidebar-sublink">
                      Graph
                    </A>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </Show>

      <div class="sidebar-footer">
        <div class={`connection-dot ${props.mainState().machines !== null ? "connected" : "disconnected"}`} />
        <span>{props.mainState().machines !== null ? "Connected" : "Connecting…"}</span>
      </div>
    </nav>
  );
}
