# Rewinder Control Review Prompt

This document is a self-contained prompt/report for asking another LLM or controls engineer to review the QiTech Rewinder control approach. It assumes the reviewer has no prior knowledge of this codebase.

## Copy-Paste Prompt

You are reviewing the motion-control strategy for a glass-fiber rewinder. The machine is being developed by adapting ideas from an existing machine called Winder2, but the rewinder geometry is different enough that the Winder2 algorithm may not transfer directly.

I need you to review the system from first principles and propose a robust, non-aggressive, flexible control algorithm that can work across many initial tension-arm angles, changing spool diameters, and speed/acceleration changes during rewinding. I want a serious controls-oriented answer, not just parameter tuning. Please reason about safe angle ranges, initial angle requirements, hard stops vs soft correction, acceleration limiting, source/takeup spool coordination, and what data should be logged to validate the algorithm.

## What We Are Trying To Make

We are building a rewinder for fragile glass fiber. The operator transfers material from a source spool to a takeup spool. The machine must:

- Pull material at a commanded line speed.
- Pay out material from the source spool without over-tensioning or dumping slack.
- Wind material onto a takeup spool with reasonable tension and traverse lay.
- Keep two tension arms inside safe mechanical ranges.
- Avoid sudden motor kicks, stop/start oscillations, and aggressive tension corrections because glass fiber is fragile.
- Work across changing source/takeup spool diameters.
- Handle speed target changes during rewind, e.g. low speed to medium/high speed and deceleration.

The current goal is a guaranteed baseline algorithm that is smooth and robust. Fine tuning can come later.

## Fixed Constraints

Do not propose solutions that require replacing the major codebase framework, changing the EtherCAT hardware stack, or redesigning the physical machine layout. Those are fixed for this review.

Fixed software/framework constraints:

- The implementation is in Rust inside the existing QiTech machine framework.
- The machine integrates through the existing `Machine`, `MachineNew`, API/event, and frontend state/mutation structure.
- The control loop is called from the existing machine `act` cycle.
- The solution should fit into the current mode model: `Standby`, `Hold`, `Pull`, `Rewind`, with internal rewind phases if needed.
- The frontend can expose settings and diagnostics, but the core architecture should remain compatible with the existing API/state/event pattern.
- Winder2 code may be reused or adapted, but the solution must not require a new control framework.

Fixed hardware constraints:

- Beckhoff EtherCAT hardware remains as listed below: EK1100, EL2002, EL7041-0052, EL7031, and EL7031-0030 modules.
- The puller, source spool, takeup spool, traverse, and two tension arms are the available actuators/sensors.
- There is no laser diameter sensor for the rewinder.
- The source and takeup spool diameters may vary, but they are not directly measured by a dedicated diameter sensor.
- Stepper motors and current EtherCAT velocity-control mode remain the actuation method.

Fixed mechanical structure:

- Material path remains: source spool -> source tension arm -> puller -> takeup tension arm -> traverse -> takeup spool.
- The machine has one source spool and one takeup spool.
- The source tension arm geometry is different from the Winder2 tension arm geometry.
- The takeup side is closer to Winder2 but still part of a two-tension-arm rewinder system.
- The review may recommend start-angle requirements, setup/calibration routines, and control logic, but not moving the physical tension arms or changing the routing as a primary solution.

## Operator Workflow

The workflow is currently:

1. Install wire/fiber on source spool.
2. Set the two tension arms to their zero reference in the frontend.
3. Lift/position the arms on their supports while threading.
4. Press Pull. The puller moves the material so the operator can thread it through the traverse and manually install it on the takeup spool.
5. Stop Pull when threading is complete.
6. Lower the tension arms onto the wire/fiber.
7. Press Rewind.

Important workflow conclusions learned during testing:

- The Pull mode should feel like Winder2 Pull mode: simple, smooth puller motion.
- Source spool assist during Pull was tried but made Pull clanky. It was removed/disabled. In Pull mode the source spool should not be energized for now.
- Rewind starts only after both tension arms are lowered and legal.

## Reference Machine: Winder2

Winder2 is the reference because it already winds smoothly. It has:

- One puller.
- One spool.
- One tension arm.
- A traverse.
- Optional laser features, which are not relevant to rewinder.

Winder2 spool control has two modes:

- Min/Max: converts tension arm angle into a speed between min and max spool speed.
- Adaptive: estimates the spool speed from puller speed and an adaptive learned speed factor, then uses tension error to update the factor.

Relevant Winder2 ideas:

- Puller is the master of line speed.
- Spool speed is smoothed through an acceleration controller.
- Tension arm angle is converted into normalized filament tension.
- Adaptive spool speed uses the relationship between linear speed and angular speed:

```text
angular_speed ~= line_speed / effective_radius
```

Winder2 adaptive controller code concept:

```rust
fn get_max_speed(&self, puller_speed_controller: &PullerSpeedController) -> AngularVelocity {
    AngularVelocity::new::<radian_per_second>(
        (puller_speed_controller.last_speed.get::<meter_per_second>()
            / self.speed_factor.get::<meter>())
            * self.max_speed_multiplier,
    )
}
```

Winder2 MinMax spool controller concept:

```rust
// If the arm clamps outside the range, return min or max speed depending on response.
// Otherwise interpolate between min and max speed based on normalized tension.
let filament_tension = filament_calc.calc_filament_tension(clamped_angle);
let factor = tension_response.speed_factor(filament_tension);
speed = scale(factor, min_speed, max_speed);
```

Winder2 tension response direction:

```rust
pub enum SpoolTensionResponse {
    Takeup,
    Source,
}

// Takeup and source respond in opposite directions.
// Takeup: higher tension generally means reduce takeup speed.
// Source: higher tension generally means increase payout speed.
```

The core issue: the rewinder has two spools and two tension arms, and the source arm geometry appears not compatible with the Winder2 assumptions.

## Rewinder Hardware And EtherCAT Technology

The code uses Beckhoff EtherCAT devices through `qitech_lib`:

- EK1100 EtherCAT coupler.
- EL2002 digital output module.
- EL7041-0052 for the takeup spool stepper.
- EL7031 for the traverse stepper.
- EL7031-0030 for the puller stepper.
- EL7031-0030 for the source spool stepper.

Stepper configuration:

- Puller and source spool EL7031-0030:
  - Direct velocity mode.
  - Speed range `Steps1000`.
  - Motor max current `2700`.
  - Velocity control compact PDO.
- Traverse EL7031:
  - Direct velocity mode.
  - Speed range `Steps1000`.
  - Motor max current `1500`.
  - Velocity control compact PDO.
- Takeup spool EL7041-0052:
  - Direct velocity mode.
  - Motor max current `2800`.

Converters and mechanical assumptions in code:

- Puller line speed uses `LinearStepConverter::from_diameter(200, 8 cm)`.
- Takeup spool step converter uses `AngularStepConverter::new(200)`.
- Source spool step converter uses `AngularStepConverter::new(200)`.
- Traverse initial range is approximately `22 mm` to `92 mm`.
- Traverse controller initial parameter appears to use `64` as a step/count parameter.

Gear ratio:

```rust
pub enum GearRatio {
    OneToOne,
    OneToFive,
    OneToTen,
}

impl GearRatio {
    pub fn multiplier(&self) -> f64 {
        match self {
            GearRatio::OneToOne => 1.0,
            GearRatio::OneToFive => 5.0,
            GearRatio::OneToTen => 10.0,
        }
    }
}
```

UI intent:

- `1:1` should allow up to about `50 m/min`.
- `1:5` should allow up to about `10 m/min`.
- `1:10` should allow up to about `5 m/min`.

Please verify whether this interpretation of gear ratio and speed limiting is correct for this kind of machine.

## Full Mechanical System

The rewinder has:

- Source spool: holds the material being unwound.
- Source tension arm: measures/interacts with the source-side tension between source spool and puller.
- Puller: establishes commanded line speed in m/min.
- Takeup tension arm: measures/interacts with the material after puller and before takeup/traverse path.
- Traverse: positions material laterally on the takeup spool.
- Takeup spool: winds material onto the destination spool.

Approximate material path:

```text
source spool -> source tension arm -> puller -> takeup tension arm -> traverse -> takeup spool
```

Important geometry lesson from testing:

- The source tension arm does not behave like the Winder2 spool-side tension arm.
- Source angle around `50..52 deg` appears to be normal/acceptable in the current physical setup.
- Treating `51..52 deg` as a near-fault caused clanky limit cycling.

The exact sign convention must be checked mechanically, but current interpretation:

- For the source arm, high angle means the source side needs more payout or less puller acceleration.
- For the source arm, low angle means possible slack/overfeed or too much source payout.
- For takeup, Winder2-style adaptive control is still closer to correct.

## Target Material

The target material is glass fiber, which is fragile. Consequences:

- Sudden tension spikes can damage or break it.
- Sudden slack followed by re-tension can also be bad.
- Motor commands should be smooth, ramped, and predictable.
- A hard stop is still necessary for dangerous out-of-range angles, but the control should act before hard stops.
- The system should avoid aggressive stop/start recovery loops.

Testing may use plastic wire or other less fragile material first. Glass-fiber parameters are not yet validated.

## Experimental Setup

Testing is physical/manual, not simulation-based. The setup varies:

- Source spool diameter changes over a rewind and differs by spool.
- Takeup spool diameter changes as material accumulates.
- Initial source tension arm angle varies depending on how much material is between source spool and puller.
- Initial takeup tension arm angle varies depending on routing and installed length between puller, traverse, and takeup spool.
- Operator may start in different legal angles.
- Speed changes happen during rewind: examples tested include 1 -> 3, 3 -> 8, 8 -> 15, 15 -> 25, and direct standby -> 15.

Qualitative test logs currently print:

```text
phase
puller_target m/min
puller_command m/min
puller_actual m/min
takeup_angle deg
source_angle deg
source_filtered deg
source_ratio rpm_per_m_per_min
source_scale
takeup rpm
source rpm
can_rewind
reason
```

## Current Legal And Start Angle Ranges

Current hard limits in code:

```rust
source hard range: 20..60 deg
takeup hard range: 20..90 deg
```

Current start windows:

```rust
source rewind start range: 35..55 deg
takeup rewind start range: 35..70 deg
```

Open question for reviewer:

- Should Rewind start from any legal angle and let the controller converge?
- Should the operator be forced into a narrow start window, e.g. around `45..50 deg`?
- Should there be an automated setup/calibration action that moves source/takeup/puller slowly until both arms are in a desired target window?
- How tolerant should the runtime legal range be if material is fragile?
- Should hard stop remain exactly 20..60 for source and 20..90 for takeup, with only soft controls inside the band?

## Current Rewinder Algorithm

The current rewinder code is an experimental hybrid:

- Puller has a commanded speed ramp separate from the UI target.
- Puller command ramps toward UI target unless source/takeup angles approach warning zones.
- Takeup spool uses the existing Winder2 adaptive spool controller.
- Source spool does not currently use Winder2 adaptive spool controller directly in Rewind. It uses custom feed-forward plus angle correction.
- Source ratio is an estimated `rpm per m/min` factor.
- Source angle is filtered.
- Source ratio learning is gated to avoid learning during unstable arm angles.

Current source/puller constants:

```rust
source hard: 20..60 deg
takeup hard: 20..90 deg
source start: 35..55 deg
takeup start: 35..70 deg

puller normal ramp: 1.0 m/min/s
puller low-margin ramp: 0.5 m/min/s
puller source-high ramp: 0.5 m/min/s
puller backoff: -3.0 m/min/s

source puller backoff low angle: <=28 deg
source puller slow low angle: <=30 deg
source puller slow high angle: >=54 deg
source puller backoff high angle: >=58 deg

source target angle: 50 deg
source initial ratio: 1.5 rpm per m/min
source min ratio: 0.8 rpm per m/min
source max ratio: 4.0 rpm per m/min
source ratio learning allowed only above 5 m/min
source ratio learning angle window: 42..54 deg

source high boost starts: 54 deg
source high boost full: 58 deg
source high boost max multiplier: 1.5
source recovery starts: 56 deg
source recovery full: 59 deg
source recovery rpm floor: 12 rpm

source angle filter:
  rise time constant: 0.25 s
  fall time constant: 0.9 s
source deadband: 4 deg
source ratio increase/decrease rate: 0.02 per deg per sec
source rpm slew: 24 rpm/s
```

Simplified current source command logic:

```rust
line_speed = puller_command_speed_m_per_min
filtered_source_angle = asymmetric_filter(raw_source_angle)

if line_speed > 5.0
   && raw/filtered source angle both inside 42..54:
    source_ratio += angle_error_to_target * learning_rate * dt
    source_ratio = clamp(source_ratio, 0.8, 4.0)

source_low_scale = smoothstep((low_guard_angle - 24) / (35 - 24))
source_drop_scale = 1 - smoothstep((angle_drop - 5) / (15 - 5))
source_high_boost = 1 + (1.5 - 1) * smoothstep((high_guard_angle - 54) / (58 - 54))

source_scale = min(source_low_scale, source_drop_scale) * source_high_boost
feed_forward_rpm = line_speed * source_ratio * source_scale
high_recovery_rpm = 12 * smoothstep((high_guard_angle - 56) / (59 - 56))
target_rpm = max(feed_forward_rpm, high_recovery_rpm)
source_command_rpm = slew_limit(source_command_rpm, target_rpm, 24 rpm/s)
```

Simplified puller ramp logic:

```rust
if source >= 58 || takeup >= 85 || source <= 28:
    puller ramp rate = -3.0 m/min/s
else:
    ramp rate = 1.0 m/min/s
    if source <= 30 || takeup <= 25:
        ramp rate = min(ramp rate, 0.5)
    if source >= 54 || takeup >= 80:
        ramp rate = min(ramp rate, 0.5)
```

## Problems Encountered

The current approach has gone through many iterations. Observed problems:

1. Early source assist during Pull made Pull mode clanky. It was removed.
2. Directly using Winder2-like tension interpretation on the source spool caused wrong behavior because source geometry is different.
3. Hard stops triggered during acceleration because source or takeup arms moved rapidly to limits.
4. Source ratio learning initially overlearned during transients, e.g. climbing from about `1.5` to near `3.0`, causing overfeeding or later instability.
5. After gating learning, the source ratio stayed stable, but correction thresholds around `51..52 deg` caused puller command limit cycling: puller would ramp up, hit threshold, back down, then ramp again.
6. Latest interpretation is that source `50..52 deg` may be a normal operating region, so high correction should begin later.
7. The system is still not proven smooth across all initial angles and speed shifts.
8. There is visible/physical clankiness on the source side in some tests.
9. The current algorithm may be too ad hoc: feed-forward, scaling, learning, high recovery, and puller backoff are all interacting.

Recent testing lesson:

- When source ratio stayed fixed at `1.50`, source angle around `51..52 deg` was stable but the controller treated it too aggressively.
- Moving target to `50 deg` and high correction later may help, but we are pausing to review whether the entire approach is right.

## What We Need From The Reviewer

Please propose a robust control architecture for this rewinder. Address these questions explicitly:

1. Should puller be the only master speed, with source and takeup as tension regulators?
2. Should source spool control be feed-forward plus slow feedback, PID-like, state-machine-based, or adaptive-radius-based?
3. Should takeup remain Winder2 adaptive, or should both spools use a unified rewinder-specific controller?
4. What should the source tension arm target angle be, given hard range 20..60 and observed normal behavior around 50..52?
5. What should the takeup tension arm target angle be, given hard range 20..90?
6. What initial angle windows should be required before Rewind can start?
7. Should the system provide an automatic setup/calibration button that slowly moves source/takeup/puller to settle both arms before enabling Rewind?
8. Should hard limits remain hard, or should there be wider software tolerance with soft recovery?
9. What soft zones should exist inside the hard limits?
10. How should speed changes be handled, especially acceleration from low to high speed and deceleration from high to low?
11. How should changing spool diameters be handled without a laser diameter sensor?
12. Is adaptive diameter learning from tension behavior viable here, or too risky for fragile material?
13. What should be logged to identify which actuator is causing an angle excursion?
14. How should the algorithm avoid source spool start/stop twitching?
15. What should the fallback behavior be when a tension arm goes out of range?

Please give:

- A recommended control algorithm.
- A clear state diagram or phase description.
- Equations/pseudocode for source spool rpm and takeup spool rpm.
- Suggested angle targets and soft/hard zones.
- Suggested initial start requirements.
- Suggested acceleration/deceleration limits.
- Suggested anti-windup and filtering strategy.
- A test plan to validate it on plastic wire first and glass fiber later.
- A list of parameters that should be tunable vs fixed.

## Non-Negotiable Safety/Usability Requirements

- Glass fiber must not see sudden aggressive tension changes.
- Motors should not repeatedly stop/start during normal rewind.
- Pull mode must remain simple and smooth.
- Hard stops should still stop the process when the system is truly unsafe.
- The algorithm must be understandable enough for a startup team to tune and maintain.
- The algorithm must work across changing spool diameters.
- It must handle speed target changes during rewind.

## Unimplemented / Missing Features

These are not the immediate control-loop blocker, but they are missing from the rewinder compared with Winder2:

1. Gear-ratio-aware frontend speed limit:
   - `1:1` max `50 m/min`
   - `1:5` max `10 m/min`
   - `1:10` max `5 m/min`

2. Dedicated Rewinder settings page:
   - Move tuning controls away from the main operation page.

3. Source spool tuning controls:
   - Expose source tension target, learning rate, max speed multiplier, acceleration factor, deacceleration urgency multiplier if source uses adaptive control.

4. Pull source assist parameterization:
   - Currently source assist during Pull is disabled because it was clanky.
   - If reintroduced, expose estimated diameter, underfeed factor, and max assist rpm.

5. Length tracking:
   - Track rewound length from puller speed over time.
   - Add reset progress.
   - Add display/history graphs.

6. Automatic stop/hold by target length:
   - Stop or hold after configured meters.

7. Presets:
   - Save material/process settings once glass-fiber parameters are validated.

8. Glass-fiber parameter validation:
   - Mechanical/electrical team must validate legal angle ranges, speed range, acceleration/deceleration limits, and acceptable traverse lay.

9. Operator documentation:
   - Setup, zeroing, Pull threading, Rewind start requirements, fault recovery, and glass-fiber limits.

Explicitly omitted Winder2 features:

- Laser diameter measurement.
- Laser-based adaptive puller regulation.
- Laser pointer controls.
- Diameter reference machine selection.

Reason: this rewinder transfers existing material; core problem is tension and speed coordination, not extrusion diameter regulation.

## Current Ask

Given all the above, review whether our current Winder2-derived approach should continue or whether the rewinder needs a ground-up source/takeup control design. If redesigning, propose the simplest robust algorithm that can:

- Start from reasonable initial angles.
- Ramp smoothly to many speed targets.
- Tolerate acceleration/deceleration changes.
- Keep source within 20..60 deg and takeup within 20..90 deg.
- Avoid clankiness and source spool twitching.
- Protect fragile glass fiber.

## Candidate Redesign Direction

The most promising redesign direction is to stop layering aggressive source-specific boost/recovery logic onto the Winder2 algorithm and instead implement a phase-gated follower architecture.

Core philosophy:

- Puller is the only master line-speed axis.
- Source spool is a velocity follower regulated by source arm angle.
- Takeup spool is a velocity follower regulated by takeup arm angle.
- Tension arms are buffers, not fast error signals.
- Feed-forward should do most of the work.
- Arm feedback should provide slow bounded trim.
- All commands must be slew-limited.
- No aggressive stop/start correction.
- No source assist in Pull mode.
- No source boost/recovery zones initially.
- No integral or derivative control initially.

Recommended internal phases:

```rust
enum RewindPhase {
    Standby,
    Pull,
    Validate,
    Precharge,
    CrawlStart,
    Rewind,
    ControlledHold,
    FaultHold,
}
```

The UI can still expose only `Standby`, `Hold`, `Pull`, and `Rewind`; these internal phases are implementation details.

Phase behavior:

- `Standby`: all motors stopped; no learning; no arm correction.
- `Pull`: only puller moves; source and takeup spools off; no learning.
- `Validate`: before Rewind starts, require arms lowered, source inside `40..55 deg`, takeup inside `40..70 deg`, and target speed legal for gear ratio.
- `Precharge`: puller command `0 m/min`; source and takeup use very low-authority arm centering; learning disabled; stay until both arms are stable for about `2 s`.
- `CrawlStart`: puller ramps gently to `1 m/min`; source and takeup use feed-forward plus weak trim; learning disabled; stay until stable for about `3 s`.
- `Rewind`: puller ramps toward UI target; source and takeup use feed-forward plus bounded trim; learning enabled only when stable.
- `ControlledHold`: ramp puller/source/takeup down smoothly; disable learning; do not instantly zero commands unless safety requires it.
- `FaultHold`: fault stop; controlled deceleration if possible; learning disabled; operator reset required; no automatic restart.

Suggested zones:

```text
Source:
  hard:       20..60 deg
  emergency: 23 / 59 deg
  warning:   28 / 56 deg
  comfort:   38..53 deg
  target:    50 deg

Takeup:
  hard:       20..90 deg
  emergency: 24 / 86 deg
  warning:   30 / 78 deg
  comfort:   42..68 deg
  target:    55 deg
```

Warning behavior:

- Freeze learning.
- Reduce puller acceleration, for example to `0.2 m/min/s`.
- Do not abruptly change spool speeds.

Emergency behavior:

- Freeze learning.
- Prevent further puller acceleration.
- If angle continues moving in the wrong direction, begin controlled slowdown.

Hard fault behavior:

- Source `<=20` or `>=60`: enter `FaultHold`.
- Takeup `<=20` or `>=90`: enter `FaultHold`.

Puller command:

```text
UI target speed -> backend gear-ratio clamp -> puller_command through ramp limiter

normal accel: 0.5 m/min/s
normal decel: 1.0 m/min/s
fault decel:  2.0..3.0 m/min/s
```

Backend should clamp by gear ratio, not only frontend:

```text
1:1  -> 50 m/min max
1:5  -> 10 m/min max
1:10 -> 5 m/min max
```

Angle filtering:

```text
filtered += alpha * (raw - filtered)
alpha = dt / (tau + dt)
angle_rate = (filtered - previous_filtered) / dt

source tau: 0.15..0.30 s
takeup tau: 0.10..0.25 s
```

Follower command shape:

```text
spool_command_rpm = feed_forward_rpm + trim_rpm
spool_command_rpm = slew_limit(previous_command, spool_command_rpm, max_rpm_per_s, dt)
```

Source follower:

```text
source_ff_rpm = puller_command_m_per_min * source_ratio_rpm_per_mpm
source_error_deg = source_filtered_deg - 50.0
source_trim_rpm = source_kp_rpm_per_deg * deadband(source_error_deg, 1.5)
source_trim_rpm = clamp(source_trim_rpm, -10, 10)
source_rpm_slew = 15 rpm/s
source_ratio_initial = 1.5 rpm per m/min
source_ratio_min = 0.5
source_ratio_max = 6.0
```

Positive source error means source angle is high, so positive trim should increase source payout rpm. Confirm motor sign mechanically before trusting this.

Takeup follower:

```text
takeup_ff_rpm = puller_command_m_per_min * takeup_ratio_rpm_per_mpm
takeup_error_deg = 55.0 - takeup_filtered_deg
takeup_trim_rpm = takeup_kp_rpm_per_deg * deadband(takeup_error_deg, 1.5)
takeup_trim_rpm = clamp(takeup_trim_rpm, -10, 10)
takeup_rpm_slew = 20 rpm/s
```

If takeup angle is low, takeup should wind faster. Confirm sign mechanically.

Deadband:

```text
if abs(x) <= db:
    0
else:
    sign(x) * (abs(x) - db)
```

Slew limit:

```text
delta = target - current
max_delta = max_rate_per_sec * dt
next = current + clamp(delta, -max_delta, max_delta)
```

Learning should be slow and heavily gated:

```text
learning allowed only when:
  phase == Rewind
  puller speed > 5 m/min
  abs(puller acceleration) < 0.2 m/min/s
  source and takeup inside comfort zones
  source and takeup angle rates small
  trims are not saturated
  no warning active
  no emergency active
  no fault active
  stable for at least 3 s

learning time constant: about 60 s
```

If using ratio learning:

```text
observed_ratio = command_rpm / puller_command_m_per_min
ratio += (dt / tau) * (observed_ratio - ratio)
ratio = clamp(ratio, ratio_min, ratio_max)
```

Do not learn during Validate, Precharge, CrawlStart, acceleration, deceleration, warning, emergency, fault, or arm excursions.

Suggested implementation structure:

```rust
enum ArmZone {
    Comfort,
    WarningLow,
    WarningHigh,
    EmergencyLow,
    EmergencyHigh,
    HardLow,
    HardHigh,
}

struct ArmConfig {
    hard_low: f64,
    emergency_low: f64,
    warning_low: f64,
    comfort_low: f64,
    target: f64,
    comfort_high: f64,
    warning_high: f64,
    emergency_high: f64,
    hard_high: f64,
    filter_tau_s: f64,
    deadband_deg: f64,
}

struct ArmState {
    raw_deg: f64,
    filtered_deg: f64,
    prev_filtered_deg: f64,
    rate_deg_s: f64,
    zone: ArmZone,
}

struct FollowerConfig {
    kp_rpm_per_deg: f64,
    trim_max_rpm: f64,
    rpm_slew_rpm_per_s: f64,
    ratio_initial: f64,
    ratio_min: f64,
    ratio_max: f64,
    learning_tau_s: f64,
}

struct FollowerState {
    ratio_rpm_per_mpm: f64,
    ff_rpm: f64,
    trim_rpm: f64,
    target_rpm: f64,
    command_rpm: f64,
    learning_enabled: bool,
    learning_block_reason: String,
}
```

Recommended act-cycle shape:

```rust
fn act(dt) {
    read_inputs();
    filter_source_angle();
    filter_takeup_angle();
    compute_angle_rates();
    update_rewind_phase();
    compute_puller_command();
    compute_source_command();
    compute_takeup_command();
    compute_traverse_command();
    apply_slew_limits();
    write_outputs();
    log_control_sample();
}
```

Required commissioning checks:

1. Confirm source arm sensor sign.
2. Confirm takeup arm sensor sign.
3. Confirm source motor positive direction pays out.
4. Confirm takeup motor positive direction winds in.
5. Confirm puller positive direction pulls forward.
6. Run Pull mode only.
7. Run Precharge with plastic material.
8. Run CrawlStart at `1 m/min`.
9. Run steady speeds `1`, `3`, `5`, `8 m/min` without learning.
10. Enable slow learning.
11. Test ramps `1->3`, `3->8`, `8->15`, `15->25`.
12. Only then test glass fiber with reduced acceleration and trim limits.

Additional logs needed:

- `dt`
- `puller_accel_mpm_s`
- source/takeup raw angle
- source/takeup filtered angle
- source/takeup angle rate
- source/takeup feed-forward rpm
- source/takeup trim rpm
- source/takeup target rpm
- source/takeup command rpm
- source/takeup actual rpm
- source/takeup ratio or radius estimate
- source/takeup zone
- learning enabled flags
- learning block reason
- warning/emergency/fault flags
- fault reason
