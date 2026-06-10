# Rewinder Qualitative Test Suites

These tests are meant to validate whether the rewinder is usable and smooth enough to continue tuning. They are not strict acceptance tests yet.

For every test, record:

- Starting source tension arm angle.
- Starting takeup tension arm angle.
- Speed changes.
- Whether it hard-stopped.
- Physical observation: smooth, shaky, clanky, delayed recovery, material slip, bad winding, etc.
- Relevant diagnostic lines around speed changes and failures.

Watch these diagnostic fields:

```text
puller_command
puller_actual
takeup_angle
source_angle
source_filtered
source_ratio
source_scale
takeup rpm
source rpm
```

Good signs:

- Source angle stays mostly inside `30..55 deg`.
- Takeup angle stays stable, ideally `45..65 deg`.
- `source_scale` drops below `1.0` before source reaches the low hard limit.
- `source_scale` rises above `1.0` before source reaches the high hard limit.
- `source_ratio` changes gradually.
- Source rpm changes smoothly.
- No repeated source rpm `0 -> high -> 0` during normal rewind.

Bad signs:

- Source crosses below `20 deg` or above `60 deg`.
- Takeup crosses below `20 deg` or above `90 deg`.
- `source_scale` stays near `1.00` while source is near a limit.
- Puller continues accelerating while source is already near a limit.
- Any hard stop before the test goal is reached.

## 1. Baseline Low Speed

Setup:

- Start source angle: preferably `40..50 deg`.
- Start takeup angle: preferably `45..55 deg`.
- Speed: `1 m/min`.
- Hold: `30..60 sec`.

Goal:

- Confirm stable low-speed rewinding before testing acceleration.

Current result:

- Status: **Pass qualitatively**.
- Sample duration: about `60 sec`.
- Hard stop: no.
- Takeup angle: stable around `53.4..54.1 deg`.
- Source angle: stayed legal, roughly `37..51 deg`.
- Puller actual: stable around `0.98..1.06 m/min`.
- Source scale: mostly `1.00`, with small high-angle boost around `1.01..1.06`.
- Source rpm: mostly `1.2..1.8 rpm`.
- Observation: source still shows slow hunting through the range, but not clanky or dangerous in this sample.

Verdict:

```text
Baseline 1 m/min is acceptable for moving to acceleration tests.
```

## 2. Small Acceleration

Setup:

- Start source angle: `40..50 deg`.
- Start takeup angle: `45..55 deg`.
- Speed shift: `1 -> 3 m/min`.
- Hold: `30 sec`.

Goal:

- Check whether source and takeup remain smooth during a small speed increase.

Pass if:

- No hard stop.
- Source remains mostly `30..55 deg`.
- Takeup remains stable.
- Source rpm does not twitch or repeatedly drop to zero.

Current result:

- Status: **Pass qualitatively**.
- Speed shift: `1 -> 3 m/min`.
- Hard stop: no in the provided sample.
- Puller actual: settled around `2.94..3.02 m/min` after the shift.
- Takeup angle: stable, mostly around `53.4..55.4 deg`.
- Source angle: stayed legal, roughly `36.5..51.5 deg`.
- Source scale: mostly `1.00`, with small high-angle boost around `1.03..1.08`.
- Source ratio: adjusted gradually, roughly `1.16..1.73` at low speed and `1.34..1.62` at `3 m/min`.
- Source rpm: around `3.9..4.8 rpm` once running at `3 m/min`.
- Observation: source still slow-hunts through the band, but the small acceleration did not amplify it into a hard stop or obvious runaway.

Verdict:

```text
1 -> 3 m/min is acceptable for moving to the medium acceleration test.
```

## 3. Medium Acceleration

Setup:

- Start from a stable low or small-speed state.
- Speed shift: `3 -> 8 m/min`.
- Hold: `30 sec`.

Goal:

- Check whether the source controller remains stable under moderate acceleration.

Pass if:

- Source may move, but recovers before reaching `20` or `60 deg`.
- `source_scale` reacts in the correct direction near bounds.
- No hard stop.

Current result:

- Status: **Pass qualitatively**.
- Speed shift: `3 -> 8 m/min`.
- Hard stop: no in the provided sample.
- At target `3 m/min`: takeup ranged `43.8..59.9 deg`, source ranged `31.5..51.9 deg`.
- At target `8 m/min`: takeup ranged `53.6..58.0 deg`, source ranged `35.5..52.1 deg`.
- Puller actual: settled around `7.99..8.07 m/min`.
- Source scale: reacted in both directions, roughly `0.95..1.14` at `8 m/min`.
- Source ratio: adjusted gradually, roughly `1.32..1.57` at `8 m/min`.
- Source rpm: mostly around `10.8..12.6 rpm` at `8 m/min`.
- Observation: source still oscillates through the band, but it does not touch hard limits and does not show repeated stop/start behavior in this run.

Verdict:

```text
3 -> 8 m/min is acceptable for moving to high acceleration.
```

## 4. High Acceleration Attempt

Setup:

- Start from a stable `8 m/min` run.
- Speed shift: `8 -> 15 m/min`.
- Hold: `30 sec`.

Goal:

- Find whether the current acceleration and source feed logic can support higher speed without source arm collapse.

Pass if:

- Source does not cross hard limits.
- If source approaches high limit, `source_scale > 1.0`.
- If source approaches low limit, `source_scale < 1.0`.

Current result:

- Status: **Pass qualitatively**.
- Speed shift: `8 -> 15 m/min`.
- Hard stop: no in the provided sample.
- At target `8 m/min`: takeup ranged `38.2..60.3 deg`, source ranged `31.1..51.9 deg`.
- At target `15 m/min`: takeup ranged `53.2..57.4 deg`, source ranged `37.9..51.3 deg`.
- Puller actual: settled around `14.93..15.00 m/min`.
- Source scale: mild high-angle correction only, roughly `1.00..1.06` at `15 m/min`.
- Source ratio: adjusted gradually, roughly `1.39..1.56` at `15 m/min`.
- Source rpm: continuous, mostly around `21.3..23.4 rpm` at `15 m/min`.
- Observation: this is the cleanest high-speed behavior so far. The source oscillation remains, but it stays comfortably away from both hard limits.

Verdict:

```text
8 -> 15 m/min is acceptable for moving to the aggressive target test.
```

## 5. Aggressive Target

Setup:

- Only run after tests 1-4 look acceptable.
- Speed shift: `15 -> 25 m/min`.
- Hold as long as safe.

Goal:

- Probe whether the controller is usable near production-style higher speeds.

Pass if:

- No immediate hard stop.
- Correction begins before hard limits.
- Motion remains physically acceptable.

## 6. Sharp Deceleration

Setup:

- Start from a stable medium/high speed.
- Speed shifts:

```text
15 -> 3 m/min
3 -> 1 m/min
```

Goal:

- Check whether sharp deceleration causes source or takeup instability.

Pass if:

- Source does not swing violently.
- Takeup does not fall or over-tighten.
- No hard stop.

## 7. Low Source Start

Setup:

- Start source angle: `35..38 deg`.
- Start takeup angle: `45..55 deg`.
- Speed shift: `1 -> 5 m/min`.

Goal:

- Verify that a low but legal source start does not immediately collapse below `20 deg`.

Pass if:

- Source recovers or remains legal.
- `source_scale` reduces feed as needed.

## 8. High Source Start

Setup:

- Start source angle: `52..55 deg`.
- Start takeup angle: `45..55 deg`.
- Speed shift: `1 -> 5 m/min`.

Goal:

- Verify that a high but legal source start does not immediately exceed `60 deg`.

Pass if:

- Source recovers or remains legal.
- `source_scale` boosts source feed as needed.

## 9. High-ish Takeup Start

Setup:

- Start takeup angle: `60..65 deg`.
- Start source angle: `40..50 deg`.
- Speed shift: `1 -> 5 m/min`.

Goal:

- Check whether takeup adaptive behavior is stable from a higher initial takeup angle.

Pass if:

- Takeup does not cross `90 deg`.
- Takeup does not abruptly fall below `20 deg`.
- Traverse and winding behavior remain acceptable.

## 10. Real Use Run

Setup:

- Start source angle: `40..50 deg`.
- Start takeup angle: `45..55 deg`.
- Speed shifts:

```text
1 -> 3 -> 8 -> 12 -> 18 m/min
```

- Hold each speed for `20..30 sec`.

Goal:

- Qualitatively validate the machine in a realistic speed progression.

Pass if:

- No hard stop.
- Source and takeup remain legal.
- Motion feels smooth enough for continued tuning.
- Winding quality is acceptable enough to proceed.
