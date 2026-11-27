//! Synthetic measurement curves. Each [`Signal`] is a pure function of time plus uniform
//! noise, evaluated in Rust for the live data and compiled to a ClickHouse expression
//! for the 30-day backfill, so both produce the same curve and the seam between history
//! and live data does not jump.

use std::f64::consts::TAU;

/// Time origin of every signal (2026-01-01T00:00:00Z). Keeps `t` small enough that
/// `sin(t)` and the ramps stay precise in `f64`.
pub const EPOCH_MS: i64 = 1_767_225_600_000;

#[derive(Debug, Clone, Copy)]
pub enum Base {
    Const(f64),
    /// Follows the current value of a config property, plus `offset`.
    Config {
        path: &'static str,
        offset: f64,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Signal {
    /// `base + slow + fast + noise`; `slow` and `fast` are `(amplitude, period in s)`.
    Wave {
        base: Base,
        slow: (f64, f64),
        fast: (f64, f64),
        noise: f64,
    },
    /// Back and forth between `lo` and `hi` once per `period_s`.
    Triangle {
        lo: f64,
        hi: f64,
        period_s: f64,
        noise: f64,
    },
    /// Counts up by `rate` per second, restarting at 0 on reaching `wrap` (never if 0).
    Ramp { rate: f64, wrap: f64 },
}

/// Seconds since [`EPOCH_MS`].
pub fn seconds(unix_ms: i64) -> f64 {
    (unix_ms - EPOCH_MS) as f64 / 1000.0
}

impl Signal {
    /// Value at `t` (seconds since [`EPOCH_MS`]). `base` is the resolved [`Base`],
    /// `phase` (rad) decorrelates signals with the same periods, `unit_noise` is uniform
    /// in `[-1, 1)`.
    pub fn eval(&self, t: f64, base: f64, phase: f64, unit_noise: f64) -> f64 {
        match *self {
            Signal::Wave {
                slow, fast, noise, ..
            } => {
                base + slow.0 * (TAU * t / slow.1 + phase).sin()
                    + fast.0 * (TAU * t / fast.1 + 2.0 * phase).sin()
                    + noise * unit_noise
            }
            Signal::Triangle {
                lo,
                hi,
                period_s,
                noise,
            } => {
                let x = t / period_s + phase / TAU;
                let frac = x - x.floor();
                lo + (hi - lo) * (1.0 - (2.0 * frac - 1.0).abs()) + noise * unit_noise
            }
            Signal::Ramp { rate, wrap } => {
                let v = rate * t;
                if wrap > 0.0 {
                    v - wrap * (v / wrap).floor()
                } else {
                    v
                }
            }
        }
    }

    /// ClickHouse expression of [`Signal::eval`] over a Float64 column `t`.
    pub fn sql(&self, base: f64, phase: f64) -> String {
        // `{:?}` always prints a decimal point or exponent, so ClickHouse reads Float64
        let noise = "(randCanonical() * 2 - 1)";
        match *self {
            Signal::Wave {
                slow,
                fast,
                noise: n,
                ..
            } => format!(
                "{base:?} + {a1:?} * sin({tau:?} * t / {p1:?} + {phase:?}) \
                 + {a2:?} * sin({tau:?} * t / {p2:?} + {phase2:?}) + {n:?} * {noise}",
                tau = TAU,
                a1 = slow.0,
                p1 = slow.1,
                a2 = fast.0,
                p2 = fast.1,
                phase2 = 2.0 * phase,
            ),
            Signal::Triangle {
                lo,
                hi,
                period_s,
                noise: n,
            } => format!(
                "{lo:?} + {span:?} * (1 - abs(2 * ((t / {period_s:?} + {shift:?}) \
                 - floor(t / {period_s:?} + {shift:?})) - 1)) + {n:?} * {noise}",
                span = hi - lo,
                shift = phase / TAU,
            ),
            Signal::Ramp { rate, wrap } => {
                if wrap > 0.0 {
                    format!("({rate:?} * t) - {wrap:?} * floor({rate:?} * t / {wrap:?})")
                } else {
                    format!("{rate:?} * t")
                }
            }
        }
    }
}

/// Phase of the `index`-th measurement of a machine, spread so neighbours differ.
pub fn phase(index: usize) -> f64 {
    (index as f64 * 2.399_963) % TAU // golden angle
}

/// Small xorshift generator for the live noise; no need for a crypto RNG here.
pub struct Noise(u64);

impl Noise {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    /// Uniform in `[-1, 1)`.
    pub fn next(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_wraps() {
        let ramp = Signal::Ramp {
            rate: 1.0,
            wrap: 10.0,
        };
        assert!((ramp.eval(25.0, 0.0, 0.0, 0.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn triangle_stays_in_range() {
        let tri = Signal::Triangle {
            lo: 22.0,
            hi: 92.0,
            period_s: 126.0,
            noise: 0.0,
        };
        for i in 0..1000 {
            let v = tri.eval(i as f64 * 0.37, 0.0, 1.3, 0.0);
            assert!((22.0..=92.0).contains(&v), "{v}");
        }
    }

    #[test]
    fn noise_is_in_unit_range() {
        let mut noise = Noise::new(42);
        for _ in 0..10_000 {
            let v = noise.next();
            assert!((-1.0..1.0).contains(&v));
        }
    }

    #[test]
    fn sql_uses_float_literals() {
        let sql = Signal::Ramp {
            rate: 1.0,
            wrap: 0.0,
        }
        .sql(0.0, 0.0);
        assert_eq!(sql, "1.0 * t");
    }
}
