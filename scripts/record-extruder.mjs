#!/usr/bin/env node
// Record an extruder heat-up to the CSV format the thermal simulation calibrates
// against (`machine_implementations/src/extruder1/simulation/data/`).
//
//   node scripts/record-extruder.mjs --serial 1234 --out heatup.csv
//
// Needs socket.io-client. The Electron app already depends on it, so the
// simplest way to run this is from that directory:
//
//   cd electron && node ../scripts/record-extruder.mjs --serial 1234 --out ../heatup.csv
//
// Subscribes to the machine's namespace and writes one row per second from the
// `LiveValuesEvent` the server already broadcasts at 30 Hz. Ctrl-C to stop; the
// file is flushed as it goes, so an interrupted run is still usable.
//
// IMPORTANT — the wattage caveat
//
// `LiveValuesEvent`'s `*_power` fields are `duty * heating_element_wattage`,
// using whatever wattage the firmware has configured — not a measurement. This
// script divides that back out to recover the duty cycle, which is what the
// simulation needs and what keeps the file independent of the constant being
// right. Check `--watt-*` against `machine_implementations/src/extruder1/new.rs`
// before recording, and against the heaters actually fitted. They have
// disagreed before: the 2026-02-24 reference run was logged with 900 W barrel
// and 150 W nozzle against real 700 W and 200 W hardware.

import { writeFileSync, appendFileSync } from "node:fs";
import { io } from "socket.io-client";

const argv = process.argv.slice(2);
const arg = (name, fallback) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 && argv[i + 1] ? argv[i + 1] : fallback;
};

if (argv.includes("--help")) {
  console.log(`Usage: node scripts/record-extruder.mjs [options]

  --serial <n>       machine serial number (required)
  --vendor <n>       vendor id (default 1, QiTech)
  --machine <n>      machine id (default 22 = 0x0016, the newer extruder;
                     use 4 = 0x0004 for the older one)
  --url <url>        server base url (default http://localhost:3001)
  --out <path>       output csv (default extruder-heatup.csv)
  --period <s>       seconds between rows (default 1)
  --watt-barrel <w>  rated watts per barrel band (default 700)
  --watt-nozzle <w>  rated watts for the nozzle band (default 200)
`);
  process.exit(0);
}

const serial = arg("serial");
if (!serial) {
  console.error("--serial is required (see --help)");
  process.exit(1);
}

const vendor = Number(arg("vendor", "1"));
const machine = Number(arg("machine", "22"));
const baseUrl = arg("url", "http://localhost:3001");
const out = arg("out", "extruder-heatup.csv");
const period = Number(arg("period", "1")) * 1000;
const wBarrel = Number(arg("watt-barrel", "700"));
const wNozzle = Number(arg("watt-nozzle", "200"));

// Matches serializeNamespaceId() in electron/src/client/socketioStore.ts
const namespace = `/machine/${vendor}/${machine}/${serial}`;

writeFileSync(
  out,
  [
    `# QiTech extruder heat-up recorded ${new Date().toISOString()}`,
    `# namespace ${namespace} at ${baseUrl}`,
    `# duty_* recovered from LiveValuesEvent *_power by dividing by the rated`,
    `# wattage: barrel ${wBarrel} W, nozzle ${wNozzle} W. Confirm these match the`,
    `# heaters actually fitted before trusting the file.`,
    `# t_s is seconds since the first sample with any zone drawing power.`,
    "t_s,T_front,T_middle,T_back,T_nozzle,duty_front,duty_middle,duty_back,duty_nozzle",
  ].join("\n") + "\n",
);

const socket = io(`${baseUrl}${namespace}`, { transports: ["websocket"] });

let t0 = null;
let lastWritten = -Infinity;
let rows = 0;

socket.on("connect", () =>
  console.log(`connected to ${namespace}; writing ${out}. Ctrl-C to stop.`),
);
socket.on("connect_error", (e) => console.error("connect error:", e.message));
socket.on("disconnect", (r) => console.error("disconnected:", r));

socket.on("event", (event) => {
  if (!event || event.name !== "LiveValuesEvent") return;
  const d = event.data;
  if (!d) return;

  const duty = [
    d.front_power / wBarrel,
    d.middle_power / wBarrel,
    d.back_power / wBarrel,
    d.nozzle_power / wNozzle,
  ];

  // Start the clock at the first sample where anything is actually heating, so
  // idle time before the operator hits Heat does not end up in the file.
  const heating = duty.some((v) => v > 0);
  const now = event.ts ?? Date.now();
  if (t0 === null) {
    if (!heating) return;
    t0 = now;
  }
  if (now - lastWritten < period) return;
  lastWritten = now;

  const t = (now - t0) / 1000;
  const row = [
    t.toFixed(2),
    d.front_temperature.toFixed(1),
    d.middle_temperature.toFixed(1),
    d.back_temperature.toFixed(1),
    d.nozzle_temperature.toFixed(1),
    ...duty.map((v) => v.toFixed(4)),
  ].join(",");
  appendFileSync(out, row + "\n");

  if (++rows % 60 === 0) {
    process.stdout.write(
      `\r${rows} rows, t=${t.toFixed(0)}s  ` +
        `F${d.front_temperature.toFixed(1)} M${d.middle_temperature.toFixed(1)} ` +
        `B${d.back_temperature.toFixed(1)} N${d.nozzle_temperature.toFixed(1)}  `,
    );
  }
});

process.on("SIGINT", () => {
  console.log(`\nwrote ${rows} rows to ${out}`);
  socket.close();
  process.exit(0);
});
