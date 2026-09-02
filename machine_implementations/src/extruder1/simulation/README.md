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

## Melt transport

Until this was added the rig only ever modelled a **cold, idle machine**: a
conduction and radiation network with one actuator per zone, and the screw as a
static lump of steel. The largest disturbance the machine actually sees — cold
polymer entering at the feed throat, melting, and leaving as filament while the
screw drives mechanical work into it — was invisible, so "does the controller
hold setpoint when extrusion starts?" could not be asked at all.

Throughput is linear in screw speed at 0.1 kg/(h·rpm), so the machine's maximum
100 rpm is 10 kg/h.

### What it models

A polymer node per barrel cell, running **back → middle → front → nozzle**,
which is the direction material travels and the reverse of `Zone::port` order.
Cold pellets enter at the throat, each cell exchanges with the bore through a
melt film, viscous shear is injected along the screw, and enthalpy leaves the
model at the die.

Two things needed care:

- **Advection is not conduction.** `ThermalNetwork` had only symmetric
  conductances, so `control_core::thermal::Flow` was added: an asymmetric edge
  carrying the *upstream* enthalpy. It also makes the network an open system —
  energy enters at the throat and leaves at the die — which is why
  `stored_energy_j` is no longer a conservation check on its own.
- **Melting needs two different specific heats.** A lumped node cannot absorb
  heat at constant temperature, so the latent heat is smeared over a melting
  window as apparent capacity. A node's capacity then wants the *tangent*
  `dh/dT`, while a flow edge wants the *secant* against the enthalpy datum — only
  the secant transports true enthalpy rather than `cp·T`. Using the tangent for
  both would silently lose the entire latent heat across the melting range.

The screw geometry is measured, not assumed: `Oberbau_UBG.step` places the
`Schnecke` by a pure +879 mm translation, and its root runs Ø21.2 through
metering, tapers to Ø13.6 over 530 mm, and stays Ø13.6 through the feed section.
That is a textbook three-zone screw — deep where cold pellets enter, shallow
where melt is pumped into the die — and it holds about 0.35 kg of polymer, a
residence time near two minutes at 10 kg/h.

Making room for the polymer takes ~1.8 kg of steel out of the bore and puts
~0.35 kg of polymer in. Because polymer holds roughly four times the heat per
kilogram, the bore's total heat capacity barely moves, which is why the existing
nine-coefficient calibration survives. It is still not *identical*: with the melt
enabled and the screw stopped, the model is "the machine standing full of
stagnant polymer", which is the more honest state for a machine that has been
run, but is not the one the heat-up recording was fitted to.

### What it says

`--scenario extrude-start` heats to a normal profile, waits for the nozzle (over
half an hour — it is ~6 kg on a 200 W band), then starts the screw at 60 rpm.
Sweeping the one coefficient nobody has measured:

| SME kWh/kg | carried out | shear in | net | back load | front | nozzle |
|---|---|---|---|---|---|---|
| 0.00 | 507 W | 0 W | +507 W | +291 W | 180.0 (+0.3) | 175.0 (+0.3) |
| 0.05 | 560 W | 300 W | +260 W | +230 W | 183.8 (+3.8) | 179.9 (+4.9) |
| 0.10 | 695 W | 600 W | +95 W | +177 W | **213.8 (+33.8)** | **215.7 (+40.7)** |
| 0.20 | 1024 W | 1200 W | −176 W | +89 W | **297.6 (+117.6)** | **305.0 (+130.0)** |

Three things fall out of it, and the first two hold at *every* value:

- **The cooling is real, and it lands on the back zone.** Material enters at
  ambient against 170 °C steel, so the feed end does the melting work — 291 W
  down to 89 W as shear takes over. The back zone holds setpoint throughout, but
  its energy draw roughly doubles.
- **The front and nozzle do not get that cooling.** By the time the polymer
  reaches them it is already at or above barrel temperature, so it stops taking
  heat and starts giving it back. The melt load there goes *negative*.
- **Above about 0.05 kWh/kg the machine has no answer.** The loops can only stop
  heating; there is no barrel cooling. At 0.10 the front and nozzle settle 34 and
  41 K over setpoint with their relays already shut, and the melt leaves the die
  at 239 °C — hot enough to degrade PLA. At 0.20 it is 349 °C.

### What is not validated

**None of it.** The barrel model was fitted to an hour-long heat-up with the
screw stopped, and that recording contains no information about flow. The
geometry is measured and the structure is standard single-screw practice, but
every coefficient in `MeltParams` is a handbook value. In order of how much they
matter:

1. `specific_mechanical_energy_kwh_per_kg` — the table above is entirely a
   sweep of this one number, and it decides whether extruding cools the machine
   or cooks it. Handbook range 0.10–0.25 kWh/kg.
2. `kg_per_h_per_rpm` — plausible for this screw, never weighed.
3. `film_h` — mid-range; sets how hot the extrudate leaves.
4. `max_shear_power_w` — a guess at the drive's rating. The nameplate is not in
   the codebase.
5. `FEED_X_MM` — the throat is in the feed housing, which is not modelled.

**The recording that would fix this.** About 90 minutes, with
`scripts/record-extruder.mjs` extended to log `screw_rpm`:

1. Heat to a normal profile with the screw stopped, settle 20 minutes. Confirms
   the existing fit against fresh data.
2. Extrude at 30 rpm, hold 15 minutes. Then 60 rpm, then 100 rpm, 15 minutes each.
3. Stop the screw, hold 15 minutes.
4. **Weigh the extrudate over a timed interval at each speed.** That measures
   `kg_per_h_per_rpm` directly and takes it out of the fit.
5. If a pyrometer is to hand, take the extrudate temperature at each speed; that
   pins `film_h` on its own.

`MitsubishiCS80::motor_status` already reports frequency, current and voltage, so
logging those gives an independent check: the specific mechanical energy the
temperatures imply should agree with what the drive says it delivered.
