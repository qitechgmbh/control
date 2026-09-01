# Extruder heating traces

Simulated runs of the four heating zones, written by the offline thermal model
so the new control law can be looked at next to the old one. **Generated
output — nothing reads these, delete them freely.**

Regenerate any of them with:

```bash
CARGO_TARGET_DIR=/tmp/control_build cargo run --release \
    -p machine_implementations --example extruder_thermal_sim -- \
    --scenario recorded-heatup --strategy observer-pi --out some.csv
```

## Columns

`t_s`, then one group of four in the order **front, middle, back, nozzle**:

| group | what |
|---|---|
| `setpoint_*` | what was being asked for at that instant |
| `sensor_*` | what the controller saw, after the EL3204's 0.1 °C quantisation |
| `steel_*` | what the barrel steel was *actually* at |
| `band_*` | the band heater itself |
| `duty_*` | demanded duty, 0..1 |
| `power_w_*` | electrical power actually delivered |

Sampled at 1 Hz, matching `simulation/data/heatup_2026-02-24.csv` from the real
machine.

**The pair worth plotting first is `sensor_*` against `steel_*`.** The gap
between them is the whole problem: the RTDs trail the steel by ~150 s, so on a
0.23 K/s ramp the controller is reading a value about 34 K stale and shuts off
that far late. `band_*` is the second story — the band runs ~175 K above the
steel and keeps discharging after the relay opens.

## Files

`recorded-heatup` uses the same setpoints as the real 2026-02-24 recording
(front 180, middle 180, back 170, nozzle 175, from 22 °C), so these are directly
comparable against that file.

| file | plant | controller |
|---|---|---|
| `recorded-heatup_pid.csv` | nominal | the PID that ships today |
| `recorded-heatup_observer-pi.csv` | nominal | **the new one** |
| `recorded-heatup_cascade.csv` | nominal | cascade — built, tested, not shipped |
| `step-up_pid.csv` | nominal | old, heat to 180 then step to 200 at t=5400 |
| `step-up_observer-pi.csv` | nominal | new, same |
| `altplant-tau20_pid.csv` | alternative | old |
| `altplant-tau20_observer-pi.csv` | alternative | new |

### recorded-heatup, peak overshoot

| zone | PID | ObserverPi | energy, PID → new |
|---|---|---|---|
| front | +10.2 K | **+0.3 K** | 0.176 → 0.181 kWh |
| middle | **+15.2 K** | **−0.0 K** | 0.128 → 0.120 kWh |
| back | +4.2 K | **+0.1 K** | 0.191 → 0.196 kWh |
| nozzle | +0.1 K | +0.1 K | 0.152 → 0.151 kWh |

Rise times are unchanged (front 694 → 700 s, middle 646 → 681 s), and so is the
energy. The overshoot is not being bought with either.

### The alternative plant

`altplant-tau20_*` is the same machine under the *other* reading of the
calibration data. The model is fitted to one recording, and that recording
cannot separate the probe's lag from the band's heat capacity — both delay the
reading. The optimiser stopped at a 150 s probe with a nearly weightless band,
but a 20 s probe with a heavy band fits the recording just as well, and is a
physically very different machine: there the overshoot is stored energy rather
than measurement lag.

The new controller holds up on both (+0.3 / −0.0 / +0.2 there), which is the
point of checking. See `harness::plant_family` for the full set of four, and
`simulation::tuning`'s tests for what is asserted about them.

### Why the cascade is not shipped

`recorded-heatup_cascade.csv` is included so the decision is inspectable rather
than asserted. Its overshoot is fine, but look at where the zones end up: back
settles at 163.6 against a 170 setpoint and the nozzle at 169.6 against 175 —
it creeps, taking three to six thousand seconds to arrive where ObserverPi takes
about eight hundred.

That is a property of this machine, not of cascade control. A cascade earns its
inner loop by bounding the band's stored energy, and here that energy is small:
~41 J/K of band sitting ~175 K above ~2.5 kJ/K of steel is about 3 K of coast.
Bounding a 3 K effect does not pay for the lag the extra loop adds. Worth
revisiting if the probes are ever properly seated, which would shrink the
measurement lag and leave band storage as the dominant term.
