/**
 * Test machine control page — LED toggle proof of concept
 *
 * This replaces:
 *   electron/src/machines/testmachine/TestMachineControlPage.tsx
 *   electron/src/machines/testmachine/useTestMachine.ts        (~95 lines)
 *   electron/src/machines/testmachine/testMachineNamespace.ts  (~77 lines)
 *
 * In SolidJS the entire machine data + mutation logic lives here or in a
 * thin helper — no Zustand, no ThrottledStoreUpdater, no lifecycle hooks.
 *
 * Optimistic updates: SolidJS signals update synchronously, so the UI
 * reflects the change immediately. The server will confirm via the next
 * StateEvent pushed over WebSocket.
 */

import { For, Show, createSignal } from "solid-js";
import { useParams } from "@solidjs/router";
import { createTestMachineNamespace } from "../namespaces/testMachine";
import { mutateMachine } from "../lib/api";

// machine_identification for testmachine: vendor=1, machine=0x0033 (51)
const VENDOR = 1;
const MACHINE = 0x0033;

export default function TestMachineControlPage() {
  const params = useParams<{ serial: string }>();
  const serial = () => parseInt(params.serial);

  // createNamespace() creates the socket and returns a reactive signal.
  // The signal is updated whenever a StateEvent arrives.
  // The socket is disconnected automatically when this component unmounts.
  const [machineState] = createTestMachineNamespace(VENDOR, MACHINE, serial());

  // Optimistic LED state — start from server state, update immediately on click
  const [optimisticLeds, setOptimisticLeds] = createSignal<boolean[] | null>(null);

  // Derive: prefer optimistic state, fall back to server state
  const leds = () => optimisticLeds() ?? machineState().led_on;

  const setLed = async (index: number, on: boolean) => {
    // Optimistic update — instant UI feedback
    const current = leds();
    if (current) {
      const next = [...current];
      next[index] = on;
      setOptimisticLeds(next);
    }

    try {
      await mutateMachine(
        {
          machine_identification: { vendor: VENDOR, machine: MACHINE },
          serial: serial(),
        },
        { action: "SetLed", value: { index, on } },
      );
    } catch (e) {
      console.error("SetLed failed:", e);
      // Revert optimistic update on error
      setOptimisticLeds(null);
    }
  };

  const setAllLeds = async (on: boolean) => {
    const current = leds();
    if (current) {
      setOptimisticLeds(current.map(() => on));
    }

    try {
      await mutateMachine(
        {
          machine_identification: { vendor: VENDOR, machine: MACHINE },
          serial: serial(),
        },
        { action: "SetAllLeds", value: { on } },
      );
    } catch (e) {
      console.error("SetAllLeds failed:", e);
      setOptimisticLeds(null);
    }
  };

  return (
    <div class="page">
      <h1>Test Machine — Control</h1>
      <p class="serial">Serial: #{serial()}</p>

      <Show when={leds() !== null} fallback={<p class="muted">Waiting for machine state…</p>}>
        <div class="led-grid">
          <For each={leds()!}>
            {(on, i) => (
              <button
                class={`led-btn ${on ? "led-on" : "led-off"}`}
                onClick={() => setLed(i(), !on)}
              >
                LED {i() + 1}
                <span class="led-indicator">{on ? "●" : "○"}</span>
              </button>
            )}
          </For>
        </div>

        <div class="led-actions">
          <button onClick={() => setAllLeds(true)}>All On</button>
          <button onClick={() => setAllLeds(false)}>All Off</button>
        </div>
      </Show>
    </div>
  );
}
