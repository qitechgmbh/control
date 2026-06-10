# Rewinder Missing Features Roadmap

This document tracks the remaining non-laser work needed to bring the QiTech Rewinder closer to Winder2 feature parity while keeping the machine-specific rewind behavior clean.

The order matters: each step should be tested before moving to the next one.

## 1. Gear-Ratio-Aware Puller Speed Limit

Current issue: Rewinder frontend uses a fixed puller target speed max. Winder2 limits target speed based on selected gear ratio.

Implement:
- Use the same control-page rule as Winder2:
  - `1:1` -> max `50 m/min`
  - `1:5` -> max `10 m/min`
  - `1:10` -> max `5 m/min`
- Reuse or mirror Winder2's gear-ratio multiplier logic in the Rewinder UI.
- Keep backend gear-ratio behavior unchanged.
- Optional backend clamp can be added later, but frontend parity is enough for the first pass.

Success criteria:
- Select `1:1`: puller target speed input allows up to `50 m/min`.
- Select `1:5`: input allows up to `10 m/min`.
- Select `1:10`: input allows up to `5 m/min`.
- Changing gear ratio resets target speed to `0`, as currently done.
- Pull and Rewind still move the puller at the expected displayed line speed.

Do not move on until:
- Puller line speed display is sane for all three gear ratios.
- No source/takeup spool behavior changes unexpectedly when gear ratio changes.

## 2. Rewinder Settings Page

Current issue: Rewinder has many tuning parameters on the control page. Winder2 separates operation and settings more clearly.

Implement:
- Add a dedicated Rewinder settings page and topbar item.
- Add the required frontend route wiring.
- Move tuning controls out of the primary control page where appropriate.
- Keep operational controls on the control page:
  - mode
  - puller target speed
  - live values
  - zero buttons
  - traverse actions
- Put parameter tuning in settings:
  - takeup spool adaptive parameters
  - takeup MinMax parameters
  - source spool adaptive parameters once exposed
  - traverse limits, step size, padding if desired
- Keep backend mutation names and state fields aligned with existing Rewinder API names.

Success criteria:
- Control page remains usable for daily operation without scrolling through deep tuning.
- Settings page can edit the same values as before.
- Reloading the frontend shows the current backend values correctly.
- No API payload names are changed unnecessarily.

Do not move on until:
- Existing Rewinder control workflow still works: Standby -> Hold -> Pull -> Rewind.
- No setting silently stops updating the backend.

## 3. Source Spool Tuning Controls

Current issue: source spool in Rewind uses adaptive control, but the UI exposes only source tension target. Takeup exposes much more.

Implement:
- Expose source adaptive parameters in backend state, API mutation handling, frontend namespace parsing, hook helpers, and the Rewinder settings page:
  - tension target
  - radius learning rate
  - max speed multiplier
  - acceleration factor
  - deacceleration urgency multiplier
- Keep source defaults conservative.
- Do not add source MinMax unless testing shows adaptive source control is unsuitable.

Success criteria:
- Each source adaptive parameter can be changed from the UI.
- Backend state event reports the updated source values.
- Rewind source spool response changes predictably when source tension target or acceleration factor is changed.
- Takeup adaptive controls remain unaffected.

Do not move on until:
- Plastic-wire tests show source payout can be made smoother or more responsive through settings without code changes.

## 4. Pull Source Assist Parameterization

Current issue: Pull mode source assist uses a simple estimated source diameter and underfeed factor hardcoded in backend.

Implement:
- Keep Pull source assist simple.
- Move useful Pull-assist constants into backend state/API and expose them in the Rewinder settings page:
  - estimated source spool diameter
  - underfeed factor
  - max source assist rpm
- Avoid turning this into a second full controller.

Success criteria:
- Puller remains the master in Pull mode.
- Source spool assists without dumping slack.
- Changing puller target speed changes source assist speed immediately.
- With plastic wire, the operator no longer needs to manually rotate the source spool during Pull.

Do not move on until:
- Pull mode can thread material reliably at low speed.
- Source spool never overfeeds enough to create visible slack in normal setup.

## 5. Length Tracking

Current issue: Winder2 tracks pulled spool progress in meters; Rewinder does not.

Implement:
- Track rewound length from puller speed over time.
- Add live value/state field for progress in meters.
- Add reset progress action.
- Add the frontend display on the control page.
- Add a Rewinder graphs page and topbar item once there is useful progress/speed/tension history to plot.
- Use the same basic approach as Winder2 unless there is a clear reason not to.

Success criteria:
- Progress increases in Pull and Rewind when puller moves.
- Progress does not increase in Standby or Hold.
- Reset progress sets value to `0`.
- Measured progress roughly matches manually measured wire length over a short plastic-wire test.

Do not move on until:
- Progress is stable enough to support automatic stop/hold behavior.

## 6. Automatic Stop/Hold By Length

Current issue: Winder2 can perform actions after a target length. Rewinder lacks this.

Implement:
- Add required meters setting in backend state/API and Rewinder settings UI.
- Add automatic action mode in backend state/API and Rewinder settings UI:
  - no action
  - hold after target length
  - standby after target length, if useful
- Prefer `Hold` as the first implemented action because it keeps axes controlled.
- Show target/progress/action status on the control page so the operator can see why the machine stopped.

Success criteria:
- Set target length, start Rewind, machine transitions to configured action when progress reaches target.
- Action does not trigger when automatic mode is disabled.
- Reset progress allows another run.
- No unexpected stop occurs during manual Pull unless explicitly intended.

Do not move on until:
- Plastic-wire test confirms the machine stops/holds within an acceptable length tolerance.

## 7. Presets

Current issue: Winder2 has presets; Rewinder does not. Presets become important once glass-fiber parameters are known.

Implement:
- Add Rewinder presets page, topbar item, route wiring, and preset schema.
- Include:
  - puller target speed
  - gear ratio
  - traverse settings
  - takeup adaptive/minmax settings
  - source adaptive settings
  - source pull assist settings
  - length target/action settings, if implemented
- Exclude live values and zero states.

Success criteria:
- Save a plastic-wire preset.
- Apply it after restart.
- Verify the UI and backend state match the preset.
- Save a second preset with different speeds/tension settings and switch between them safely.

Do not move on until:
- Applying presets does not start motion by itself.
- Applying presets does not overwrite machine identity or live calibration state.

## 8. Glass-Fiber Parameter Validation

Current issue: the algorithm is structurally usable, but glass fiber needs validated numbers.

Implement:
- Work with mechanical/electrical colleagues to define:
  - source legal angle range
  - takeup legal angle range
  - recommended puller speed range
  - max allowed acceleration/deceleration behavior
  - acceptable traverse lay pattern
- Encode agreed defaults.
- Save them as a preset once presets exist.
- Add/update Rewinder operator documentation after the tested workflow is stable. This should cover setup, zeroing, Pull threading, Rewind start requirements, fault recovery, and glass-fiber limits. Keep it aligned with the actual backend behavior instead of copying Winder2 manual text blindly.

Success criteria:
- Rewind plastic wire successfully.
- Rewind glass fiber at very low speed without visible damage.
- Increase speed only after low-speed run is stable.
- Confirm no sudden source/takeup spool kicks during tension corrections.
- Confirm final winding quality is acceptable on takeup spool.

Do not consider the machine production-ready until:
- Several full rewinds complete without operator intervention.
- Source and takeup tension arms stay mostly inside their legal ranges.
- Operators can recover cleanly from an out-of-range tension arm by stopping, rethreading if needed, zeroing/checking angles, and restarting.

## Explicitly Omitted

The following Winder2 features are intentionally omitted for now:
- laser diameter measurement
- laser-based adaptive puller regulation
- laser pointer controls
- diameter reference machine selection

The Rewinder UI should still gain the non-laser Winder2 surfaces that support real operation:
- Config/Settings, tied to tuning and length/action settings above.
- Graphs, tied to length/progress and live tension/speed history above.
- Presets, tied to tested material parameter sets above.
- Manual/operator workflow, tied to validated glass-fiber operation above.

Reason: the rewinder transfers existing material. The core control problem is tension and speed coordination, not extrusion diameter regulation.
