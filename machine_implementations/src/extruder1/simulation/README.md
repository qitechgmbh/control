# Extruder heating simulation

Why this exists, what it says about the machine, and what came out of it. The
module docs cover how to use it; this is the engineering record.

## Calibration data

`data/heatup_2026-02-24.csv` is a real hour-long heat-up exported from the
machine: per-zone temperature and PID duty at 1 Hz, from 22 °C with setpoints
front 180, middle 180, back 170, nozzle 175 °C.

The firmware of the day reported power as `duty * heating_element_wattage` with
the **wrong** wattages configured (900 W barrel, 150 W nozzle; the hardware is
700 W and 200 W). The CSV therefore stores recovered *duty*, not watts, and the
model applies the real ratings from `Zone::rated_w`. If you re-export from the
machine, check that constant first — `scripts/record-extruder.mjs` writes this
format.

Refit against it with `--fit`; see the `fit` module.

## What the recording shows

Measured against those setpoints, the gains that shipped before this work
(`kp = 0.16, ki = 0, kd = 0.008`, identical on all four zones) give:

| zone | peak | overshoot | t90 | settles at |
|---|---|---|---|---|
| front | 194.4 | +14.4 K | 703 s | 181.0 |
| middle | 211.5 | **+31.5 K** | 674 s | 185.6 (still falling) |
| back | 186.6 | +16.6 K | 661 s | 168.4 |
| nozzle | 171.2 | **−3.8 K** (never arrives) | 1849 s | 171.2 |

Three effects the model has to reproduce, and why they happen:

- **Middle overshoots twice as hard.** It is flanked by heated zones, while front
  bleeds into the cold Düse across the flange and back bleeds into the 231 mm
  unheated tail and the gearbox. Same 700 W, same gains, much less escape.
- **Overshoot at all.** Two mechanisms of comparable size. The bands store energy
  and keep discharging into the steel after the relay opens — with every barrel
  zone at 0 W from ~790 s, middle still climbs another 26.5 K over the next
  360 s. And the RTDs lag the steel badly, so the controller is still driving
  after the steel has passed setpoint.
- **The nozzle never reaches setpoint.** With `ki = 0` the loop is pure
  proportional and parks at a droop of `duty / kp`; at ~97 W of a 200 W band that
  is 3.8 K short, forever. On the way up it is also clamped at `max_clamp = 0.95`
  for half an hour, being ~6 kg of uninsulated steel on a 34 mm band.

## How accurate is it

With `ExtruderThermalParams::calibrated`, closed loop against that recording:

| zone | peak sim / real | t90 sim / real |
|---|---|---|
| front | 195.2 / 194.4 | 695 / 703 s |
| middle | 211.7 / 211.5 | 637 / 674 s |
| back | 187.7 / 186.6 | 651 / 661 s |
| nozzle | 172.3 / 171.2 | 1875 / 1849 s |

Open-loop replay RMS over the hour is ~5.7 K. Asserted by the tests in `harness`.

**What this does and does not license.** The model reproduces the machine's
*behaviour* well, so it is a sound rig for comparing control strategies — which
is what it is for. The individual coefficients in `params` are a different
matter: one run with all four zones heating together cannot identify nine of
them, and three sit on a bound (see `ExtruderThermalParams::EXPECTED_PINNED`). Do
not quote them as measurements. Recording the `single-*` scenarios in `scenario`
on the real machine — one zone from cold, then decay — would separate them.

## The overshoot, and what actually fixed it

Not the gains. The RTDs have a time constant of order **150 s** and the barrel
ramps at ~0.23 K/s, so the controller spent the whole heat-up reading a value
about **34 K stale** and shut off that far late. That single number is
essentially the entire +31.5 K middle-zone overshoot, and no PID on the raw
signal can remove it: from inside the loop, a stale reading is indistinguishable
from being genuinely that far away.

What ships now is `control_core::controllers::heating::ObserverPi` —
reconstruct the steel temperature from the reading and its filtered slope,
regulate *that*, over a feedforward that already knows the duty the setpoint
costs.

Worst overshoot on profile B, across all four plants in `harness::plant_family`
(`--example bench_heating`):

| plant | front | middle | back | nozzle |
|---|---|---|---|---|
| PID, tau=20 | +5.1 | +6.3 | +2.4 | +0.1 |
| PID, tau=60 | +8.7 | +9.7 | +4.3 | +0.5 |
| PID, tau=100 | +10.4 | +13.2 | +6.1 | +0.5 |
| PID, tau=150 | +8.9 | **+15.9** | +6.6 | +0.1 |
| ObserverPi, tau=20 | +0.3 | +0.2 | +0.2 | +0.2 |
| ObserverPi, tau=60 | +0.7 | +0.2 | +0.4 | +0.2 |
| ObserverPi, tau=100 | +0.8 | +0.2 | +0.4 | +0.2 |
| ObserverPi, tau=150 | +0.3 | +0.2 | +0.1 | +0.2 |

Summed over three profiles and four plants, the benchmark's cost — seconds to
settle plus 40 s per K of overshoot beyond each zone's budget — goes from
**98 228 to 59 665**. Rise times and energy are unchanged (front t90 694 → 700 s,
middle 646 → 681 s; per-zone energy within a few percent either way), so the
overshoot is not being bought with either. The shipping configuration is
asserted by the tests in `shipping.rs`.

**Still worth doing on the machine.** A 150 s probe lag is what one sitting in an
air gap does, not one that is properly seated. Fixing the seating with
heat-transfer compound would remove the *cause* rather than compensating for it,
and would make everything here work better.

## The plant family

The model is fitted to one recording, and that recording cannot separate the
probe's lag from the band's heat capacity — both delay the reading. The optimiser
stopped at a 150 s probe with a nearly weightless band, but a 20 s probe with a
heavy band fits just as well and is a physically very different machine: there
the overshoot is stored energy rather than measurement lag.

`harness::plant_family` spreads four plants along exactly that unresolved axis,
and every strategy is scored across all of them rather than the nominal fit
alone. The shipping controller holds up on all four, which is the point of
checking.

## Why the cascade is not shipped

A cascade controller (outer loop sets a band temperature, inner loop delivers it,
both fed by an observer that splits band from metal) was built, tuned and
benchmarked before `ObserverPi` was chosen. It reaches setpoint about as cleanly
— worst overshoot under a kelvin on three of the four plants — but it *creeps*,
taking three to six thousand seconds to arrive where `ObserverPi` takes about
eight hundred, and on the benchmark's cost that was decisive: ~202 000 against
`ObserverPi`'s ~60 000.

That is a property of this machine, not of cascade control. A cascade earns its
inner loop by bounding the band's stored energy, and here that energy is small:
~41 J/K of band sitting ~175 K above ~2.5 kJ/K of steel is about 3 K of coast.
Bounding a 3 K effect does not pay for the lag the extra loop adds. The dominant
defect is the 150 s measurement lag, and that wants an estimator, not another
loop.

Worth revisiting if the hardware changes in a way that inverts that: a band with
much more mass, or — more likely — properly seated probes, which would shrink the
measurement lag and leave band storage as what remains. The implementation was
removed rather than carried as dead code; recover it from the history of
`control-core/src/controllers/heating/` if that day comes.

## Controller defects this rig exposed

All were found here rather than on the machine, and all are fixed. Kept as a
record of what the rig is for.

1. **The derivative term was quantisation noise.** `update` runs every ~1 ms on a
   value the EL3204 refreshes every few tens of ms in 0.1 °C steps. When it moved,
   `ed = 0.1 / 0.001 = 100 K/s` and `kd * ed = 0.8` — most of the duty range, from
   one LSB. Fixed by never differentiating the raw reading: `SensorLagObserver`
   low-passes first and takes the slope of the filtered signal, which needs no
   division by `dt` at all.
2. **No anti-windup.** Fixed by `PidController::update_with_antiwindup`; the
   feedforward now also means the integral only ever carries a small residual.
3. **One set of gains for four very different plants.** Fixed: every parameter is
   per zone, in `extruder1::heating_params`.
4. **PWM window off-by-one.** `elapsed` was not recomputed after
   `window_start = now`, so the relay was forced off for one tick per window.
   Fixed in `TemperatureController::update`.
5. **A failed sensor read decoded as 0 °C**, which is maximum error, which is full
   heat demand — the heater running flat out exactly when nothing could see how
   hot it was getting. Fixed: `wiring_error` now opens the relay.

## Relay autotuning

`control_core`'s `PidAutoTuner` already existed but was only wired to pressure.
`harness::ThermalSim::run_autotune` points it at a simulated zone, so temperature
autotuning can be developed without tying up the machine for an hour per attempt:

```bash
cargo run --release -p machine_implementations --features simulation \
    --example extruder_thermal_sim -- --autotune middle
```

## Plotting a run

`--out run.csv` writes one row per second with `setpoint / sensor / steel / band /
duty / power_w` per zone. Open `scripts/plot-heating.html` in a browser and drop
the file on it — no build step, and it reads the recorded-run format too.

**The pair worth plotting first is `sensor_*` against `steel_*`.** The gap between
them is the whole problem. `band_*` is the second story: the band runs ~175 K
above the steel and keeps discharging after the relay opens.
