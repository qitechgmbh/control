# `2.2.0`

_15.07.2025_

## Extruder V2

- [#455](https://github.com/qitechgmbh/control/pull/458), Greatly improved stability and precision of pressure-based regulation for the motor
- [#467](https://github.com/qitechgmbh/control/pull/468), [#448](https://github.com/qitechgmbh/control/pull/449), [#485](https://github.com/qitechgmbh/control/pull/486) Improved the Responsiveness for motor control and monitoring
- [#138](https://github.com/qitechgmbh/control/pull/495), Added a Warning in case the Pressure sensor is disconnected or incorrectly wired
- [#169](https://github.com/qitechgmbh/control/pull/492), Added a Confirmation Dialog when extruder has not reached 90% of target temperature

## Winder V2
- [#471](https://github.com/qitechgmbh/control/pull/480), Added a live value to track the pulled distance and the option of switching to a different Mode after a given distance.

## General

- [#121](https://github.com/qitechgmbh/control/pull/121), Use Linux realtime threads for EtherCAT and control loop (PREEMPT_RT)
- [#138](https://github.com/qitechgmbh/control/pull/495), Updated Quick Start Guide

# `2.1.0`

_30.06.2025_

## Extruder V2

- Intial Stable Release of **Extruder V2**

## Winder V2

- [#420](https://github.com/qitechgmbh/control/pull/429), [#401](https://github.com/qitechgmbh/control/pull/401) Add _Adaptive_ spool speed algorithm, the old algorithm is now called _Minmax_ ans tillavailable.
- [#427](https://github.com/qitechgmbh/control/pull/427) Add _Estimated Spool Diameter_ graph
- [#419](https://github.com/qitechgmbh/control/pull/419) Fix Issue where traverse won't work at low spool speed

## Laser V1

- Initial Stable Release of **Laser V1**

## HMI Panel

- [#339](https://github.com/qitechgmbh/control/pull/338) Don't go into sleep Mmde
- [#410](https://github.com/qitechgmbh/control/pull/410) Ship with Wireshark
- [#424](https://github.com/qitechgmbh/control/pull/424) Updates can be canceled

## General

- [#269](https://github.com/qitechgmbh/control/pull/269), [#370](https://github.com/qitechgmbh/control/pull/370) Improved performace of live values
- [#358](https://github.com/qitechgmbh/control/pull/358), [#343](https://github.com/qitechgmbh/control/pull/343), [#324](https://github.com/qitechgmbh/control/pull/324), [#390](https://github.com/qitechgmbh/control/pull/390), [#299](https://github.com/qitechgmbh/control/pull/299) Improved functionality for graphs
- [#405](https://github.com/qitechgmbh/control/pull/405) Remove weird color behaviour on tocuhscreens
- [#369](https://github.com/qitechgmbh/control/pull/369) More failsafe EtherCAT loop

# `2.0.1`

_10.06.2025_

# `2.0.0`

_10.06.2025_

## General

- Initial stable release of QiTech Control.

## Winder V2

- Initial stable release of **Winder V2**

## Features

- Stabilized **Winder V2**
