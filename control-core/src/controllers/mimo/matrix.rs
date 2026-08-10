//! Small dense linear algebra for the MIMO controller.
//!
//! Everything here is fixed at `N x N` with `N` known at compile time (4 for the extruder barrel),
//! so it is all stack-allocated and dependency-free. These are not general-purpose routines — they
//! are sized for matrices small enough that an `O(N^3)` algorithm with clear failure modes beats
//! anything cleverer.

/// Square matrix, row-major: `m[row][col]`.
pub type Mat<const N: usize> = [[f64; N]; N];
/// Column vector.
pub type Vec_<const N: usize> = [f64; N];

/// A matrix whose columns are numerically indistinguishable is not invertible in any useful sense.
/// Expressed relative to the largest pivot seen so far, so it scales with the matrix.
const PIVOT_EPS: f64 = 1e-12;

pub const fn zeros<const N: usize>() -> Mat<N> {
    [[0.0; N]; N]
}

pub fn identity<const N: usize>() -> Mat<N> {
    let mut m = zeros::<N>();
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    m
}

/// Diagonal matrix from a vector.
pub fn diag<const N: usize>(d: &Vec_<N>) -> Mat<N> {
    let mut m = zeros::<N>();
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = d[i];
    }
    m
}

pub fn matmul<const N: usize>(a: &Mat<N>, b: &Mat<N>) -> Mat<N> {
    let mut out = zeros::<N>();
    for i in 0..N {
        for k in 0..N {
            let aik = a[i][k];
            if aik == 0.0 {
                continue;
            }
            for j in 0..N {
                out[i][j] += aik * b[k][j];
            }
        }
    }
    out
}

pub fn matvec<const N: usize>(a: &Mat<N>, x: &Vec_<N>) -> Vec_<N> {
    let mut out = [0.0; N];
    for i in 0..N {
        let mut acc = 0.0;
        for j in 0..N {
            acc += a[i][j] * x[j];
        }
        out[i] = acc;
    }
    out
}

pub fn transpose<const N: usize>(a: &Mat<N>) -> Mat<N> {
    let mut out = zeros::<N>();
    for i in 0..N {
        for j in 0..N {
            out[j][i] = a[i][j];
        }
    }
    out
}

pub fn scale<const N: usize>(a: &Mat<N>, s: f64) -> Mat<N> {
    let mut out = *a;
    for row in &mut out {
        for v in row.iter_mut() {
            *v *= s;
        }
    }
    out
}

/// Largest absolute entry. Used as the scale reference for the pivot tolerance.
pub fn max_abs<const N: usize>(a: &Mat<N>) -> f64 {
    a.iter()
        .flat_map(|r| r.iter())
        .fold(0.0_f64, |m, v| m.max(v.abs()))
}

pub fn all_finite<const N: usize>(a: &Mat<N>) -> bool {
    a.iter().flat_map(|r| r.iter()).all(|v| v.is_finite())
}

/// Gauss-Jordan inverse with partial pivoting.
///
/// Returns `None` when the matrix is singular to working precision, rather than producing a
/// garbage inverse — the caller (static decoupling) has to refuse in that case, and a silent
/// `inf` would otherwise propagate straight into controller gains.
pub fn inverse<const N: usize>(a: &Mat<N>) -> Option<Mat<N>> {
    let scale_ref = max_abs(a);
    if scale_ref == 0.0 || !all_finite(a) {
        return None;
    }
    let tol = PIVOT_EPS * scale_ref;

    let mut work = *a;
    let mut inv = identity::<N>();

    for col in 0..N {
        // Partial pivot: the largest remaining entry in this column.
        let mut pivot_row = col;
        let mut pivot_mag = work[col][col].abs();
        for (r, row) in work.iter().enumerate().skip(col + 1) {
            let mag = row[col].abs();
            if mag > pivot_mag {
                pivot_mag = mag;
                pivot_row = r;
            }
        }
        if pivot_mag <= tol {
            return None;
        }
        work.swap(col, pivot_row);
        inv.swap(col, pivot_row);

        let pivot = work[col][col];
        for j in 0..N {
            work[col][j] /= pivot;
            inv[col][j] /= pivot;
        }

        for r in 0..N {
            if r == col {
                continue;
            }
            let factor = work[r][col];
            if factor == 0.0 {
                continue;
            }
            for j in 0..N {
                work[r][j] -= factor * work[col][j];
                inv[r][j] -= factor * inv[col][j];
            }
        }
    }

    all_finite(&inv).then_some(inv)
}

/// Determinant via LU with partial pivoting.
pub fn determinant<const N: usize>(a: &Mat<N>) -> f64 {
    let mut work = *a;
    let mut det = 1.0;

    for col in 0..N {
        let mut pivot_row = col;
        let mut pivot_mag = work[col][col].abs();
        for (r, row) in work.iter().enumerate().skip(col + 1) {
            let mag = row[col].abs();
            if mag > pivot_mag {
                pivot_mag = mag;
                pivot_row = r;
            }
        }
        if pivot_mag == 0.0 {
            return 0.0;
        }
        if pivot_row != col {
            work.swap(col, pivot_row);
            det = -det;
        }
        det *= work[col][col];
        let pivot = work[col][col];
        for r in col + 1..N {
            let factor = work[r][col] / pivot;
            if factor == 0.0 {
                continue;
            }
            for j in col..N {
                work[r][j] -= factor * work[col][j];
            }
        }
    }
    det
}

/// Singular values, descending.
///
/// One-sided Jacobi: repeatedly rotate pairs of columns until they are mutually orthogonal, at
/// which point the column norms are the singular values. Chosen over a bidiagonal reduction
/// because it is short, needs no external solver, and is accurate on the small well-scaled
/// matrices this is used for.
pub fn singular_values<const N: usize>(a: &Mat<N>) -> Vec_<N> {
    const MAX_SWEEPS: usize = 60;
    const ORTHOGONALITY_EPS: f64 = 1e-15;

    // Work on columns, so index as work[col][row].
    let mut work = transpose(a);

    for _ in 0..MAX_SWEEPS {
        let mut off_diagonal = 0.0_f64;

        for p in 0..N {
            for q in p + 1..N {
                let mut alpha = 0.0; // <col_p, col_p>
                let mut beta = 0.0; // <col_q, col_q>
                let mut gamma = 0.0; // <col_p, col_q>
                for k in 0..N {
                    alpha += work[p][k] * work[p][k];
                    beta += work[q][k] * work[q][k];
                    gamma += work[p][k] * work[q][k];
                }

                let denom = (alpha * beta).sqrt();
                if denom <= 0.0 {
                    continue;
                }
                off_diagonal = off_diagonal.max(gamma.abs() / denom);
                if gamma.abs() <= ORTHOGONALITY_EPS * denom {
                    continue;
                }

                // Jacobi rotation that annihilates gamma.
                let zeta = (beta - alpha) / (2.0 * gamma);
                let t = zeta.signum() / (zeta.abs() + (1.0 + zeta * zeta).sqrt());
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = c * t;

                for k in 0..N {
                    let vp = work[p][k];
                    let vq = work[q][k];
                    work[p][k] = c * vp - s * vq;
                    work[q][k] = s * vp + c * vq;
                }
            }
        }

        if off_diagonal <= ORTHOGONALITY_EPS {
            break;
        }
    }

    let mut sv = [0.0; N];
    for (i, slot) in sv.iter_mut().enumerate() {
        *slot = work[i].iter().map(|v| v * v).sum::<f64>().sqrt();
    }
    sv.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    sv
}

/// `sigma_max / sigma_min`. Infinite for a singular matrix.
pub fn condition_number<const N: usize>(a: &Mat<N>) -> f64 {
    let sv = singular_values(a);
    let smax = sv[0];
    let smin = sv[N - 1];
    if smin <= 0.0 {
        f64::INFINITY
    } else {
        smax / smin
    }
}

/// Moore-Penrose pseudo-inverse via Tikhonov damping.
///
/// `pinv(A) = (A^T A + lambda I)^-1 A^T`, with `lambda` scaled to the matrix so that a genuinely
/// singular `A` yields a bounded, small result instead of blowing up. Used by the MIMO
/// anti-windup, where being approximately right every tick matters far more than being exactly
/// right occasionally — an unbounded correction there would be worse than a damped one.
pub fn pseudo_inverse<const N: usize>(a: &Mat<N>) -> Option<Mat<N>> {
    let scale_ref = max_abs(a);
    if scale_ref == 0.0 || !all_finite(a) {
        return None;
    }
    let at = transpose(a);
    let mut ata = matmul(&at, a);
    let lambda = 1e-9 * scale_ref * scale_ref;
    for (i, row) in ata.iter_mut().enumerate() {
        row[i] += lambda;
    }
    let inv = inverse(&ata)?;
    let out = matmul(&inv, &at);
    all_finite(&out).then_some(out)
}

/// Relative Gain Array, `RGA = G .* inv(G)^T` (elementwise).
///
/// Reads as: how much the gain from input `j` to output `i` changes when every *other* loop is
/// closed. A diagonal of ones means the loops do not interact and decentralized control is already
/// correct. Entries far from one, or negative, mean the chosen pairing is fighting itself.
pub fn rga<const N: usize>(g: &Mat<N>) -> Option<Mat<N>> {
    let inv = inverse(g)?;
    let mut out = zeros::<N>();
    for i in 0..N {
        for j in 0..N {
            // (inv^T)[i][j] == inv[j][i]
            out[i][j] = g[i][j] * inv[j][i];
        }
    }
    all_finite(&out).then_some(out)
}

/// Niederlinski index, `det(G) / prod(g_ii)`.
///
/// A negative value proves that the decentralized loop with this pairing cannot be stabilised by
/// any set of controllers with integral action — a structural result, not a tuning problem. It is
/// only a sufficient condition for instability, so a positive value is not a guarantee of health.
pub fn niederlinski<const N: usize>(g: &Mat<N>) -> f64 {
    let mut denom = 1.0;
    for (i, row) in g.iter().enumerate() {
        denom *= row[i];
    }
    if denom == 0.0 {
        return f64::NAN;
    }
    determinant(g) / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f64, b: f64, tol: f64, what: &str) {
        assert!((a - b).abs() <= tol, "{what}: {a} vs {b} (tol {tol})");
    }

    fn assert_mat_close<const N: usize>(a: &Mat<N>, b: &Mat<N>, tol: f64) {
        for i in 0..N {
            for j in 0..N {
                assert!(
                    (a[i][j] - b[i][j]).abs() <= tol,
                    "[{i}][{j}]: {} vs {}",
                    a[i][j],
                    b[i][j]
                );
            }
        }
    }

    #[test]
    fn inverse_round_trips_to_identity() {
        let a: Mat<4> = [
            [4.0, 1.0, 0.3, 0.1],
            [1.2, 5.0, 1.1, 0.2],
            [0.4, 1.3, 6.0, 1.4],
            [0.1, 0.3, 1.5, 3.0],
        ];
        let inv = inverse(&a).expect("invertible");
        assert_mat_close(&matmul(&a, &inv), &identity::<4>(), 1e-12);
        assert_mat_close(&matmul(&inv, &a), &identity::<4>(), 1e-12);
    }

    #[test]
    fn inverse_needs_pivoting_to_survive_a_zero_leading_entry() {
        // Fails immediately without partial pivoting: the first pivot is exactly zero.
        let a: Mat<4> = [
            [0.0, 2.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 4.0],
            [0.0, 0.0, 3.0, 0.0],
        ];
        let inv = inverse(&a).expect("permutation matrices are invertible");
        assert_mat_close(&matmul(&a, &inv), &identity::<4>(), 1e-12);
    }

    #[test]
    fn inverse_refuses_a_singular_matrix() {
        // Row 2 is twice row 0.
        let a: Mat<4> = [
            [1.0, 2.0, 3.0, 4.0],
            [0.0, 1.0, 0.0, 0.0],
            [2.0, 4.0, 6.0, 8.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        assert!(inverse(&a).is_none());
        assert!(rga(&a).is_none());
    }

    #[test]
    fn determinant_matches_the_closed_form() {
        let a: Mat<2> = [[3.0, 7.0], [1.0, -4.0]];
        assert_close(determinant(&a), 3.0 * -4.0 - 7.0 * 1.0, 1e-12, "det 2x2");

        let d: Mat<4> = diag(&[2.0, 3.0, 4.0, 5.0]);
        assert_close(determinant(&d), 120.0, 1e-12, "det diag");
    }

    #[test]
    fn singular_values_of_a_diagonal_matrix_are_its_entries() {
        let d: Mat<4> = diag(&[3.0, -5.0, 1.0, 0.5]);
        let sv = singular_values(&d);
        // Descending magnitudes.
        assert_close(sv[0], 5.0, 1e-10, "sv0");
        assert_close(sv[1], 3.0, 1e-10, "sv1");
        assert_close(sv[2], 1.0, 1e-10, "sv2");
        assert_close(sv[3], 0.5, 1e-10, "sv3");
        assert_close(condition_number(&d), 10.0, 1e-9, "cond");
    }

    #[test]
    fn singular_values_are_rotation_invariant() {
        // An orthogonal matrix has all singular values 1 and condition number 1.
        let c = std::f64::consts::FRAC_1_SQRT_2;
        let q: Mat<2> = [[c, -c], [c, c]];
        let sv = singular_values(&q);
        assert_close(sv[0], 1.0, 1e-12, "sv0");
        assert_close(sv[1], 1.0, 1e-12, "sv1");
        assert_close(condition_number(&q), 1.0, 1e-10, "cond of a rotation");
    }

    #[test]
    fn condition_number_is_infinite_when_singular() {
        let a: Mat<2> = [[1.0, 2.0], [2.0, 4.0]];
        assert!(condition_number(&a).is_infinite());
    }

    #[test]
    fn rga_of_a_diagonal_plant_is_the_identity() {
        let g: Mat<4> = diag(&[2.0, -3.0, 0.5, 10.0]);
        let r = rga(&g).expect("diagonal is invertible");
        assert_mat_close(&r, &identity::<4>(), 1e-12);
    }

    #[test]
    fn rga_rows_and_columns_sum_to_one() {
        // A defining property of the RGA, and a strong check on the elementwise/transpose wiring.
        let g: Mat<4> = [
            [4.0, 1.0, 0.3, 0.1],
            [1.2, 5.0, 1.1, 0.2],
            [0.4, 1.3, 6.0, 1.4],
            [0.1, 0.3, 1.5, 3.0],
        ];
        let r = rga(&g).expect("invertible");
        for i in 0..4 {
            let row: f64 = r[i].iter().sum();
            let col: f64 = (0..4).map(|k| r[k][i]).sum();
            assert_close(row, 1.0, 1e-10, "row sum");
            assert_close(col, 1.0, 1e-10, "column sum");
        }
    }

    #[test]
    fn rga_diagonal_moves_away_from_one_as_coupling_grows() {
        let weak: Mat<2> = [[1.0, 0.05], [0.05, 1.0]];
        let strong: Mat<2> = [[1.0, 0.8], [0.8, 1.0]];
        let dw = rga(&weak).unwrap()[0][0];
        let ds = rga(&strong).unwrap()[0][0];
        assert!(
            (dw - 1.0).abs() < (ds - 1.0).abs(),
            "weak {dw} should sit closer to 1 than strong {ds}"
        );
    }

    #[test]
    fn niederlinski_is_one_for_a_diagonal_plant() {
        let g: Mat<4> = diag(&[2.0, 3.0, 4.0, 5.0]);
        assert_close(niederlinski(&g), 1.0, 1e-12, "NI");
    }

    #[test]
    fn niederlinski_goes_negative_on_a_badly_paired_plant() {
        // Off-diagonal dominance: det flips sign against the product of the diagonal.
        let g: Mat<2> = [[1.0, 2.0], [2.0, 1.0]];
        assert!(niederlinski(&g) < 0.0, "expected a negative NI");
    }

    #[test]
    fn pseudo_inverse_matches_the_true_inverse_when_well_conditioned() {
        let a: Mat<4> = [
            [4.0, 1.0, 0.3, 0.1],
            [1.2, 5.0, 1.1, 0.2],
            [0.4, 1.3, 6.0, 1.4],
            [0.1, 0.3, 1.5, 3.0],
        ];
        let inv = inverse(&a).unwrap();
        let pinv = pseudo_inverse(&a).unwrap();
        assert_mat_close(&inv, &pinv, 1e-7);
    }

    #[test]
    fn pseudo_inverse_stays_bounded_on_a_singular_matrix() {
        // The anti-windup path depends on this: no inverse exists, but the result must not be
        // infinite, because it gets subtracted from the integral state every tick.
        let a: Mat<2> = [[1.0, 2.0], [2.0, 4.0]];
        let pinv = pseudo_inverse(&a).expect("damped inverse always exists");
        assert!(all_finite(&pinv), "pinv must stay finite: {pinv:?}");
    }

    #[test]
    fn matmul_and_matvec_agree() {
        let a: Mat<3> = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]];
        let x = [1.0, -2.0, 0.5];
        let via_vec = matvec(&a, &x);
        let via_mat = matmul(&a, &diag(&x));
        for i in 0..3 {
            let summed: f64 = via_mat[i].iter().sum();
            assert_close(via_vec[i], summed, 1e-12, "row");
        }
    }
}
