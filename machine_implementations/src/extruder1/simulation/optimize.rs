//! A Nelder-Mead simplex search, shared by everything in here that tunes
//! something: the model calibration in [`super::fit`] and the controller-gain
//! searches in the `bench_heating` example.
//!
//! Plain Nelder-Mead parks in a corner distressingly often on the objectives
//! here — nine correlated coefficients, or four zones' gains at once — so
//! [`nelder_mead`] restarts the simplex around the best point with a smaller step
//! when it collapses, and tracks the best vertex ever seen rather than trusting
//! the final simplex.

/// Search settings. [`Options::default`] is the configuration the model
/// calibration uses.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Evaluation budget. The search stops shortly after exceeding it.
    pub max_evaluations: usize,
    /// Initial simplex edge, in the units of the vector being searched. Callers
    /// working in log space want something like 0.6 (a factor of ~1.8).
    pub initial_step: f64,
    /// Give up once a restart would use a step below this.
    pub min_step: f64,
    /// Restart when the spread across the simplex falls below this fraction of
    /// the best value.
    pub collapse_tolerance: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_evaluations: 400,
            initial_step: 0.6,
            min_step: 0.05,
            collapse_tolerance: 1e-3,
        }
    }
}

/// Result of a search.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// Best point found. Never worse than `start`.
    pub point: Vec<f64>,
    pub value: f64,
    pub evaluations: usize,
}

/// Minimise `cost` starting from `start`.
///
/// `cost` may be called with points anywhere in the space; a caller with bounds
/// should clamp inside its own closure, which is what makes the returned point
/// meaningful without this knowing anything about them.
pub fn nelder_mead<F>(start: &[f64], mut cost: F, opts: Options) -> Outcome
where
    F: FnMut(&[f64]) -> f64,
{
    let n = start.len();
    assert!(n > 0, "nothing to optimise");

    let mut evaluations = 0usize;
    let mut evaluate = |x: &[f64], evaluations: &mut usize| {
        *evaluations += 1;
        cost(x)
    };

    // A simplex around `centre`, one step per axis.
    let build = |centre: &[f64], step: f64| {
        let mut s = Vec::with_capacity(n + 1);
        s.push(centre.to_vec());
        for i in 0..n {
            let mut v = centre.to_vec();
            v[i] += step;
            s.push(v);
        }
        s
    };

    let mut step = opts.initial_step;
    let mut simplex = build(start, step);
    let mut values: Vec<f64> = simplex
        .iter()
        .map(|s| evaluate(s, &mut evaluations))
        .collect();
    let mut best_point = simplex[0].clone();
    let mut best_value = f64::INFINITY;

    let (alpha, gamma, rho, sigma) = (1.0_f64, 2.0_f64, 0.5_f64, 0.5_f64);

    while evaluations < opts.max_evaluations {
        let mut order: Vec<usize> = (0..simplex.len()).collect();
        order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
        simplex = order.iter().map(|&i| simplex[i].clone()).collect();
        values = order.iter().map(|&i| values[i]).collect();

        if values[0] < best_value {
            best_value = values[0];
            best_point.clone_from(&simplex[0]);
        }

        let spread = (values[values.len() - 1] - values[0]).abs();
        if spread < opts.collapse_tolerance * values[0].abs().max(1e-6) {
            if step < opts.min_step {
                break;
            }
            step *= 0.4;
            simplex = build(&best_point.clone(), step);
            values = simplex
                .iter()
                .map(|s| evaluate(s, &mut evaluations))
                .collect();
            continue;
        }

        // Centroid of everything but the worst point.
        let worst = simplex.len() - 1;
        let mut centroid = vec![0.0; n];
        for s in &simplex[..worst] {
            for (c, v) in centroid.iter_mut().zip(s) {
                *c += v / worst as f64;
            }
        }

        let reflect: Vec<f64> = centroid
            .iter()
            .zip(&simplex[worst])
            .map(|(c, w)| alpha.mul_add(c - w, *c))
            .collect();
        let f_reflect = evaluate(&reflect, &mut evaluations);

        if f_reflect < values[0] {
            let expand: Vec<f64> = centroid
                .iter()
                .zip(&reflect)
                .map(|(c, r)| gamma.mul_add(r - c, *c))
                .collect();
            let f_expand = evaluate(&expand, &mut evaluations);
            if f_expand < f_reflect {
                simplex[worst] = expand;
                values[worst] = f_expand;
            } else {
                simplex[worst] = reflect;
                values[worst] = f_reflect;
            }
        } else if f_reflect < values[worst - 1] {
            simplex[worst] = reflect;
            values[worst] = f_reflect;
        } else {
            let contract: Vec<f64> = centroid
                .iter()
                .zip(&simplex[worst])
                .map(|(c, w)| rho.mul_add(w - c, *c))
                .collect();
            let f_contract = evaluate(&contract, &mut evaluations);
            if f_contract < values[worst] {
                simplex[worst] = contract;
                values[worst] = f_contract;
            } else {
                // Shrink towards the best vertex.
                let best = simplex[0].clone();
                for i in 1..simplex.len() {
                    simplex[i] = best
                        .iter()
                        .zip(&simplex[i])
                        .map(|(b, s)| sigma.mul_add(s - b, *b))
                        .collect();
                    values[i] = evaluate(&simplex[i], &mut evaluations);
                }
            }
        }
    }

    // The final simplex can be worse than something seen along the way.
    if let Some((point, value)) = simplex
        .iter()
        .zip(&values)
        .min_by(|a, b| a.1.total_cmp(b.1))
        && *value < best_value
    {
        best_value = *value;
        best_point.clone_from(point);
    }

    Outcome {
        point: best_point,
        value: best_value,
        evaluations,
    }
}

/// xorshift64*, so searches are reproducible from a seed without pulling in a
/// dependency.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub const fn new(seed: u64) -> Self {
        // A zero state is a fixed point of xorshift.
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    pub const fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform in `[0, 1)`.
    pub const fn next_f64(&mut self) -> f64 {
        // Top 53 bits, the mantissa width of an f64.
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[lo, hi)`.
    pub const fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_minimum_of_a_quadratic() {
        let out = nelder_mead(
            &[0.0, 0.0, 0.0],
            |x| (x[0] - 1.0).powi(2) + (x[1] + 2.0).powi(2) + (x[2] - 0.5).powi(2),
            Options::default(),
        );
        assert!(out.value < 1e-6, "value {}", out.value);
        for (got, want) in out.point.iter().zip([1.0, -2.0, 0.5]) {
            assert!((got - want).abs() < 1e-2, "{got} vs {want}");
        }
    }

    /// Rosenbrock has a curved valley that a single un-restarted descent stalls
    /// in; the restart logic is what gets through it.
    #[test]
    fn works_on_a_curved_valley() {
        let out = nelder_mead(
            &[-1.2, 1.0],
            |x| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2),
            Options {
                max_evaluations: 2000,
                initial_step: 0.5,
                min_step: 1e-6,
                ..Options::default()
            },
        );
        assert!(out.value < 1e-4, "value {}", out.value);
    }

    #[test]
    fn respects_the_evaluation_budget() {
        let mut calls = 0usize;
        let out = nelder_mead(
            &[0.0; 4],
            |x| {
                calls += 1;
                x.iter().map(|v| v * v).sum()
            },
            Options {
                max_evaluations: 50,
                min_step: 1e-9,
                collapse_tolerance: 0.0,
                ..Options::default()
            },
        );
        // A single iteration can spend a few evaluations past the check.
        assert!(calls <= 60, "{calls} evaluations for a budget of 50");
        assert_eq!(calls, out.evaluations);
    }

    #[test]
    fn never_returns_worse_than_the_start() {
        let out = nelder_mead(&[3.0], |x| x[0] * x[0], Options::default());
        assert!(out.value <= 9.0);
    }

    #[test]
    fn rng_is_reproducible_and_in_range() {
        let a: Vec<f64> = (0..100).map(|_| Rng::new(42).next_f64()).collect();
        let mut b = Rng::new(42);
        assert!(a.iter().all(|v| (0.0..1.0).contains(v)));
        assert_eq!(a[0], b.next_f64());

        let mut rng = Rng::new(7);
        assert!((0..1000).all(|_| (-2.0..5.0).contains(&rng.range(-2.0, 5.0))));
    }

    #[test]
    fn rng_survives_a_zero_seed() {
        let mut rng = Rng::new(0);
        let first = rng.next_u64();
        assert_ne!(first, 0);
        assert_ne!(first, rng.next_u64());
    }
}
